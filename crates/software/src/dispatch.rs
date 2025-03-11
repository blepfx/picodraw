use std::{
    alloc::{Layout, alloc_zeroed},
    sync::mpsc::{Receiver, Sender},
};

use crate::vm::{
    CompiledShader, PIXEL_COUNT, VMInterpreter, VMProgram, VMRegister, VMSlot, VMTile,
};
use fxhash::FxHashMap;
use picodraw_core::{Bounds, Size};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub struct DispatchObject<'a> {
    pub shader: &'a CompiledShader,
    pub data: &'a [VMSlot],
    pub bounds: Bounds,
}

pub struct DispatchRequest<'a> {
    pub objects: &'a [DispatchObject<'a>],
    pub bounds: Bounds,

    pub target_size: Size,
    pub target_buffer: &'a mut [u32],
}

pub struct Dispatcher {
    sender: Sender<(u32, u32, DispatchTile)>,
    receiver: Receiver<(u32, u32, DispatchTile)>,
    tiles: FxHashMap<(u32, u32), Vec<u32>>,
}

struct DispatchTile(pub Box<[u32; PIXEL_COUNT]>);

impl DispatchTile {
    pub fn new() -> Self {
        unsafe {
            Self(Box::from_raw(
                alloc_zeroed(Layout::new::<[u32; PIXEL_COUNT]>()) as *mut _,
            ))
        }
    }

    pub fn blend_result(&mut self, bounds: Bounds, r: &[f32], g: &[f32], b: &[f32], a: &[f32]) {
        let bounds = bounds.intersect(Bounds {
            left: 0,
            top: 0,
            right: PIXEL_COUNT as u32,
            bottom: PIXEL_COUNT as u32,
        });

        for y in bounds.top..bounds.bottom {
            for x in bounds.left..bounds.right {
                let i = y as usize * PIXEL_COUNT + x as usize;
                let r0 = (self.0[i] & 0xFF) as f32 * (1.0 / 255.0);
                let g0 = ((self.0[i] >> 8) & 0xFF) as f32 * (1.0 / 255.0);
                let b0 = ((self.0[i] >> 16) & 0xFF) as f32 * (1.0 / 255.0);
                let a0 = (self.0[i] >> 24) as f32 * (1.0 / 255.0);

                let r1 = r[i as usize];
                let g1 = g[i as usize];
                let b1 = b[i as usize];
                let a1 = a[i as usize];

                let r2 = r0 * (1.0 - a1) + r1 * a1;
                let g2 = g0 * (1.0 - a1) + g1 * a1;
                let b2 = b0 * (1.0 - a1) + b1 * a1;
                let a2 = a0 * (1.0 - a1) + a1;

                self.0[i] = ((a2 * 255.0) as u32) << 24
                    | ((b2 * 255.0) as u32) << 16
                    | ((g2 * 255.0) as u32) << 8
                    | (r2 * 255.0) as u32;
            }
        }
    }

    pub fn blit_into(
        &self,
        bounds: Bounds,
        target: &mut [u32],
        target_size: Size,
        target_x: u32,
        target_y: u32,
    ) {
        let bounds = bounds.intersect(Bounds {
            left: 0,
            top: 0,
            right: PIXEL_COUNT as u32,
            bottom: PIXEL_COUNT as u32,
        });

        for y in 0..bounds.height().min(target_size.height - target_y) {
            for x in 0..bounds.width().min(target_size.width - target_x) {
                let i = (y + target_y) * target_size.width + (x + target_x);
                let j = (y + bounds.top) * PIXEL_COUNT as u32 + (x + bounds.left);
                target[i as usize] = self.0[j as usize];
            }
        }
    }
}

impl Dispatcher {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            sender,
            receiver,
            tiles: FxHashMap::default(),
        }
    }

    pub fn dispatch(&mut self, mut request: DispatchRequest) {
        // step 1: split objects into tiles
        self.tiles.clear();

        for (index, object) in request.objects.iter().enumerate() {
            let bounds = object.bounds.intersect(request.bounds);
            let x0 = bounds.left / PIXEL_COUNT as u32;
            let y0 = bounds.top / PIXEL_COUNT as u32;
            let x1 = (bounds.right + PIXEL_COUNT as u32 - 1) / PIXEL_COUNT as u32;
            let y1 = (bounds.bottom + PIXEL_COUNT as u32 - 1) / PIXEL_COUNT as u32;

            for y in y0..=y1 {
                for x in x0..=x1 {
                    self.tiles
                        .entry((x, y))
                        .or_insert_with(|| vec![])
                        .push(index as u32);
                }
            }
        }

        self.tiles.par_iter().for_each(|((x, y), objects)| {
            let mut tile = DispatchTile::new();
            let mut interpreter = VMInterpreter::<VMTile>::new();

            for object in objects {
                let object = &request.objects[*object as usize];

                unsafe {
                    interpreter.execute(VMProgram {
                        ops: object.shader.opcodes(),
                        data: object.data,
                        tile_x: (*x * PIXEL_COUNT as u32) as f32,
                        tile_y: (*y * PIXEL_COUNT as u32) as f32,
                        quad_t: object.bounds.top as f32,
                        quad_b: object.bounds.bottom as f32,
                        quad_l: object.bounds.left as f32,
                        quad_r: object.bounds.right as f32,
                        res_x: request.target_size.width as f32,
                        res_y: request.target_size.height as f32,
                    })
                };

                tile.blend_result(
                    object.bounds.offset(
                        -(*x as i32 * PIXEL_COUNT as i32),
                        -(*y as i32 * PIXEL_COUNT as i32),
                    ),
                    interpreter
                        .register(object.shader.output_register(0))
                        .as_f32(),
                    interpreter
                        .register(object.shader.output_register(1))
                        .as_f32(),
                    interpreter
                        .register(object.shader.output_register(2))
                        .as_f32(),
                    interpreter
                        .register(object.shader.output_register(3))
                        .as_f32(),
                );
            }

            let _ = self.sender.send((*x, *y, tile));
        });

        for (x, y, tile) in self.receiver.iter() {
            tile.blit_into(
                request.bounds.offset(
                    -(x as i32 * PIXEL_COUNT as i32),
                    -(y as i32 * PIXEL_COUNT as i32),
                ),
                &mut request.target_buffer,
                request.target_size,
                x * PIXEL_COUNT as u32,
                y * PIXEL_COUNT as u32,
            );
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {}
}

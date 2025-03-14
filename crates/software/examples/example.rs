use picodraw_core::{
    Graph,
    shader::{float4, io},
};
use picodraw_software::{CompiledShader, DispatchBuffer, Dispatcher, VMSlot};

fn main() {
    let arena = bumpalo::Bump::new();
    let start = std::time::Instant::now();
    let shader = CompiledShader::compile(
        &arena,
        &Graph::collect(|| {
            let z = io::read::<f32>();
            let x = io::position();
            let r = io::resolution().x().max(io::resolution().y());

            let sdf = (x - r * 0.5).len() - r * z * 0.25;
            let alpha = 1.0 - sdf.smoothstep(-0.6, 0.6);

            float4((x.x() / r, x.y() / r, 1.0, alpha * 0.5))
        }),
    );

    println!(
        "compiled in {:?} (used {}KB) \n{:?}",
        start.elapsed(),
        arena.allocated_bytes() / 1024,
        shader
    );

    let mut arena = bumpalo::Bump::new();
    let mut thread_pool = picodraw_software::ThreadPool::new();
    let start = std::time::Instant::now();

    const ITERS: usize = 10000;
    for i in 0..ITERS {
        let mut buffer = arena.alloc_slice_fill_default(512 * 512);
        let mut dispatcher = Dispatcher::new(&arena, DispatchBuffer {
            buffer: &mut buffer,
            width: 512,
            height: 512,
            bounds: [0, 0, 512, 512].into(),
        });

        for i in 0..10 {
            dispatcher.write_start([0, 0, 512, 512], &shader);
            dispatcher.write_data(&[VMSlot {
                float: 2.0 - i as f32 * 0.15,
            }]);
            dispatcher.write_end();
        }
        dispatcher.dispatch(&mut thread_pool);

        if i == 0 {
            println!("memory per draw: {}KB", arena.allocated_bytes() / 1024);

            image::RgbaImage::from_fn(512, 512, |x, y| {
                let color = buffer[(y * 512 + x) as usize];
                image::Rgba([
                    (color >> 16) as u8,
                    (color >> 8) as u8,
                    color as u8,
                    (color >> 24) as u8,
                ])
            })
            .save("test.png")
            .unwrap();
        }

        arena.reset();
    }
    println!("time per frame: {:?}", start.elapsed() / ITERS as u32);
}

use std::u32;

use bumpalo::{Bump, collections::Vec};

#[repr(align(64))]
struct Node([u32; 16]);

pub struct SparseMap<'a, T> {
    leafs: Vec<'a, Vec<'a, T>>,
    nodes: Vec<'a, Node>,

    roots: Vec<'a, u32>,
    width: u32,
    height: u32,
}

impl<'a, T: Copy> SparseMap<'a, T> {
    pub fn new(bump: &'a Bump, width: u32, height: u32) -> Self {
        let width = width.div_ceil(16);
        let height = height.div_ceil(16);

        Self {
            leafs: Vec::new_in(bump),
            nodes: Vec::new_in(bump),
            roots: Vec::from_iter_in((0..width * height).map(|_| u32::MAX), bump),
            width,
            height,
        }
    }

    pub unsafe fn get_unchecked(&self, x: u32, y: u32) -> &[T] {
        unsafe {
            let (x0, y0) = (x >> 4, y >> 4);
            let (x1, y1) = ((x >> 2) & 3, (y >> 2) & 3);
            let (x2, y2) = (x & 3, y & 3);

            let node0 = *self.roots.get_unchecked((x0 + self.width * y0) as usize);
            let node1 = *self
                .nodes
                .get_unchecked(node0 as usize)
                .0
                .get_unchecked((x1 + y1 * 4) as usize);
            let node2 = *self
                .nodes
                .get_unchecked(node1 as usize)
                .0
                .get_unchecked((x2 + y2 * 4) as usize);

            self.leafs.get_unchecked(node2 as usize)
        }
    }

    pub fn iter_non_empty(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        todo!()
    }

    pub fn push(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, value: T) {}
}

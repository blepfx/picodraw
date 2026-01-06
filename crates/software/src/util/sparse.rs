use bumpalo::{Bump, collections::Vec};

pub struct SparseMap<'a, T> {
    buckets: Vec<'a, (u32, u32, Vec<'a, T>)>,
    mapping: Vec<'a, u32>,
    width: u32,
    height: u32,
}

impl<'a, T> SparseMap<'a, T> {
    pub fn new(width: u32, height: u32, arena: &'a Bump) -> Self {
        assert!(width.checked_mul(height).is_some(), "dimensions too large");

        Self {
            buckets: Vec::new_in(arena),
            mapping: Vec::from_iter_in((0..width * height).map(|_| u32::MAX), arena),
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn push(&mut self, x: u32, y: u32, value: T) {
        assert!(x < self.width && y < self.height);

        let mapping = &mut self.mapping[(x + y * self.width) as usize];
        if *mapping == u32::MAX {
            *mapping = self.buckets.len() as u32;
            self.buckets
                .push((x, y, Vec::from_iter_in([value], self.buckets.bump())));
        } else {
            self.buckets[*mapping as usize].2.push(value);
        }
    }
}

impl<'a, T> IntoIterator for SparseMap<'a, T> {
    type Item = (u32, u32, Vec<'a, T>);
    type IntoIter = <Vec<'a, (u32, u32, Vec<'a, T>)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.buckets.into_iter()
    }
}

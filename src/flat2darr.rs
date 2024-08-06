use crate::{ WIDTH, HEIGHT };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Flat2DArray<T> {
    pub data: [T; WIDTH * HEIGHT],
}

impl<T: Default + Copy> Flat2DArray<T> {
    pub fn new() -> Self {
        let data = [T::default(); WIDTH * HEIGHT];
        Flat2DArray { data }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row < HEIGHT && col < WIDTH { Some(&self.data[row * WIDTH + col]) } else { None }
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut T> {
        if row < HEIGHT && col < WIDTH { Some(&mut self.data[row * WIDTH + col]) } else { None }
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) {
        if row < HEIGHT && col < WIDTH {
            self.data[row * WIDTH + col] = value;
        }
    }
}

use crate::vec::{VecElement, Vector};

#[derive(Clone, Debug)]
pub struct Ray<T: VecElement, const N: usize> {
    pub origin: Vector<T, N>,
    pub direction: Vector<T, N>,
}

impl<T: VecElement, const N: usize> Ray<T, N> {
    pub fn at(&self, t: T) -> Vector<T, N> {
        self.origin + self.direction * t
    }
}

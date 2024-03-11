use crate::vec::{VecElement, Vector};

struct Ray<T: VecElement, const N: usize> {
    pub origin: Vector<T, N>,
    pub direction: Vector<T, N>,
}

impl<T: VecElement, const N: usize> Ray<T, N> {
    pub fn at(&self, t: T) -> Vector<T, N> {
        self.origin.clone() + self.direction.clone() * t
    }
}

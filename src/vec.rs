#![allow(dead_code)]

use num::traits::Num;
use std::ops::{Add, Div, Mul, Neg, Sub};

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: VecElement, const N: usize> $trait for Vector<T, N> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                let mut data = [T::default(); N];
                for i in 0..N {
                    data[i] = self.data[i] $op rhs.data[i];
                }
                Self { data }
            }
        }

        impl<T: VecElement, const N: usize> $trait<T> for Vector<T, N> {
            type Output = Self;

            fn $method(self, rhs: T) -> Self::Output {
                let mut data = [T::default(); N];
                data.iter_mut().for_each(|x| *x = *x $op rhs);
                Self { data }
            }
        }
    };
}

pub trait VecElement: Copy + Default + Num + Neg<Output = Self> {}

// blanket implementation for VecElement
impl<T> VecElement for T where T: Copy + Default + Num + Neg<Output = Self> {}

/// Mathematical object vector struct
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Vector<T: VecElement, const N: usize> {
    pub data: [T; N],
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);

impl<T: VecElement, const N: usize> Neg for Vector<T, N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut data = [T::default(); N];
        for i in 0..N {
            data[i] = -self.data[i];
        }
        Self { data }
    }
}

impl<T: VecElement, const N: usize> Default for Vector<T, N> {
    fn default() -> Self {
        Self {
            data: [T::default(); N],
        }
    }
}

impl<T: VecElement, const N: usize> Vector<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Self { data }
    }

    pub fn dot(&self) -> T {
        self.data.iter().fold(T::default(), |acc, x| acc + *x * *x)
    }

    pub fn length_squared(&self) -> T {
        self.dot()
    }
}

// Ideally I want this method to be able to return T and operate on T as long sqrt is defined for T
impl<T: VecElement + Into<f64> + From<f64>, const N: usize> Vector<T, N> {
    pub fn length(&self) -> f64 {
        let length_squared: f64 = self.length_squared().into();
        length_squared.sqrt()
    }

    pub fn unit_vector(&self) -> Vector<T, N> {
        let length = self.length();
        let mut data = [T::default(); N];
        for i in 0..N {
            let element: f64 = self.data[i].into();
            let new_data = element / length;
            data[i] = T::from(new_data);
        }
        Vector::new(data)
    }
}

impl<T: VecElement> Vector<T, 3> {
    pub fn cross(&self, rhs: Self) -> Self {
        let ([x0, y0, z0], [x1, y1, z1]) = (self.data, rhs.data);
        Self::new([
            y0 * z1 - z0 * y1, // x
            z0 * x1 - x0 * z1, // y
            x0 * y1 - y0 * x1, // z
        ])
    }
}

// // Rust can't do this, because of the orphan rule:
// //       read: https://users.rust-lang.org/t/operator-overloading-and-generics/77485/6
// // What a shame, no symmetric operator for Vector sadly
// impl<T: VecElement, const N: usize> Add<Vector<T, N>> for T {
//     type Output = Vector<T, N>;

//     fn add(self, rhs: Vector<T, N>) -> Self::Output {
//         let mut data = [T::default(); N];
//         data.iter_mut().for_each(|x| *x = *x + rhs);
//         Vector { data }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        let a = Vector::new([1.0, 2.0, 3.0]);
        let b = Vector::new([4.0, 5.0, 6.0]);

        assert_eq!(a.clone() + b.clone(), Vector::new([5.0, 7.0, 9.0]));
        assert_eq!(a.clone() - b.clone(), Vector::new([-3.0, -3.0, -3.0]));
        assert_eq!(a.clone() * b.clone(), Vector::new([4.0, 10.0, 18.0]));
        assert_eq!(
            a.clone() / b.clone(),
            Vector::new([1.0 / 4.0, 2.0 / 5.0, 3.0 / 6.0])
        );
        assert_eq!((-a).clone(), Vector::new([-1.0, -2.0, -3.0]));
    }
}

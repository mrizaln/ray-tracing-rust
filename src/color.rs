#![allow(dead_code)]

use crate::interval::Interval;
use crate::util;
use crate::vec::{VecElement, Vector};
use std::ops::{Add, Div, Mul, Neg, Sub};

macro_rules! gen_getter {
    ($name:ident, $index:literal) => {
        pub fn $name(&self) -> T {
            self.0.data[$index]
        }
    };
}

macro_rules! impl_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: VecElement> $trait for Color<T> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                Self(self.0 $op rhs.0)
            }
        }

        impl<T: VecElement> $trait<T> for Color<T> {
            type Output = Self;

            fn $method(self, rhs: T) -> Self::Output {
                Self(self.0 $op rhs)
            }
        }
    };
    ($trait:ident) => {
        impl<T: VecElement> $trait for Color<T> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color<T: VecElement = f64>(Vector<T, 3>);

impl<T: VecElement> Color<T> {
    pub fn new(data: [T; 3]) -> Self {
        Self(Vector::new(data))
    }

    pub fn new_one(value: T) -> Self {
        Self(Vector::new_one(value))
    }

    gen_getter!(r, 0);
    gen_getter!(g, 1);
    gen_getter!(b, 2);

    gen_getter!(h, 0);
    gen_getter!(s, 1);
    gen_getter!(v, 2);
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);
impl_op!(Neg);

impl<T: VecElement> From<Vector<T, 3>> for Color<T> {
    fn from(value: Vector<T, 3>) -> Self {
        Self(value)
    }
}

pub fn transform<T: VecElement, U: VecElement>(color: Color<T>, f: fn(T) -> U) -> Color<U> {
    let mut data = [U::default(); 3];
    for i in 0..3 {
        data[i] = f(color.0.data[i]);
    }
    Color::new(data)
}

pub fn clamp<T: VecElement + PartialOrd>(color: Color<T>, range: Interval<T>) -> Color<T> {
    let mut data = [T::default(); 3];
    for i in 0..3 {
        data[i] = range.clamp(color.0.data[i]);
    }
    Color::new(data)
}

pub fn correct_gamma<T: VecElement + Into<f64> + From<f64>>(color: Color<T>) -> Color<T> {
    let mut data = [T::default(); 3];
    for i in 0..3 {
        data[i] = T::from(util::linear_to_gamma(color.0.data[i].into()));
    }
    Color::new(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        assert_eq!(Color::<f64>::new_one(0.0).0.data, [0.0, 0.0, 0.0]);
        assert_eq!(
            Color::<f64>::new([1.2, 3.7545, 4.1910]).0.data,
            [1.2, 3.7545, 4.1910]
        );

        let vec = Vector::new([0.3, 10.23, 1.764]);
        assert_eq!(Color::from(vec).0.data, [0.3, 10.23, 1.764]);
    }

    #[test]
    fn test_getters() {
        let color = Color::new([1.3, 5.33, 2.9]);

        assert_eq!(color.0, Vector::new([1.3, 5.33, 2.9]));
        assert_eq!(color.0.data, [1.3, 5.33, 2.9]);

        let values = [1.3, 5.33, 2.9];
        assert_eq!(color.r(), values[0]);
        assert_eq!(color.g(), values[1]);
        assert_eq!(color.b(), values[2]);
        assert_eq!(color.h(), values[0]);
        assert_eq!(color.s(), values[1]);
        assert_eq!(color.v(), values[2]);
    }
}

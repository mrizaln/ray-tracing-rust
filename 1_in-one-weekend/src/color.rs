#![allow(dead_code)]

use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

use crate::interval::Interval;
use crate::util;
use crate::vec::{VecElement, Vector};

macro_rules! impl_binary_op {
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
}

macro_rules! impl_unary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: VecElement> $trait for Color<T> {
            type Output = Self;

            fn $method(self) -> Self::Output {
                Self($op self.0)
            }
        }
    };
}

macro_rules! gen_getter {
    ($name:ident, $name_mut:ident, $index:literal) => {
        pub fn $name(&self) -> &T {
            self.0.index($index)
        }

        pub fn $name_mut(&mut self) -> &T {
            self.0.index_mut($index)
        }
    };
    ($(($name:ident, $name_mut:ident, $index:literal)),+) => {
        $(gen_getter!($name, $name_mut, $index);)+
    }
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

    gen_getter!((r, r_mut, 0), (g, g_mut, 1), (b, b_mut, 2));
    gen_getter!((h, h_mut, 0), (s, s_mut, 1), (v, v_mut, 2));
}

impl_binary_op!(Add, add, +);
impl_binary_op!(Sub, sub, -);
impl_binary_op!(Mul, mul, *);
impl_binary_op!(Div, div, /);
impl_unary_op!(Neg, neg, -);

impl<T: VecElement> From<Vector<T, 3>> for Color<T> {
    fn from(value: Vector<T, 3>) -> Self {
        Self(value)
    }
}

impl<T: VecElement> Color<T> {
    pub fn transform<U: VecElement>(&self, f: fn(T) -> U) -> Color<U> {
        self.0.transform(f).into()
    }

    pub fn transform_into<U: VecElement>(&self) -> Color<U>
    where
        T: Into<U>,
    {
        self.0.transform_into().into()
    }

    pub fn clamp(&self, range: Interval<T>) -> Color<T>
    where
        T: PartialOrd,
    {
        self.0.transform(|x| range.clamp(x)).into()
    }

    pub fn correct_gamma(&self) -> Color<T>
    where
        T: Into<f64> + From<f64>,
    {
        self.transform(|x| util::linear_to_gamma(x.into()).into())
    }
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
        assert_eq!(color.r(), &values[0]);
        assert_eq!(color.g(), &values[1]);
        assert_eq!(color.b(), &values[2]);
        assert_eq!(color.h(), &values[0]);
        assert_eq!(color.s(), &values[1]);
        assert_eq!(color.v(), &values[2]);
    }

    #[test]
    fn test_operators() {
        let a = Color::new([1.0, 2.0, 3.0]);
        let b = Color::new([4.0, 5.0, 6.0]);

        assert_eq!(a + b, Color::new([5.0, 7.0, 9.0]));
        assert_eq!(a - b, Color::new([-3.0, -3.0, -3.0]));
        assert_eq!(a * b, Color::new([4.0, 10.0, 18.0]));
        assert_eq!(a / b, Color::new([1.0 / 4.0, 2.0 / 5.0, 3.0 / 6.0]));
        assert_eq!((-a), Color::new([-1.0, -2.0, -3.0]));

        let c = 5.33424;

        assert_eq!(a + c, Color::new([1.0 + c, 2.0 + c, 3.0 + c]));
        assert_eq!(a - c, Color::new([1.0 - c, 2.0 - c, 3.0 - c]));
        assert_eq!(a * c, Color::new([1.0 * c, 2.0 * c, 3.0 * c]));
        assert_eq!(a / c, Color::new([1.0 / c, 2.0 / c, 3.0 / c]));
    }
}

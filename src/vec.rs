#![allow(dead_code)]

use num::traits::Num;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::util;

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
                for i in 0..N {
                    data[i] = self.data[i] $op rhs;
                }
                Self { data }
            }
        }
    };
    ($trait:ident) => {
        impl<T: VecElement, const N: usize> $trait for Vector<T, N> {
            type Output = Self;

            fn neg(self) -> Self::Output {
                let mut data = [T::default(); N];
                for i in 0..N {
                    data[i] = -self.data[i];
                }
                Self { data }
            }
        }
    };
}

pub trait VecElement: Copy + Default + Num + Neg<Output = Self> {}

// blanket implementation for VecElement
impl<T> VecElement for T where T: Copy + Default + Num + Neg<Output = Self> {}

/// Mathematical object vector struct
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector<T: VecElement, const N: usize> {
    pub data: [T; N],
}

impl_op!(Add, add, +);
impl_op!(Sub, sub, -);
impl_op!(Mul, mul, *);
impl_op!(Div, div, /);
impl_op!(Neg);

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

    pub fn new_one(value: T) -> Self {
        let data = [value; N];
        Self { data }
    }

    pub fn dot(&self, other: Self) -> T {
        self.data
            .iter()
            .zip(other.data)
            .fold(T::default(), |acc, (l, r)| acc + *l * r)
    }

    pub fn length_squared(&self) -> T {
        self.dot(self.clone())
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

    pub fn near_zero(&self) -> bool {
        let delta = 1e-8;
        self.data
            .iter()
            .all(|x| (*x).into() < delta && (*x).into() > -delta)
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

pub fn reflect<T, const N: usize>(unit_vec: Vector<T, N>, normal: Vector<T, N>) -> Vector<T, N>
where
    T: VecElement + From<f64>,
{
    unit_vec - normal * unit_vec.dot(normal) * T::from(2.0)
}

// TODO: inspect this code for bugs
pub fn refract<T, const N: usize>(
    unit_vec: Vector<T, N>,
    normal: Vector<T, N>,
    refraction_ratio: f64,
) -> Vector<T, N>
where
    T: VecElement + From<f64> + Into<f64>,
{
    let cos_theta = Into::<f64>::into((-unit_vec).dot(normal)).min(1.0);
    let r_out_perpendicular = (unit_vec + normal * T::from(cos_theta)) * T::from(refraction_ratio);
    let diff = Into::<f64>::into(1.0 - r_out_perpendicular.length_squared().into()).abs();
    let r_out_parallel = -(normal * T::from(diff.sqrt()));

    r_out_perpendicular + r_out_parallel
}

pub fn random_vector<T, const N: usize>(from: T, to: T) -> Vector<T, N>
where
    T: VecElement + From<f64> + Into<f64>,
{
    let mut data = [T::default(); N];
    data.iter_mut()
        .for_each(|x| *x = util::get_random(from, to));
    Vector { data }
}

pub fn random_in_unit_sphere<T, const N: usize>() -> Vector<T, N>
where
    T: VecElement + From<f64> + Into<f64>,
{
    loop {
        let point = random_vector::<T, N>(T::from(-1.0), T::from(1.0));
        if point.length_squared().into() < 1.0 {
            break point;
        }
    }
}

pub fn random_unit_vector<T, const N: usize>() -> Vector<T, N>
where
    T: VecElement + From<f64> + Into<f64>,
{
    random_in_unit_sphere().unit_vector()
}

pub fn random_on_hemisphere<T, const N: usize>(normal: Vector<T, N>) -> Vector<T, N>
where
    T: VecElement + From<f64> + Into<f64>,
{
    let point = random_unit_vector::<T, N>();
    if point.dot(normal).into() > 0.0 {
        point
    } else {
        -point
    }
}

pub fn random_in_unit_disk<T>() -> Vector<T, 2>
where
    T: VecElement + From<f64> + Into<f64> + std::cmp::PartialOrd,
{
    loop {
        let point = random_vector::<T, 2>(T::from(-1.0), T::from(1.0));
        if point.length_squared().into() < 1.0 {
            break point;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functions() {
        let a = Vector::new([1.0, 2.0, 3.0]);

        assert_eq!(a.data, [1.0, 2.0, 3.0]);
        assert_eq!(a.length_squared(), 14.0);
        assert_eq!(a.length(), 14.0f64.sqrt());
        assert_eq!(
            a.unit_vector(),
            Vector::new([
                1.0 / 14.0f64.sqrt(),
                2.0 / 14.0f64.sqrt(),
                3.0 / 14.0f64.sqrt()
            ])
        );
    }

    #[test]
    fn test_operators() {
        let a = Vector::new([1.0, 2.0, 3.0]);
        let b = Vector::new([4.0, 5.0, 6.0]);

        assert_eq!(a + b, Vector::new([5.0, 7.0, 9.0]));
        assert_eq!(a - b, Vector::new([-3.0, -3.0, -3.0]));
        assert_eq!(a * b, Vector::new([4.0, 10.0, 18.0]));
        assert_eq!(a / b, Vector::new([1.0 / 4.0, 2.0 / 5.0, 3.0 / 6.0]));
        assert_eq!((-a), Vector::new([-1.0, -2.0, -3.0]));

        let c = 5.33424;

        assert_eq!(a + c, Vector::new([1.0 + c, 2.0 + c, 3.0 + c]));
        assert_eq!(a - c, Vector::new([1.0 - c, 2.0 - c, 3.0 - c]));
        assert_eq!(a * c, Vector::new([1.0 * c, 2.0 * c, 3.0 * c]));
        assert_eq!(a / c, Vector::new([1.0 / c, 2.0 / c, 3.0 / c]));
    }

    #[test]
    fn test_free_functions() {
        // reflect = v - 2 * dot(v, n) * n
        let a = Vector::new([
            -0.8471285155916642,
            -0.27185175689173274,
            -0.327858942211803,
        ]);
        let b = Vector::new([
            -0.4306728257561193,
            -0.2940344720125667,
            -0.8532670428555941,
        ]);

        let reflected = reflect(a, b);
    }
}

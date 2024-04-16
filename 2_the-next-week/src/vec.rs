#![allow(dead_code)]

use core::fmt::{self, Debug};
use std::cmp::PartialOrd;
use std::fmt::Display;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

use num::traits::Num;
use rand::distributions::uniform::SampleUniform;

use crate::util;

macro_rules! impl_binary_op {
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
}

macro_rules! impl_unary_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl<T: VecElement, const N: usize> $trait for Vector<T, N> {
            type Output = Self;

            fn $method(self) -> Self::Output {
                let mut data = [T::default(); N];
                for i in 0..N {
                    data[i] = $op self.data[i];
                }
                Self { data }
            }
        }
    };
}

macro_rules! gen_getter {
    ($name:ident, $name_mut:ident, $index:literal) => {
        pub fn $name(&self) -> &T {
            self.index($index)
        }

        pub fn $name_mut(&mut self) -> &mut T {
            self.index_mut($index)
        }
    };
    ($(($name:ident, $name_mut:ident, $index:literal)),+) => {
        $(gen_getter!($name, $name_mut, $index);)+
    };
}

pub trait VecElement: Copy + Default + Num + Neg<Output = Self> + Display {}

// blanket implementation for VecElement
impl<T> VecElement for T where T: Copy + Default + Num + Neg<Output = Self> + Display {}

/// Mathematical object vector struct
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector<T: VecElement, const N: usize> {
    pub data: [T; N],
}

impl<T: VecElement, const N: usize> Vector<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Self { data }
    }

    pub fn new_one(value: T) -> Self {
        [value; N].into()
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

    pub fn transform<U, F>(&self, f: F) -> Vector<U, N>
    where
        U: VecElement,
        F: Fn(T) -> U,
    {
        let mut data = [U::default(); N];
        data.iter_mut().zip(self.data).for_each(|(u, t)| *u = f(t));
        data.into()
    }

    pub fn transform_into<U>(&self) -> Vector<U, N>
    where
        U: VecElement,
        T: Into<U>,
    {
        self.transform(|v| v.into())
    }
}

impl<T: VecElement, const N: usize> Default for Vector<T, N> {
    fn default() -> Self {
        Self {
            data: [T::default(); N],
        }
    }
}

impl<T: VecElement, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(value: [T; N]) -> Self {
        Vector::new(value)
    }
}

impl<T: VecElement, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T: VecElement, const N: usize> IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl<T: VecElement, const N: usize> Display for Vector<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = self
            .data
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "Vector ({})", joined)
    }
}

impl<T: VecElement, const N: usize> Debug for Vector<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

// for operations that involve f64
impl<T, const N: usize> Vector<T, N>
where
    T: VecElement,
{
    pub fn length(&self) -> f64
    where
        T: Into<f64>,
    {
        let length_squared: f64 = self.length_squared().into();
        length_squared.sqrt()
    }

    pub fn unit_vector(&self) -> Vector<T, N>
    where
        T: From<f64> + Into<f64>,
    {
        let mut data = [T::default(); N];
        data.iter_mut().zip(self.data).for_each(|(u, t)| {
            let new_data: f64 = t.into() / self.length();
            *u = new_data.into();
        });
        data.into()
    }

    pub fn near_zero(&self) -> bool
    where
        T: Into<f64>,
    {
        const DELTA: f64 = 1e-8;
        self.data
            .iter()
            .all(|x| (*x).into() < DELTA && (*x).into() > -DELTA)
    }

    pub fn reflect(&self, normal: Vector<T, N>) -> Vector<T, N>
    where
        T: From<f64>,
    {
        *self - normal * self.dot(normal) * T::from(2.0)
    }

    pub fn refract(&self, normal: Vector<T, N>, refraction_ratio: f64) -> Vector<T, N>
    where
        T: From<f64> + Into<f64>,
    {
        let vec: Vector<f64, N> = self.transform_into();
        let normal: Vector<f64, N> = normal.transform_into();

        let cos_theta = (-vec).dot(normal).min(1.0);
        let r_out_perpendicular = (vec + normal * cos_theta) * refraction_ratio;
        let r_out_parallel = -normal * (1.0 - r_out_perpendicular.length_squared()).abs().sqrt();

        (r_out_perpendicular + r_out_parallel).transform_into()
    }
}

// special case for N = 3
impl<T: VecElement> Vector<T, 3> {
    pub fn cross(&self, rhs: Self) -> Self {
        let ([x0, y0, z0], [x1, y1, z1]) = (self.data, rhs.data);
        Self::from([
            y0 * z1 - z0 * y1, // x
            z0 * x1 - x0 * z1, // y
            x0 * y1 - y0 * x1, // z
        ])
    }
}

// special case for N = 7
impl<T: VecElement> Vector<T, 7> {
    pub fn cross(&self, rhs: Self) -> Self {
        let ([x1, x2, x3, x4, x5, x6, x7], [y1, y2, y3, y4, y5, y6, y7]) = (self.data, rhs.data);
        Self::from([
            x2 * y4 - x4 * y2 + x3 * y7 - x7 * y3 + x5 * y6 - x6 * y5, // x1
            x3 * y5 - x5 * y3 + x4 * y1 - x1 * y4 + x6 * y7 - x7 * y6, // x2
            x4 * y6 - x6 * y4 + x5 * y2 - x2 * y5 + x7 * y1 - x1 * y7, // x3
            x5 * y7 - x7 * y5 + x6 * y3 - x3 * y6 + x1 * y2 - x2 * y1, // x4
            x6 * y1 - x1 * y6 + x7 * y4 - x4 * y7 + x2 * y3 - x3 * y2, // x5
            x7 * y2 - x2 * y7 + x1 * y5 - x5 * y1 + x3 * y4 - x4 * y3, // x6
            x1 * y3 - x3 * y1 + x2 * y6 - x6 * y2 + x4 * y5 - x5 * y4, // x7
        ])
    }
}

impl_binary_op!(Add, add, +);
impl_binary_op!(Sub, sub, -);
impl_binary_op!(Mul, mul, *);
impl_binary_op!(Div, div, /);
impl_unary_op!(Neg, neg, -);

// getters for common vector sizes

impl<T: VecElement> Vector<T, 1> {
    gen_getter!((x, x_mut, 0), (x1, x1_mut, 0));
}

impl<T: VecElement> Vector<T, 2> {
    gen_getter!((x, x_mut, 0), (x1, x1_mut, 0), (u, u_mut, 0));
    gen_getter!((y, y_mut, 1), (x2, x2_mut, 1), (v, v_mut, 0));
}

impl<T: VecElement> Vector<T, 3> {
    gen_getter!((x, x_mut, 0), (x1, x1_mut, 0), (u, u_mut, 0));
    gen_getter!((y, y_mut, 1), (x2, x2_mut, 1), (v, v_mut, 0));
    gen_getter!((z, z_mut, 2), (x3, x3_mut, 2), (w, w_mut, 0)); // w: 3rd dim for tex
}

impl<T: VecElement> Vector<T, 4> {
    gen_getter!((x, x_mut, 0), (x1, x1_mut, 0));
    gen_getter!((y, y_mut, 1), (x2, x2_mut, 1));
    gen_getter!((z, z_mut, 2), (x3, x3_mut, 2));
    gen_getter!((w, w_mut, 3), (x4, x4_mut, 3)); // w: 4th dim no tex
}

// // Rust can't do this, because of the orphan rule:
// //       read: https://users.rust-lang.org/t/operator-overloading-and-generics/77485/6
// // What a shame, no symmetric binary operator for Vector with scalar sadly
// impl<T: VecElement, const N: usize> Add<Vector<T, N>> for T {
//     type Output = Vector<T, N>;

//     fn add(self, rhs: Vector<T, N>) -> Self::Output {
//         let mut data = [T::default(); N];
//         data.iter_mut().for_each(|x| *x = *x + rhs);
//         Vector { data }
//     }
// }

pub fn random_vector<T, const N: usize>(from: T, to: T) -> Vector<T, N>
where
    T: VecElement + SampleUniform,
{
    let mut data = [T::default(); N];
    data.iter_mut()
        .for_each(|x| *x = util::get_random(from, to));
    Vector { data }
}

pub fn random_in_unit_sphere<T, const N: usize>() -> Vector<T, N>
where
    T: VecElement + SampleUniform + From<f64> + Into<f64>,
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
    T: VecElement + SampleUniform + From<f64> + Into<f64>,
{
    random_in_unit_sphere().unit_vector()
}

pub fn random_on_hemisphere<T, const N: usize>(normal: Vector<T, N>) -> Vector<T, N>
where
    T: VecElement + SampleUniform + From<f64> + Into<f64>,
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
    T: VecElement + SampleUniform + From<f64> + Into<f64> + PartialOrd,
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

    // #[test]
    // fn test_free_functions() {
    //     // reflect = v - 2 * dot(v, n) * n
    //     let a = Vector::new([
    //         -0.8471285155916642,
    //         -0.27185175689173274,
    //         -0.327858942211803,
    //     ]);
    //     let b = Vector::new([
    //         -0.4306728257561193,
    //         -0.2940344720125667,
    //         -0.8532670428555941,
    //     ]);

    //     let reflected = reflect(a, b);
    // }
}

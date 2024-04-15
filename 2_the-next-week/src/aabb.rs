use std::array;
use std::cmp::Ordering::{Equal, Greater, Less};

use num::Float;

use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec::{VecElement, Vector};

#[derive(Clone, Debug)]
pub struct AABB<T, const N: usize>
where
    T: VecElement + PartialOrd,
{
    intervals: [Interval<T>; N],
}

impl<T, const N: usize> AABB<T, N>
where
    T: VecElement + PartialOrd,
{
    pub fn new(intervals: [Interval<T>; N]) -> Self {
        Self { intervals }
    }

    pub fn from_points(a: Vector<T, N>, b: Vector<T, N>) -> Self {
        AABB::new(array::from_fn(|i| match a[i].partial_cmp(&b[i]) {
            Some(Less) => Interval::new(a[i], b[i]),
            Some(Equal) => Interval::new(a[i], b[i]),
            Some(Greater) => Interval::new(b[i], a[i]),
            None => panic!("Invalid comparison"),
        }))
    }

    pub fn empty() -> Self
    where
        T: Float,
    {
        AABB::new(array::from_fn(|_| Interval::empty()))
    }

    pub fn universe() -> Self
    where
        T: Float,
    {
        AABB::new(array::from_fn(|_| Interval::universe()))
    }

    pub fn combine_new(mut self, other: &Self) -> Self {
        for (idx, int) in self.intervals.iter_mut().enumerate() {
            int.combine(&other.intervals[idx]);
        }
        self
    }

    pub fn combine(&mut self, other: &Self) -> &mut Self {
        for (idx, int) in self.intervals.iter_mut().enumerate() {
            int.combine(&other.intervals[idx]);
        }
        self
    }

    pub fn axis_interval(&self, axis: usize) -> &Interval<T> {
        &self.intervals[axis]
    }

    pub fn hit(&self, ray: Ray<T, N>, mut interval: Interval<T>) -> bool {
        for (ax, int) in self.intervals.iter().enumerate() {
            let t0 = (int.min - ray.origin[ax]) / ray.direction[ax];
            let t1 = (int.max - ray.origin[ax]) / ray.direction[ax];

            // it will be easier if i were to use min and max function, but it needs Ord, i'm not
            // sure i want to add that trait bound...
            if t0 < t1 {
                if t0 > interval.min {
                    interval.min = t0
                }
                if t1 < interval.max {
                    interval.max = t1
                }
            } else {
                if t1 > interval.min {
                    interval.min = t1
                }
                if t0 < interval.max {
                    interval.max = t0
                }
            }

            if interval.min >= interval.max {
                return false;
            }
        }
        true
    }
}

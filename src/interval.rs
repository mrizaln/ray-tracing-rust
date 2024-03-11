#![allow(dead_code)]

use num::Num;

#[derive(Clone, Debug)]
pub struct Interval<T: Num + PartialOrd + Clone> {
    min: T,
    max: T,
}

impl<T: Num + PartialOrd + Clone> Interval<T> {
    fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    fn contains(&self, value: T) -> bool {
        self.min <= value && value <= self.max
    }

    fn surrounds(&self, value: T) -> bool {
        self.min < value && value < self.max
    }

    fn contains_interval(&self, other: &Self) -> bool {
        self.min <= other.min && other.max <= self.max
    }

    fn surrounds_interval(&self, other: &Self) -> bool {
        self.min < other.min && other.max < self.max
    }

    fn clamp(&self, value: T) -> T {
        if value < self.min {
            self.min.clone()
        } else if value > self.max {
            self.max.clone()
        } else {
            value
        }
    }
}

#![allow(dead_code)]

use num::Num;

#[derive(Clone, Debug)]
pub struct Interval<T: Num + PartialOrd + Clone = f64> {
    pub min: T,
    pub max: T,
}

impl<T: Num + PartialOrd + Clone> Interval<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, value: T) -> bool {
        self.min <= value && value <= self.max
    }

    pub fn surrounds(&self, value: T) -> bool {
        self.min < value && value < self.max
    }

    pub fn contains_interval(&self, other: &Self) -> bool {
        self.min <= other.min && other.max <= self.max
    }

    pub fn surrounds_interval(&self, other: &Self) -> bool {
        self.min < other.min && other.max < self.max
    }

    pub fn clamp(&self, value: T) -> T {
        match value {
            v if v < self.min => self.min.clone(),
            v if v > self.max => self.max.clone(),
            _ => value,
        }
    }
}

impl<T: Num + PartialOrd + Clone> From<(T, T)> for Interval<T> {
    fn from(value: (T, T)) -> Self {
        Interval::new(value.0, value.1)
    }
}

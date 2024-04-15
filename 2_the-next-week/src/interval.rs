#![allow(dead_code)]

use num::{Float, Num};

#[derive(Clone, Debug)]
pub struct Interval<T: Num + PartialOrd + Clone = f64> {
    pub min: T,
    pub max: T,
}

impl<T: Num + PartialOrd + Clone> Interval<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn combine_new(&self, other: &Self) -> Self {
        Self {
            min: if self.min < other.min {
                self.min.clone()
            } else {
                other.min.clone()
            },
            max: if self.max > other.max {
                self.max.clone()
            } else {
                other.max.clone()
            },
        }
    }

    pub fn combine(&mut self, other: &Self) -> &mut Self {
        self.min = if self.min < other.min {
            self.min.clone()
        } else {
            other.min.clone()
        };
        self.max = if self.max > other.max {
            self.max.clone()
        } else {
            other.max.clone()
        };
        self
    }

    pub fn expand_new(&self, padding: T) -> Self {
        Interval::new(
            self.min.clone() - padding.clone(),
            self.max.clone() + padding.clone(),
        )
    }

    pub fn expand(&mut self, padding: T) -> &mut Self {
        self.min = self.min.clone() - padding.clone();
        self.max = self.max.clone() + padding.clone();
        self
    }

    pub fn empty() -> Self
    where
        T: Float,
    {
        Self {
            min: Float::infinity(),
            max: Float::neg_infinity(),
        }
    }

    pub fn universe() -> Self
    where
        T: Float,
    {
        Self {
            min: Float::neg_infinity(),
            max: Float::infinity(),
        }
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

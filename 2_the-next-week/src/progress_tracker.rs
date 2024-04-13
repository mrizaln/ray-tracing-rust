use std::ops::{Add, Div};
use std::time::{Duration, Instant};

struct MovingAverage<T, const N: usize>
where
    T: Add<T, Output = T> + Div<usize, Output = T> + Default + Copy,
{
    entries: Box<[T; N]>,
    index: usize,
    average: T,
    full: bool,
}

impl<T, const N: usize> MovingAverage<T, N>
where
    T: Add<T, Output = T> + Div<usize, Output = T> + Default + Copy,
{
    pub fn new() -> Self {
        Self {
            entries: Box::new([T::default(); N]),
            index: 0,
            average: T::default(),
            full: false,
        }
    }

    pub fn update(&mut self, new_entry: T) -> T {
        if self.index == N - 1 {
            self.full = true;
        }

        self.entries[self.index] = new_entry;
        self.index = (self.index + 1) % N;

        self.average = if self.full {
            self.entries.iter().fold(T::default(), |acc, &x| acc + x) / N
        } else {
            self.entries
                .iter()
                .take(self.index + 1)
                .fold(T::default(), |acc, &x| acc + x)
                / (self.index + 1)
        };
        self.average
    }

    pub fn average(&self) -> T {
        self.average
    }
}

#[derive(Clone, Copy, Default)]
struct UpdateRecord {
    pub time: Duration,
    pub diff: usize,
}

impl Add<UpdateRecord> for UpdateRecord {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            time: self.time + other.time,
            diff: self.diff + other.diff,
        }
    }
}

impl Div<usize> for UpdateRecord {
    type Output = Self;

    fn div(self, rhs: usize) -> Self {
        Self {
            time: self.time / rhs as u32,
            diff: self.diff / rhs,
        }
    }
}

pub struct ProgressTracker {
    min: isize,
    max: isize,
    current: isize,
    first_update: Instant,
    last_update: Instant,
    records: MovingAverage<UpdateRecord, 32>,
}

impl ProgressTracker {
    pub fn new(min: isize, max: isize) -> Self {
        Self {
            min,
            max,
            current: min,
            first_update: Instant::now(),
            last_update: Instant::now(),
            records: MovingAverage::new(),
        }
    }

    pub fn update(&mut self, new_current: isize) {
        let last = self.current;
        self.current = new_current;

        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;

        let diff = (self.current - last).max(0);
        self.records.update(UpdateRecord {
            time: delta,
            diff: diff as usize,
        });
    }

    pub fn progress(&self) -> f64 {
        (self.current - self.min) as f64 / (self.max - self.min) as f64 * 100.0
    }

    pub fn get_eta(&self) -> Duration {
        let UpdateRecord { time, diff } = self.records.average();
        let speed = diff as f64 / time.as_secs_f64();
        if speed == 0.0 {
            Duration::from_secs(0)
        } else {
            let remaining = (self.max - self.current) as f64;
            Duration::from_secs_f64(remaining / speed)
        }
    }

    pub fn get_elapsed(&self) -> Duration {
        self.first_update.elapsed()
    }

    pub fn max(&self) -> isize {
        self.max
    }
}

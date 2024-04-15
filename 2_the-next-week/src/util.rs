use num::Num;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;

pub fn get_random_canonical() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn get_random<T: Num + SampleUniform>(from: T, to: T) -> T {
    let mut rng = rand::thread_rng();
    let dist = Uniform::<T>::new(from, to);
    dist.sample(&mut rng)
}

pub fn random() {}

pub fn linear_to_gamma(linear: f64) -> f64 {
    linear.sqrt()
}

use num::Num;
use rand::Rng;

pub fn get_random_canonical() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn get_random<T: Num + Copy + Into<f64> + From<f64>>(from: T, to: T) -> T {
    let canonical = get_random_canonical();
    let range = to - from;
    from + T::from(canonical * range.into())
}

pub fn linear_to_gamma(linear: f64) -> f64 {
    linear.sqrt()
}

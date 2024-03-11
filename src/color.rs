#![allow(dead_code)]

use crate::vec::{VecElement, Vector};

macro_rules! gen_getter {
    ($name:ident, $index:literal) => {
        pub fn $name(&self) -> T {
            self.0.data[$index]
        }
    };
}

pub struct Color<T: VecElement>(Vector<T, 3>);

impl<T: VecElement> Color<T> {
    pub fn new(data: [T; 3]) -> Self {
        Self(Vector::new(data))
    }

    gen_getter!(r, 0);
    gen_getter!(g, 1);
    gen_getter!(b, 2);

    gen_getter!(h, 0);
    gen_getter!(s, 1);
    gen_getter!(v, 2);
}

pub fn cast<T, U>(color: Color<T>) -> Color<U>
where
    T: VecElement,
    U: VecElement,
    T: Into<U>,
{
    let data = [color.r().into(), color.g().into(), color.b().into()];
    Color::new(data)
}

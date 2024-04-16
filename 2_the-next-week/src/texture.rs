use crate::color::Color;
use crate::vec::Vector;

type Vec2 = Vector<f64, 2>;
type Vec3 = Vector<f64, 3>;

pub trait Texture {
    fn value(&self, uv: Vec2, point: Vec3) -> Color;
}

// A solid color texture
pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Texture for SolidColor {
    fn value(&self, _uv: Vec2, _point: Vec3) -> Color {
        self.albedo.clone()
    }
}

// A checkerboard-colored texture (comprised of two textures)
pub struct CheckerTexture {
    inv_scale: f64,
    even_tex: Box<dyn Texture>,
    odd_tex: Box<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even_tex: Box<dyn Texture>, odd_tex: Box<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even_tex,
            odd_tex,
        }
    }

    pub fn from_color(scale: f64, even_color: Color, odd_color: Color) -> Self {
        Self::new(
            scale,
            Box::new(SolidColor::new(even_color)),
            Box::new(SolidColor::new(odd_color)),
        )
    }
}

impl Texture for CheckerTexture {
    fn value(&self, uv: Vec2, point: Vec3) -> Color {
        let value: i32 = point
            .data
            .iter()
            .map(|v| (v * self.inv_scale).floor() as i32)
            .sum();

        match value % 2 {
            0 => self.even_tex.value(uv, point),
            _ => self.odd_tex.value(uv, point),
        }
    }
}

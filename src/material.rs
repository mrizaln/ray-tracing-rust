use crate::color::Color;
use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::util;
use crate::vec::{self, Vector};

type Ray3 = Ray<f64, 3>;
type Vec3 = Vector<f64, 3>;

pub struct ScatterResult {
    pub ray: Ray3,
    pub attenuation: Color,
}

pub trait Material {
    fn scatter(&self, ray: Ray3, hit_record: HitRecord) -> Option<ScatterResult>;
}

// diffuse material
pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _ray: Ray3, hit_record: HitRecord) -> Option<ScatterResult> {
        let mut scatter_direction: Vec3 = hit_record.normal + vec::random_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = hit_record.normal;
        }

        Some(ScatterResult {
            ray: Ray {
                origin: hit_record.point,
                direction: scatter_direction,
            },
            attenuation: self.albedo.clone(),
        })
    }
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

// shiny material
pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, ray: Ray3, hit_record: HitRecord) -> Option<ScatterResult> {
        let reflected = vec::reflect(ray.direction.unit_vector(), hit_record.normal)
            + vec::random_in_unit_sphere() * self.fuzz;

        if reflected.dot(hit_record.normal) <= 0.0 {
            None
        } else {
            Some(ScatterResult {
                ray: Ray {
                    origin: hit_record.point,
                    direction: reflected,
                },
                attenuation: self.albedo.clone(),
            })
        }
    }
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

// glassy material
pub struct Dielectric {
    pub refractive_index: f64,
}

impl Dielectric {
    pub fn new(refractive_index: f64) -> Self {
        Self { refractive_index }
    }

    fn reflectance(cosine: f64, refractive_index: f64) -> f64 {
        // Schlick approx.
        let mut r0 = (1.0 - refractive_index) / (1.0 + refractive_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: Ray3, hit_record: HitRecord) -> Option<ScatterResult> {
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };
        let unit_direction = ray.direction.unit_vector();

        // total internal reflection
        let cos_theta = (-unit_direction).dot(hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0
            || Self::reflectance(cos_theta, refraction_ratio) > util::get_random_canonical();

        let scatter = if cannot_refract {
            vec::reflect(unit_direction, hit_record.normal)
        } else {
            vec::refract(unit_direction, hit_record.normal, refraction_ratio)
        };

        Some(ScatterResult {
            ray: Ray {
                origin: hit_record.point,
                direction: scatter,
            },
            attenuation: Color::new_one(1.0),
        })
    }
}

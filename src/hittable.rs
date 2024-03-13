use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec::Vector;

type Vec3 = Vector<f64, 3>;
type Ray3 = Ray<f64, 3>;

#[derive(Clone)]
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t_value: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(ray: Ray3, out_normal: Vec3, point: Vec3, t_value: f64) -> HitRecord {
        let front_face = ray.direction.dot(out_normal) < 0.0;
        let normal = if front_face { out_normal } else { -out_normal };

        Self {
            point,
            normal,
            t_value,
            front_face,
        }
    }
}

pub struct HitResult<'a> {
    pub record: HitRecord,
    pub material: Option<&'a dyn Material>,
}

pub trait Hittable {
    fn hit(&self, ray: Ray3, t_range: Interval) -> Option<HitResult>;
    fn get_material<'a>(&'a self) -> Option<&'a dyn Material> {
        None
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Option<Box<dyn Material>>,
}

impl Hittable for Sphere {
    fn get_material<'a>(&'a self) -> Option<&'a dyn Material> {
        self.material.as_deref()
    }

    fn hit(&self, ray: Ray3, t_range: Interval) -> Option<HitResult> {
        // basically quadratic formula
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let b_half = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let d = b_half * b_half - a * c;
        if d < 0.0 {
            return None;
        }

        let d_sqrt = d.sqrt();
        let root1 = (-b_half - d_sqrt) / a;
        let root2 = (-b_half + d_sqrt) / a;
        let root = match (t_range.surrounds(root1), t_range.surrounds(root2)) {
            (false, false) => return None,
            (false, true) => root2,
            (true, _) => root1,
        };

        let point = ray.at(root);
        let out_normal = (point - self.center) / self.radius;

        Some(HitResult {
            record: HitRecord::new(ray, out_normal, point, root),
            material: self.get_material(),
        })
    }
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: Option<Box<dyn Material>>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

unsafe impl Sync for HittableList {}

impl Hittable for HittableList {
    fn hit(&self, ray: Ray3, t_range: Interval) -> Option<HitResult> {
        let mut current_hit = None;
        let mut t_closest = t_range.max;

        for object in self.objects.iter() {
            if let Some(hit) = object.hit(ray.clone(), (t_range.min, t_closest).into()) {
                t_closest = hit.record.t_value;
                current_hit = Some(hit);
            }
        }

        current_hit
    }
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

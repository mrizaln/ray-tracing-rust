use crate::aabb::AABB;
use crate::hittable::{HitResult, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;

type AABB3 = AABB<f64, 3>;
type Ray3 = Ray<f64, 3>;

pub enum BvhNodeElement {
    Leaf(Box<dyn Hittable>),
    Node(Box<BvhNode>),
}

// Bounding Volume Hierarchy (basically a binary tree)
// TODO: A node can only exist if it has at least one child. One of the node can be not an Option.
pub struct BvhNode {
    left: Option<BvhNodeElement>,
    right: Option<BvhNodeElement>,
    bbox: AABB3,
}

impl BvhNode {
    pub fn new(objects: Vec<Box<dyn Hittable>>) -> BvhNode {
        Self::split(objects)
    }

    // TODO: Find a more concrete way to subdivide the objects (read some papers or articles)
    fn split(mut objects: Vec<Box<dyn Hittable>>) -> BvhNode {
        let mut bbox = AABB3::empty();
        objects.iter().for_each(|o| {
            bbox.combine(&o.bounding_box());
        });

        if objects.len() == 1 {
            return Self {
                left: Some(BvhNodeElement::Leaf(objects.pop().unwrap())),
                right: None,
                bbox,
            };
        }
        if objects.len() == 2 {
            return Self {
                left: Some(BvhNodeElement::Leaf(objects.pop().unwrap())),
                right: Some(BvhNodeElement::Leaf(objects.pop().unwrap())),
                bbox,
            };
        }

        let axis = bbox.longest_axis();
        objects.sort_by(move |a: &Box<dyn Hittable>, b: &Box<dyn Hittable>| {
            let a_interval = a.bounding_box().axis_interval(axis);
            let b_interval = b.bounding_box().axis_interval(axis);
            a_interval.min.partial_cmp(&b_interval.min).unwrap()
        });

        let mid = objects.len() / 2;
        let left = Self::split(objects.drain(..mid).collect::<Vec<_>>());
        let right = Self::split(objects);

        Self {
            left: Some(BvhNodeElement::Node(Box::new(left))),
            right: Some(BvhNodeElement::Node(Box::new(right))),
            bbox,
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: Ray3, t_range: Interval) -> Option<HitResult> {
        if !self.bbox.hit(ray.clone(), t_range.clone()) {
            return None;
        }

        let left_hit = self.left.as_ref().and_then(|n| match n {
            BvhNodeElement::Leaf(h) => h.hit(ray.clone(), t_range.clone()),
            BvhNodeElement::Node(n) => n.hit(ray.clone(), t_range.clone()),
        });

        let t_max = left_hit
            .as_ref()
            .map(|r| r.record.t_value)
            .unwrap_or(t_range.max);
        let new_t_range = Interval::new(t_range.min, t_max);

        let right_hit = self.right.as_ref().and_then(|n| match n {
            BvhNodeElement::Leaf(h) => h.hit(ray.clone(), new_t_range.clone()),
            BvhNodeElement::Node(n) => n.hit(ray.clone(), new_t_range.clone()),
        });

        match (left_hit, right_hit) {
            (None, None) => None,
            (None, Some(r)) => Some(r),
            (Some(l), None) => Some(l),
            (Some(l), Some(r)) => {
                if l.record.t_value < r.record.t_value {
                    Some(l)
                } else {
                    Some(r)
                }
            }
        }
    }

    fn bounding_box(&self) -> &AABB3 {
        &self.bbox
    }

    fn get_material<'a>(&'a self) -> Option<&'a dyn Material> {
        None
    }
}

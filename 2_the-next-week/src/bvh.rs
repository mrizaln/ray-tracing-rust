use crate::aabb::AABB;
use crate::hittable::{HitResult, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util;

type AABB3 = AABB<f64, 3>;
type Ray3 = Ray<f64, 3>;

// Bounding Volume Hierarchy (basically a binary tree)
// TODO: A node can only exist if it has at least one child. One of the node can be not an Option.
pub struct BvhNode {
    left: Option<Box<dyn Hittable>>,
    right: Option<Box<dyn Hittable>>,
    bbox: AABB3,
}

impl BvhNode {
    // NOTE: implementation cater to simplicity rather than correctness
    //       - randomly choose an axis
    //       - sort the primitives
    //       - put half in each subtree
    pub fn new(mut objects: Vec<Box<dyn Hittable>>) -> Self {
        let axis = util::get_random(0, 3);
        objects.sort_by(move |a: &Box<dyn Hittable>, b: &Box<dyn Hittable>| {
            let a_interval = a.bounding_box().axis_interval(axis);
            let b_interval = b.bounding_box().axis_interval(axis);
            a_interval.min.partial_cmp(&b_interval.min).unwrap()
        });

        let (left, right) = Self::split(objects);
        let bbox = left
            .as_ref()
            .map(|h| h.bounding_box().clone())
            .unwrap_or(AABB3::empty())
            .combine_new(
                &right
                    .as_ref()
                    .map(|h| h.bounding_box().clone())
                    .unwrap_or(AABB3::empty()),
            );
        Self { left, right, bbox }
    }

    fn split(
        mut objects: Vec<Box<dyn Hittable>>,
    ) -> (Option<Box<dyn Hittable>>, Option<Box<dyn Hittable>>) {
        if objects.len() == 1 {
            return (objects.pop(), None);
        }
        if objects.len() == 2 {
            return (objects.pop(), objects.pop());
        }

        let mid = objects.len() / 2;
        let left_objects = Self::split(objects.drain(..mid).collect::<Vec<_>>());
        let right_objects = Self::split(objects.drain(..).collect::<Vec<_>>());

        let combine_bbox =
            |(ref a, ref b): &(Option<Box<dyn Hittable>>, Option<Box<dyn Hittable>>)| {
                let a = a
                    .as_ref()
                    .map(|h| h.bounding_box().clone())
                    .unwrap_or(AABB3::empty());
                let b = b
                    .as_ref()
                    .map(|h| h.bounding_box().clone())
                    .unwrap_or(AABB3::empty());
                a.combine_new(&b)
            };

        let bbox = combine_bbox(&left_objects);
        let left = Box::new(Self {
            left: left_objects.0,
            right: left_objects.1,
            bbox,
        });

        let bbox = combine_bbox(&right_objects);
        let right = Box::new(Self {
            left: right_objects.0,
            right: right_objects.1,
            bbox,
        });

        (Some(left), Some(right))
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: Ray3, t_range: Interval) -> Option<HitResult> {
        if !self.bbox.hit(ray.clone(), t_range.clone()) {
            return None;
        }

        let left_hit = self
            .left
            .as_ref()
            .and_then(|h| h.hit(ray.clone(), t_range.clone()));

        let t_max = left_hit
            .as_ref()
            .map(|r| r.record.t_value)
            .unwrap_or(t_range.max);
        let new_t_range = Interval::new(t_range.min, t_max);

        let right_hit = self
            .right
            .as_ref()
            .and_then(|h| h.hit(ray.clone(), new_t_range));

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

    fn get_material<'a>(&'a self) -> Option<&'a dyn crate::Material> {
        None
    }
}

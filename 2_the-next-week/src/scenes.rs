use crate::bvh::BvhNode;
use crate::color::Color;
use crate::hittable::{Hittable, HittableList, Sphere};
use crate::material::{Dielectric, Lambertian, Material, Metal};
use crate::texture::CheckerTexture;
use crate::vec::Vector;
use crate::{util, vec};

pub fn ray_tracing_in_one_week_book_scene() -> HittableList {
    let mut scene = HittableList::new();

    // ground
    scene.add(Box::new(Sphere::new(
        Vector::new([0.0, -1000.0, 0.0]),
        1000.0,
        Some(Box::new(Lambertian::new(Color::new([0.5, 0.5, 0.5])))), // diffuse
    )));

    // small spheres
    for a in -11..11 {
        for b in -11..11 {
            let center = Vector::new([
                a as f64 + 0.9 * util::get_random_canonical(),
                0.2,
                b as f64 + 0.9 * util::get_random_canonical(),
            ]);
            let offset = Vector::new([4.0, 0.2, 0.0]);

            if (center - offset).length() <= 0.9 {
                break;
            }

            let choose_material = util::get_random_canonical();
            let material: Box<dyn Material> = if choose_material < 0.8 {
                let albedo = vec::random_vector(0.0, 1.0) * vec::random_vector(0.0, 1.0);
                Box::new(Lambertian::new(Color::from(albedo)))
            } else if choose_material < 0.95 {
                let albedo = vec::random_vector(0.5, 1.0);
                let fuzz = util::get_random(0.0, 0.5);
                Box::new(Metal::new(Color::from(albedo), fuzz))
            } else {
                Box::new(Dielectric::new(1.5))
            };

            let sphere = Box::new(Sphere::new(center, 0.2, Some(material)));
            scene.add(sphere);
        }
    }

    // large spheres
    scene.add(Box::new(Sphere::new(
        Vector::new([0.0, 1.0, 0.0]),
        1.0,
        Some(Box::new(Dielectric::new(1.5))), // glassy
    )));

    scene.add(Box::new(Sphere::new(
        Vector::new([-4.0, 1.0, 0.0]),
        1.0,
        Some(Box::new(Lambertian::new(Color::new([0.4, 0.2, 0.1])))), // diffuse
    )));

    scene.add(Box::new(Sphere::new(
        Vector::new([4.0, 1.0, 0.0]),
        1.0,
        Some(Box::new(Metal::new(Color::new([0.7, 0.6, 0.5]), 0.0))), // shiny
    )));

    scene
}

fn ray_tracing_in_one_week_book_scene_modified() -> Vec<Box<dyn Hittable>> {
    let mut objects = Vec::<Box<dyn Hittable>>::new();

    let checker = Box::new(CheckerTexture::from_color(
        0.32,
        Color::new([0.2, 0.3, 0.1]),
        Color::new([0.9, 0.9, 0.9]),
    ));

    // ground (static)
    objects.push(Box::new(Sphere::new(
        Vector::new([0.0, -1000.0, 0.0]),
        1000.0,
        Some(Box::new(Lambertian::with_texture(checker))), // diffuse
    )));

    // small spheres (moving)
    for a in -11..11 {
        for b in -11..11 {
            let center = Vector::new([
                a as f64 + 0.9 * util::get_random_canonical(),
                0.2,
                b as f64 + 0.9 * util::get_random_canonical(),
            ]);
            let offset = Vector::new([4.0, 0.2, 0.0]);

            if (center - offset).length() <= 0.9 {
                break;
            }

            let choose_material = util::get_random_canonical();

            type M = Box<dyn Material>;
            let (material, is_moving) = if choose_material < 0.8 {
                let albedo = vec::random_vector(0.0, 1.0) * vec::random_vector(0.0, 1.0);
                (Box::new(Lambertian::new(Color::from(albedo))) as M, true)
            } else if choose_material < 0.95 {
                let albedo = vec::random_vector(0.5, 1.0);
                let fuzz = util::get_random(0.0, 0.5);
                (Box::new(Metal::new(Color::from(albedo), fuzz)) as M, false)
            } else {
                (Box::new(Dielectric::new(1.5)) as M, false)
            };

            let sphere = if is_moving {
                let center2 = center + Vector::new([0.0, util::get_random(0.0, 0.5), 0.0]);
                Box::new(Sphere::new_moving(center, center2, 0.2, Some(material)))
            } else {
                Box::new(Sphere::new(center, 0.2, Some(material)))
            };
            objects.push(sphere);
        }
    }

    // large spheres (static)
    objects.push(Box::new(Sphere::new(
        Vector::new([0.0, 1.0, 0.0]),
        1.0,
        Some(Box::new(Dielectric::new(1.5))), // glassy
    )));

    objects.push(Box::new(Sphere::new(
        Vector::new([-4.0, 1.0, 0.0]),
        1.0,
        Some(Box::new(Lambertian::new(Color::new([0.4, 0.2, 0.1])))), // diffuse
    )));

    objects.push(Box::new(Sphere::new(
        Vector::new([4.0, 1.0, 0.0]),
        1.0,
        Some(Box::new(Metal::new(Color::new([0.7, 0.6, 0.5]), 0.0))), // shiny
    )));

    objects
}

pub fn ray_tracing_in_one_week_book_scene_modified_simple() -> HittableList {
    let mut list = HittableList::new();
    ray_tracing_in_one_week_book_scene_modified()
        .into_iter()
        .for_each(|o| list.add(o));
    list
}

pub fn ray_tracing_in_one_week_book_scene_modified_bvh() -> HittableList {
    let mut list = HittableList::new();
    let objects = ray_tracing_in_one_week_book_scene_modified();
    list.add(Box::new(BvhNode::new(objects)));
    list
}

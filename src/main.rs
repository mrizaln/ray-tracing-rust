use core::panic;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::interval::Interval;
use crate::ray_tracer::Image;

use self::color::Color;
use self::hittable::{HittableList, Sphere};
use self::material::{Dielectric, Lambertian, Material, Metal};
use self::ray_tracer::{RayTracer, TracerParams};
use self::vec::Vector;

mod color;
mod hittable;
mod interval;
mod material;
mod ray;
mod ray_tracer;
mod util;
mod vec;

fn generate_ppm_image(image: Image, path: &Path) {
    if path.exists() && path.is_dir() {
        panic!("File exists and is a directory! Aborting");
    }

    if path.exists() {
        println!(
            "File {} exists. Will overwrite.",
            path.to_str().unwrap_or("{unknown}")
        );
    }

    let mut file = File::create(path).expect(
        format!(
            "Failed to open file {}",
            path.to_str().unwrap_or("{unknwon}")
        )
        .as_str(),
    );
    let Image { pixels, dimension } = image;

    const MAX_COLOR: i32 = 255;
    let mut temp = format!(
        "P3\n{} {}\n{}\n",
        dimension.width, dimension.height, MAX_COLOR
    );

    for (i, pixel) in pixels.iter().enumerate() {
        let corrected = color::correct_gamma(*pixel);
        let clamped = color::clamp(corrected, Interval::new(0.0, 0.999));
        let color = color::transform(clamped, |v| (MAX_COLOR as f64 * v) as i32);
        // let color = color::transform(*pixel, |v| (MAX_COLOR as f64 * v) as i32);

        let line = format!("{} {} {}\n", color.r(), color.g(), color.b());
        temp += &line;

        if (i + 1) % dimension.width as usize == 0 {
            file.write_all(temp.as_bytes())
                .expect("Failed to write to file");
            temp.clear();
        }
    }
}

fn create_scene() -> HittableList {
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

fn main() {
    // println!("Hello, world!");

    let args = std::env::args().collect::<Vec<_>>();
    let filename = if args.len() >= 2 {
        args[1].clone()
    } else {
        "image.ppm".into()
    };

    let ray_tracer = RayTracer::new(TracerParams {
        aspect_ratio: 16.0 / 9.0,
        height: 720,
        sampling_rate: 25,
        max_depth: 10,
        vfov: 20.0,
        defocus_angle: 0.6,
        focus_distance: 10.0,
        look_from: Vector::new([13.0, 2.0, 3.0]),
        look_at: Vector::new([0.0, 0.0, 0.0]),
    });
    let scene = create_scene();
    let image = ray_tracer.render(&scene);

    let path = Path::new(filename.as_str());
    generate_ppm_image(image, path)
}

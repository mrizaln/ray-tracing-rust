use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod color;
pub mod hittable;
pub mod interval;
pub mod material;
pub mod ray;
pub mod ray_tracer;
pub mod util;
pub mod vec;

use color::Color;
use hittable::{HittableList, Sphere};
use material::*;
use ray_tracer::Image;
use vec::Vector;

pub fn generate_ppm_image(image: Image, path: &Path) {
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
        let color: Color<i32> = pixel
            .correct_gamma()
            .clamp((0.0, 0.999).into())
            .transform(|v| (v * MAX_COLOR as f64) as i32);

        let line = format!("{} {} {}\n", color.r(), color.g(), color.b());
        temp += &line;

        if (i + 1) % dimension.width as usize == 0 {
            file.write_all(temp.as_bytes())
                .expect("Failed to write to file");
            temp.clear();
        }
    }
}

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

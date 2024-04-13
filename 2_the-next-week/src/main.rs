use std::io::Read;
use std::path::Path;
use std::time::Instant;

use ray_tracing_the_next_week as rtr;
use rtr::ray_tracer::{RayTracer, TracerParams};
use rtr::vec::Vector;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let filepath = if args.len() >= 2 {
        Path::new(args[1].as_str())
    } else {
        Path::new("image.ppm")
    };

    if filepath.exists() {
        eprintln!("File already exist! ({})", filepath.to_str().unwrap());
        eprint!("Proceed anyway [y/N]? ");

        let mut c = [b'\0'; 1];
        std::io::stdin().read(&mut c).unwrap();
        if c[0] != b'y' && c[0] != b'Y' {
            println!("Aborted.");
            return;
        }
    }

    let ray_tracer = RayTracer::new(TracerParams {
        aspect_ratio: 16.0 / 9.0,
        height: 720,
        sampling_rate: 50,
        max_depth: 10,
        vfov: 20.0,
        defocus_angle: 0.6,
        focus_distance: 10.0,
        look_from: Vector::new([13.0, 2.0, 3.0]),
        look_at: Vector::new([0.0, 0.0, 0.0]),
    });

    // let scene = rtr::ray_tracing_in_one_week_book_scene();
    let scene = rtr::ray_tracing_in_one_week_book_scene_but_moving();

    let now = Instant::now();
    let image = ray_tracer.render_multi(&scene);
    let duration = now.elapsed();

    println!("Rendering took {:.2} seconds", duration.as_secs_f64());

    rtr::generate_ppm_image(image, filepath)
}

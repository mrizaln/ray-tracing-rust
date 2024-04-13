use std::io::Read;
use std::time::Instant;

use ray_tracing_the_next_week as rtr;
use rtr::ray_tracer::RayTracer;

macro_rules! timeit {
    ($e:expr) => {{
        let now = Instant::now();
        let result = $e;
        let duration = now.elapsed();
        (result, duration)
    }};
}

fn main() {
    let (tracer_params, filepath) = rtr::parse_args();
    eprintln!("Parameters: {:#?}", tracer_params);

    if filepath.exists() {
        eprintln!("File already exist! ({})", filepath.display());
        eprint!("Proceed anyway [y/N]? ");

        let mut c = [b'\0'; 1];
        std::io::stdin().read(&mut c).unwrap();
        if c[0] != b'y' && c[0] != b'Y' {
            eprintln!("Aborted.");
            return;
        }
    }

    let ray_tracer = RayTracer::new(tracer_params);

    // let scene = rtr::ray_tracing_in_one_week_book_scene();
    let scene = rtr::ray_tracing_in_one_week_book_scene_but_moving();

    let (image, duration) = timeit! { ray_tracer.render_multi(&scene) };
    eprintln!("Rendering took {:.2} seconds", duration.as_secs_f64());

    rtr::generate_ppm_image(image, &filepath)
}

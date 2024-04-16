use std::io::Read;
use std::time::Instant;

use ray_tracing_the_next_week as rtr;
use rtr::ray_tracer::RayTracer;
use rtr::ParsedArgs;

macro_rules! timeit {
    ($e:expr) => {{
        let now = Instant::now();
        let result = $e;
        let duration = now.elapsed();
        (result, duration)
    }};
}

fn main() {
    let ParsedArgs {
        tracer_params,
        scene,
        output,
        use_single_thread,
        force_output,
    } = rtr::parse_args();
    eprintln!("\n{:#?}\n", tracer_params);

    if output.exists() {
        eprintln!("File already exist! ({})", output.display());

        if !force_output {
            eprint!("Proceed anyway [y/N]? ");

            let mut c = [b'\0'; 1];
            std::io::stdin().read(&mut c).unwrap();
            if c[0] != b'y' && c[0] != b'Y' {
                eprintln!("Aborted.");
                return;
            }
        } else {
            eprintln!("Force flag enabled. Proceeding anyway.");
        }
    }

    let ray_tracer = RayTracer::new(tracer_params);

    let (image, duration) = timeit!(match use_single_thread {
        true => ray_tracer.render(&scene),
        false => ray_tracer.render_multi(&scene),
    });
    eprintln!("Rendering took {:.2} seconds", duration.as_secs_f64());

    rtr::generate_ppm_image(image, &output)
}

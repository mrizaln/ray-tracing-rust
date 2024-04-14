use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::usize;

pub mod color;
pub mod hittable;
pub mod interval;
pub mod material;
pub mod progress_tracker;
pub mod ray;
pub mod ray_tracer;
pub mod util;
pub mod vec;

use clap::{arg, value_parser, Arg, ArgAction, Command};
use color::Color;
use config::Config;
use hittable::{HittableList, Sphere};
use material::*;
use ray_tracer::Image;
use vec::Vector;

use self::ray_tracer::TracerParams;
use self::vec::VecElement;

macro_rules! parse_config {
    ($config:expr, $matches:expr, $name:literal, $type:ty, $dest:expr) => {
        $config
            .get($name)
            .and_then(|v| v.parse::<$type>().ok())
            .map(|v| $dest = v);
        $matches.get_one::<$type>($name).map(|v| $dest = *v);
    };
}

macro_rules! parse_config_fn {
    ($config:expr, $matches:expr, $name:literal, $parsefn:expr, $dest:expr) => {
        $config
            .get($name)
            .and_then(|v| $parsefn(v))
            .map(|v| $dest = v);
        $matches
            .get_one::<String>($name)
            .and_then(|v| $parsefn(v.as_str()))
            .map(|v| $dest = v);
    };
}

pub struct ParsedArgs {
    pub tracer_params: TracerParams,
    pub output: PathBuf,
    pub use_single_thread: bool,
}

pub fn parse_args() -> ParsedArgs {
    let mut param = TracerParams::default();

    let matches = Command::new("RayTracer")
        .version("2.0")
        .about("A ray tracer")
        .arg(arg!([output] "Optional output file"))
        .arg(arg!(-g --config <config> "Config file"))
        .arg(arg!(-t --height <height> "Height").value_parser(value_parser!(u32)))
        .arg(arg!(-s --sampling <sampling> "Sampling rate").value_parser(value_parser!(u32)))
        .arg(arg!(-d --depth <depth> "Max depth").value_parser(value_parser!(u32)))
        .arg(arg!(-v --vfov <vfov> "Vertical FOV").value_parser(value_parser!(f64)))
        .arg(arg!(-a --angle <angle> "Defocus angle").value_parser(value_parser!(f64)))
        .arg(arg!(-c --focus <focus> "Focus distance").value_parser(value_parser!(f64)))
        .arg(arg!(-f --look_from <look_from> "Look from vector (FMT: \"f64/f64/f64\")")) // custom
        .arg(arg!(-l --look_at <look_at> "Look at vector (FMT: \"f64/f64/f64\")")) // custom
        .arg(
            Arg::new("single-thread")
                .short('1')
                .long("single-thread")
                .help("Use single thread for rendering instead of multi-thread")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let config_file = if let Some(config) = matches.get_one::<String>("config") {
        config.as_str()
    } else {
        "renderconfig.toml"
    };

    let config = Config::builder()
        .add_source(config::File::with_name(config_file))
        .build()
        .and_then(|c| c.try_deserialize::<HashMap<String, String>>())
        .unwrap_or_else(|e| {
            eprintln!("Failed to read config file: {}", e);
            HashMap::new()
        });

    parse_config!(config, matches, "height", u32, param.height);
    parse_config!(config, matches, "sampling", u32, param.sampling_rate);
    parse_config!(config, matches, "depth", u32, param.max_depth);
    parse_config!(config, matches, "vfov", f64, param.vfov);
    parse_config!(config, matches, "angle", f64, param.defocus_angle);
    parse_config!(config, matches, "focus", f64, param.focus_distance);
    parse_config_fn!(config, matches, "look_from", parse_vector, param.look_from);
    parse_config_fn!(config, matches, "look_at", parse_vector, param.look_at);

    let output = matches
        .get_one::<String>("output")
        .map(|s| s.as_str())
        .unwrap_or("image.ppm");

    let use_single_thread = matches.get_flag("single-thread");

    ParsedArgs {
        tracer_params: param,
        output: output.into(),
        use_single_thread,
    }
}

pub fn generate_ppm_image(image: Image, path: &Path) {
    if path.exists() && path.is_dir() {
        panic!("File exists and is a directory! Aborting");
    }

    if path.exists() {
        eprintln!(
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

pub fn ray_tracing_in_one_week_book_scene_but_moving() -> HittableList {
    let mut scene = HittableList::new();

    // ground (static)
    scene.add(Box::new(Sphere::new(
        Vector::new([0.0, -1000.0, 0.0]),
        1000.0,
        Some(Box::new(Lambertian::new(Color::new([0.5, 0.5, 0.5])))), // diffuse
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
            scene.add(sphere);
        }
    }

    // large spheres (static)
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

fn parse_vector<T, const N: usize>(string: &str) -> Option<Vector<T, N>>
where
    T: VecElement + std::str::FromStr + Debug,
{
    let result = string
        .split('/')
        .map(|e| e.parse::<T>().ok())
        .collect::<Option<Vec<T>>>()
        .and_then(|v| {
            let boxed_slice = v.into_boxed_slice();
            match TryInto::<Box<[T; N]>>::try_into(boxed_slice) {
                Ok(array) => Some(Vector::new(*array)),
                Err(_) => None,
            }
        });
    result
}

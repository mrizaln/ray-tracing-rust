use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::usize;

pub mod aabb;
pub mod bvh;
pub mod color;
pub mod hittable;
pub mod interval;
pub mod material;
pub mod progress_tracker;
pub mod ray;
pub mod ray_tracer;
pub mod scenes;
pub mod texture;
pub mod util;
pub mod vec;

use clap::{arg, value_parser, Arg, ArgAction, Command};
use color::Color;
use config::Config;
use rand::seq::SliceRandom;
use ray_tracer::Image;
use vec::Vector;

use self::hittable::HittableList;
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
    pub scene: HittableList,
    pub output: PathBuf,
    pub use_single_thread: bool,
    pub force_output: bool,
}

pub fn parse_args() -> ParsedArgs {
    let mut param = TracerParams::default();

    let scene_list = scenes::SCENES
        .iter()
        .map(|(k, _)| *k)
        .collect::<Vec<_>>()
        .join("\n\t- ");

    let matches = Command::new("RayTracer")
        .version("2.0")
        .about("A ray tracer")
        .arg(arg!([output] "Optional output file (default: 'image.ppm')"))
        .arg(arg!(-g --config <FILE> "Config file (default: 'renderconfig.toml')"))
        .arg(arg!(-t --height <INT> "Height").value_parser(value_parser!(u32)))
        .arg(arg!(-s --sampling <INT> "Sampling rate").value_parser(value_parser!(u32)))
        .arg(arg!(-d --depth <INT> "Max depth").value_parser(value_parser!(u32)))
        .arg(arg!(-v --vfov <FLOAT> "Vertical FOV").value_parser(value_parser!(f64)))
        .arg(arg!(-a --angle <FLOAT> "Defocus angle").value_parser(value_parser!(f64)))
        .arg(arg!(-c --focus <FLOAT> "Focus distance").value_parser(value_parser!(f64)))
        .arg(arg!(-f --look_from <FMT> "Look from vector (FMT: \"FLOAT/FLOAT/FLOAT\")"))
        .arg(arg!(-l --look_at <FMT> "Look at vector (FMT: \"FLOAT/FLOAT/FLOAT\")"))
        .arg(
            Arg::new("single-thread")
                .short('1')
                .long("single-thread")
                .help("Use single thread for rendering instead of multi-thread")
                .action(ArgAction::SetTrue),
        )
        .arg(arg!(--force "Overwrite output if exists"))
        .arg(
            Arg::new("scene")
                .short('i')
                .long("scene")
                .help(
                    "Choose scenes from these options:\n\t- ".to_string()
                        + &scene_list
                        + "\nIf you omit this option, the scene will be randomly selected",
                )
                .action(ArgAction::Set),
        )
        .get_matches();

    let config_file = if let Some(config) = matches.get_one::<String>("config") {
        eprintln!("Using config file: '{}'", config);
        config.as_str()
    } else {
        eprintln!("No config file specified. Using default config file: 'renderconfig.toml'");
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
    let force_output = matches.get_flag("force");

    let get_random_scene = || {
        *scenes::SCENES
            .keys()
            .collect::<Vec<_>>()
            .choose(&mut rand::thread_rng())
            .unwrap()
    };

    let scene_name = match matches.get_one::<String>("scene") {
        Some(s) => match scenes::SCENES.get(s.as_str()) {
            Some(_) => {
                eprintln!("Using scene: '{}'", s);
                s.as_str()
            }
            None => {
                let name = get_random_scene();
                eprintln!("Scene '{}' not found. Random select scene: '{}'", s, name);
                name
            }
        },
        None => {
            let name = get_random_scene();
            eprintln!("Scene not specified. Randomly selecting scene: '{}'", name);
            eprintln!("If you don't want this behavior specify scene with --scene (see --help)");
            name
        }
    };

    ParsedArgs {
        tracer_params: param,
        output: output.into(),
        use_single_thread,
        force_output,
        scene: scenes::SCENES[scene_name](),
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

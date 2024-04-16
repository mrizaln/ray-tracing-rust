use std::num::NonZeroUsize;
use std::thread;

use crate::color::Color;
use crate::hittable::{HitResult, Hittable};
use crate::interval::Interval;
use crate::material::ScatterResult;
use crate::progress_tracker::ProgressTracker;
use crate::ray::Ray;
use crate::vec::Vector;
use crate::{util, vec};

type Vec3 = Vector<f64, 3>;
type Ray3 = Ray<f64, 3>;

#[derive(Clone, Debug)]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Image {
    pub pixels: Vec<Color>,
    pub dimension: Dimension,
}

#[derive(Debug)]
pub struct TracerParams {
    pub aspect_ratio: f64,
    pub height: u32,
    pub sampling_rate: u32,
    pub max_depth: u32,
    pub vfov: f64,
    pub defocus_angle: f64,
    pub focus_distance: f64,
    pub look_from: Vec3,
    pub look_at: Vec3,
}

#[derive(Debug)]
struct Viewport {
    pub width: f64,
    pub height: f64,
    pub u_vector: Vec3,
    pub v_vector: Vec3,
    pub du_vector: Vec3,
    pub dv_vector: Vec3,
    pub upper_left: Vec3,
    pub pixel_origin: Vec3,
}

#[derive(Debug)]
struct Camera {
    pub position: Vec3,
    pub view_up: Vec3,
    pub view_right: Vec3,
    pub view_dir: Vec3,
    pub defocus_disk_u_vec: Vec3,
    pub defocus_disk_v_vec: Vec3,
    pub vfov: f64,
    pub defocus_angle: f64,
    pub focus_distance: f64,
}

#[derive(Debug)]
pub struct RayTracer {
    dimension: Dimension,
    viewport: Viewport,
    camera: Camera,
    sampling_rate: u32,
    max_depth: u32,
}

impl RayTracer {
    pub fn new(params: TracerParams) -> Self {
        let world_up = Vec3::new([0.0, 1.0, 0.0]);

        let cam_center = params.look_from;

        let view_dir = (cam_center - params.look_at).unit_vector();
        let view_right = world_up.cross(view_dir).unit_vector();
        let view_up = view_dir.cross(view_right);

        let theta = params.vfov.to_radians();
        let h = (theta / 2.0).tan();

        let height = params.height;
        let width = (height as f64 * params.aspect_ratio) as u32;

        let actual_ratio = width as f64 / height as f64;
        let view_height = 2.0 * h * params.focus_distance;
        let view_width = view_height as f64 * actual_ratio;

        let view_u_vec = view_right * view_width;
        let view_v_vec = -view_up * view_height;
        let view_du_vec = view_u_vec / width as f64;
        let view_dv_vec = view_v_vec / height as f64;

        let view_upper_left =
            cam_center - (view_dir * params.focus_distance) - view_u_vec / 2.0 - view_v_vec / 2.0;
        let view_pixel_origin = view_upper_left + (view_du_vec + view_dv_vec) * 0.5;

        let defocus_radius =
            params.focus_distance * (params.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u_vec = view_right * defocus_radius;
        let defocus_disk_v_vec = view_up * defocus_radius;

        // construct
        let dimension = Dimension { width, height };

        let viewport = Viewport {
            width: view_width,
            height: view_height,
            u_vector: view_u_vec,
            v_vector: view_v_vec,
            du_vector: view_du_vec,
            dv_vector: view_dv_vec,
            upper_left: view_upper_left,
            pixel_origin: view_pixel_origin,
        };

        let camera = Camera {
            position: cam_center,
            view_up,
            view_right,
            view_dir,
            defocus_disk_u_vec,
            defocus_disk_v_vec,
            vfov: params.vfov,
            defocus_angle: params.defocus_angle,
            focus_distance: params.focus_distance,
        };

        Self {
            dimension,
            viewport,
            camera,
            sampling_rate: params.sampling_rate,
            max_depth: params.max_depth,
        }
    }

    pub fn render(&self, scene: &dyn Hittable) -> Image {
        let mut pixels = Vec::<Color>::with_capacity(
            self.dimension.width as usize * self.dimension.height as usize,
        );

        let Dimension { width, height } = self.dimension;
        let mut tracker = ProgressTrackerWrapper::new(width, height as usize);

        for row in 0..height {
            for col in 0..width {
                let color = self
                    .sample_color_at(col, row, scene)
                    .clamp((0.0, 1.0).into());
                pixels.push(color);

                tracker.update(row as usize, (col + 1) as usize);
            }
        }

        Image {
            pixels,
            dimension: self.dimension.clone(),
        }
    }

    pub fn render_multi(&self, scene: &(dyn Hittable + Sync)) -> Image {
        let concurrency_level: usize = thread::available_parallelism()
            .unwrap_or(NonZeroUsize::new(1).unwrap())
            .get();
        let chunk_size = self.dimension.height as usize / concurrency_level;

        enum SampleResult {
            Color(usize, Color),
            None,
        }

        let (tx, rx) = std::sync::mpsc::channel::<SampleResult>();

        // interleaved rendering
        thread::scope(|s| {
            for i in 0..concurrency_level.into() {
                let num_steps = match chunk_size * concurrency_level + i {
                    x if x < self.dimension.height as usize => chunk_size + 1,
                    _ => chunk_size,
                };
                let tx = tx.clone();

                s.spawn(move || {
                    let mut tracker = match i {
                        0 => Some(ProgressTrackerWrapper::new(self.dimension.width, num_steps)),
                        _ => None,
                    };

                    for count in 0..num_steps {
                        let row = (count as usize * concurrency_level + i) as u32;
                        for col in 0..self.dimension.width {
                            let index = (row * self.dimension.width + col) as usize;
                            let color = self
                                .sample_color_at(col, row, scene)
                                .clamp(Interval::new(0.0, 1.0));

                            tx.send(SampleResult::Color(index, color)).unwrap();

                            tracker
                                .as_mut()
                                .map(|v| v.update(count, (col + 1) as usize));
                        }
                    }
                    tx.send(SampleResult::None).unwrap();
                });
            }
        });

        let pixel_num = self.dimension.width as usize * self.dimension.height as usize;
        let mut pixels = vec![Color::new([0.0, 0.0, 0.0]); pixel_num];

        let mut completed_threads = 0usize;
        while completed_threads < concurrency_level {
            match rx.recv().unwrap() {
                SampleResult::Color(index, color) => pixels[index] = color,
                SampleResult::None => completed_threads += 1,
            }
        }

        Image {
            pixels,
            dimension: self.dimension.clone(),
        }
    }

    fn sample_color_at(&self, col: u32, row: u32, hittable: &dyn Hittable) -> Color {
        let mut accumulated_color = Color::new_one(0.0);

        let pixel_center = self.viewport.pixel_origin
            + (self.viewport.du_vector * col as f64)
            + (self.viewport.dv_vector * row as f64);

        for _ in 0..self.sampling_rate {
            let pixel_sample = pixel_center + self.sample_unit_square();
            let ray_origin = match self.camera.defocus_angle {
                x if x <= 0.0 => self.camera.position,
                _ => self.defocus_disk_sample(),
            };
            let ray_direction = pixel_sample - ray_origin;
            let ray_time = util::get_random_canonical();

            let ray = Ray3 {
                origin: ray_origin,
                direction: ray_direction.unit_vector(),
                time: ray_time,
            };

            accumulated_color = accumulated_color + self.ray_color(ray, self.max_depth, hittable);
        }

        accumulated_color / self.sampling_rate as f64
    }

    fn ray_color(&self, ray: Ray3, depth: u32, hittable: &dyn Hittable) -> Color {
        if depth <= 0 {
            return Color::new_one(0.0);
        }

        match hittable.hit(ray.clone(), Interval::new(0.001, f64::INFINITY)) {
            Some(HitResult { record, material }) => {
                let normal = record.normal.clone();
                match material.and_then(|v| v.scatter(ray, record)) {
                    Some(ScatterResult {
                        ray: new_ray,
                        attenuation,
                    }) => attenuation * self.ray_color(new_ray, depth - 1, hittable),
                    None => Color::from(normal * 0.5 + 0.5),
                }
            }
            None => {
                // missed, use background color instead
                let direction = ray.direction.unit_vector();

                // lerp
                let a = 0.5 * (direction.data[1] + 1.0);
                let white = Color::new_one(1.0);
                let blue = Color::new([0.5, 0.7, 1.0]);

                white * (1.0 - a) + blue * a
            }
        }
    }

    fn sample_unit_square(&self) -> Vec3 {
        let px = util::get_random(0.0, 1.0) - 0.5;
        let py = util::get_random(0.0, 1.0) - 0.5;
        self.viewport.du_vector * px + self.viewport.dv_vector * py
    }

    fn defocus_disk_sample(&self) -> Vec3 {
        let [x, y] = vec::random_in_unit_disk::<f64>().data;
        self.camera.position
            + self.camera.defocus_disk_u_vec * x
            + self.camera.defocus_disk_v_vec * y
    }
}

impl Default for TracerParams {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            height: 480,
            sampling_rate: 20,
            max_depth: 10,
            vfov: 20.0,
            defocus_angle: 0.6,
            focus_distance: 10.0,
            look_from: Vector::new([13.0, 2.0, 3.0]),
            look_at: Vector::new([0.0, 0.0, 0.0]),
        }
    }
}

struct ProgressTrackerWrapper {
    tracker: ProgressTracker,
    min_update_interval: usize,
    width: usize,
}

impl ProgressTrackerWrapper {
    pub fn new(width: u32, steps: usize) -> Self {
        const MINIMUM_UPDATE_INTERVAL: usize = 512;
        let max_count = steps * width as usize;
        Self {
            tracker: ProgressTracker::new(0, max_count as isize),
            // min_update_interval: MINIMUM_UPDATE_INTERVAL.min(width as usize),
            min_update_interval: MINIMUM_UPDATE_INTERVAL,
            width: width as usize,
        }
    }

    pub fn update(&mut self, count: usize, width_step: usize) {
        let new_count = count * self.width + width_step;
        let should_update = new_count % self.min_update_interval == 0;
        let reached_max = new_count == self.tracker.max() as usize;
        if should_update || reached_max {
            self.tracker.update(new_count as isize);
            eprint!(
                "Progress: {:>6.2}% | Elapsed: {:>6.2}s | ETA: {:>6.2}s\r",
                self.tracker.progress(),
                self.tracker.get_elapsed().as_secs_f64(),
                self.tracker.get_eta().as_secs_f64()
            );
            if reached_max {
                eprintln!();
            }
        }
    }
}

use crate::color::Color;
use crate::vec::Vector;

type Vec3 = Vector<f64, 3>;
type Color3 = Color<f64>;

pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

pub struct Image {
    pub pixels: Vec<Color3>,
    pub dimension: Dimension,
}

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

struct Viewport {
    pub dimension: Dimension,
    pub u_vector: Vec3,
    pub v_vector: Vec3,
    pub du_vector: Vec3,
    pub dv_vector: Vec3,
    pub upper_left: Vec3,
    pub pixel_origin: Vec3,
}

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

        let view_dir = (cam_center.clone() - params.look_at).unit_vector();
        let view_right = world_up.cross(view_dir.clone()).unit_vector();
        let view_up = view_dir.cross(view_right.clone());

        let theta = params.vfov.to_radians();
        let h = theta / 2.0;

        let height = params.height;
        let width = (height as f64 * params.aspect_ratio) as u32;

        let actual_ratio = width as f64 / height as f64;
        let view_height = (2.0 * h * params.focus_distance) as u32;
        let view_width = (view_height as f64 * actual_ratio) as u32;

        let view_u_vec = view_right.clone() * view_width as f64;
        let view_v_vec = -view_up.clone() * view_height as f64;
        let view_du_vec = view_u_vec.clone() / width as f64;
        let view_dv_vec = view_v_vec.clone() / height as f64;

        let view_upper_left = cam_center.clone()
            - (view_dir.clone() * params.focus_distance)
            - view_u_vec.clone() / 2.0
            - view_v_vec.clone() / 2.0;
        let view_pixel_origin =
            view_upper_left.clone() + (view_du_vec.clone() + view_dv_vec.clone()) * 0.5;

        let defocus_radius =
            params.focus_distance * (params.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u_vec = view_right.clone() * defocus_radius;
        let defocus_disk_v_vec = view_up.clone() * defocus_radius;

        // construct
        let dimension = Dimension { width, height };

        let view_dimension = Dimension {
            width: view_width,
            height: view_height,
        };

        let viewport = Viewport {
            dimension: view_dimension,
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
}

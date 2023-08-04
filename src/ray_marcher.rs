use gl_matrix::common::{Vec3, to_radian};
use gl_matrix::vec3;

pub type Sdf = fn(Vec3) -> f32;

pub struct RayMarcher {
    camera: Vec3,
    look_at: Vec3,
    up: Vec3,
    fov_y: f32,
    aspect_ratio: f32,
    half_screen_length_y: f32, // asuming half_screen_length_x = 1
    // Orthonormal basis of the camera system
    u: Vec3, // pointing to the right
    v: Vec3, // pointing up
    w: Vec3, // pointing towards the scene
}

impl RayMarcher {
    pub fn new(
        camera: &Vec3,
        look_at: &Vec3,
        up: &Vec3,
        fov_y_degrees: f32,
        aspect_ratio: f32
    ) -> RayMarcher {
        let fov_y = to_radian(fov_y_degrees);
        let half_screen_length_y = (0.5 * fov_y).tan();
        let mut u = vec3::create();
        let mut v = vec3::create();
        let mut w = vec3::create();
        vec3::normalize(&mut w, &vec3::subtract(&mut vec3::create(), look_at, camera)); // w = normalize(lookAt - camera)
        vec3::normalize(&mut v, &vec3::scale_and_add(&mut vec3::create(), up, &w, -vec3::dot(up, &w))); // v = normalize(up - dot(up, w) * w)
        vec3::cross(&mut u, &w, &v); // u = cross(w, v)

        RayMarcher {
            camera: *camera,
            look_at: *look_at,
            up: *up,
            fov_y,
            aspect_ratio,
            half_screen_length_y,
            u,
            v,
            w,
        }
    }
}

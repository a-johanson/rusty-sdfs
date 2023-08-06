use gl_matrix::common::Vec3;
use gl_matrix::vec3;

use crate::sdf::{sd_plane, sd_sphere};

pub fn scene(p: &Vec3) -> f32 {
    sd_sphere(p, 1.0).min(sd_plane(p, &vec3::from_values(0.0, 1.0, 0.0), -1.0))
}

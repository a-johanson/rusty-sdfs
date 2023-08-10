use crate::vector::{Vec3, vec3};

pub type Sdf = fn(&Vec3) -> f32;

pub fn sd_plane(p: &Vec3, normal: &Vec3, offset: f32) -> f32 {
    vec3::dot(p, normal) - offset
}

pub fn sd_sphere(p: &Vec3, radius: f32) -> f32 {
    vec3::len(p) - radius
}

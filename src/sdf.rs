use crate::vector::{vec3, Vec3, VecFloat};

pub type Sdf = fn(&Vec3) -> VecFloat;

pub fn op_shift(p: &Vec3, offset: &Vec3) -> Vec3 {
    vec3::sub(p, offset)
}

pub fn op_elongate_y(p: &Vec3, length: VecFloat) -> Vec3 {
    let qy = (p.1.abs() - length).max(0.0);
    vec3::from_values(p.0, qy, p.2)
}

pub fn sd_plane(p: &Vec3, normal: &Vec3, offset: VecFloat) -> VecFloat {
    vec3::dot(p, normal) - offset
}

pub fn sd_sphere(p: &Vec3, radius: VecFloat) -> VecFloat {
    vec3::len(p) - radius
}

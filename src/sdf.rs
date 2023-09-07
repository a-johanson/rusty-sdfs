use crate::vector::{vec3, Vec3, VecFloat};

pub type Sdf = fn(&Vec3) -> VecFloat;

pub fn op_shift(p: &Vec3, offset: &Vec3) -> Vec3 {
    vec3::sub(p, offset)
}

pub fn op_elongate_y(p: &Vec3, length: VecFloat) -> Vec3 {
    let qy = (p.1.abs() - length).max(0.0);
    vec3::from_values(p.0, qy, p.2)
}

pub fn op_elongate_z(p: &Vec3, length: VecFloat) -> Vec3 {
    let qz = (p.2.abs() - length).max(0.0);
    vec3::from_values(p.0, p.1, qz)
}

pub fn op_rotate_y(p: &Vec3, angle: VecFloat) -> Vec3 {
    let cos_angle = (-angle).cos();
    let sin_angle = (-angle).sin();
    vec3::from_values(
        cos_angle * p.0 + sin_angle * p.2,
        p.1,
        -sin_angle * p.0 + cos_angle * p.2,
    )
}

pub fn op_rotate_z(p: &Vec3, angle: VecFloat) -> Vec3 {
    let cos_angle = (-angle).cos();
    let sin_angle = (-angle).sin();
    vec3::from_values(
        cos_angle * p.0 + sin_angle * p.1,
        -sin_angle * p.0 + cos_angle * p.1,
        p.2,
    )
}

pub fn op_repeat_finite(p: &Vec3, diameter: &Vec3, repeat_from: &Vec3, repeat_to: &Vec3) -> Vec3 {
    vec3::from_values(
        p.0 - diameter.0 * (p.0 / diameter.0).round().clamp(repeat_from.0, repeat_to.0),
        p.1 - diameter.1 * (p.1 / diameter.1).round().clamp(repeat_from.1, repeat_to.1),
        p.2 - diameter.2 * (p.2 / diameter.2).round().clamp(repeat_from.2, repeat_to.2),
    ) // = p - s * clamp(round(p/s), lim_a, lim_b)
}

pub fn sd_plane(p: &Vec3, normal: &Vec3, offset: VecFloat) -> VecFloat {
    vec3::dot(p, normal) - offset
}

pub fn sd_sphere(p: &Vec3, radius: VecFloat) -> VecFloat {
    vec3::len(p) - radius
}

pub fn sd_box(p: &Vec3, sides: &Vec3) -> VecFloat {
    let q = vec3::from_values(
        p.0.abs() - sides.0,
        p.1.abs() - sides.1,
        p.2.abs() - sides.2,
    ); // q = abs(p) - s
    vec3::len(&vec3::max_float(&q, 0.0)) + q.0.max(q.1).max(q.2).min(0.0) // = length(max(q, 0)) + min(max(q.x, q.y, q.z), 0);
}

pub fn sd_cylinder(p: &Vec3, radius: VecFloat, height: VecFloat) -> VecFloat {
    let len_xz = (p.0 * p.0 + p.2 * p.2).sqrt();
    let d_xz = len_xz - radius;
    let d_y = p.1.abs() - height;
    let d_xz_clamp = d_xz.max(0.0);
    let d_y_clamp = d_y.max(0.0);
    let len_d_clamp = (d_xz_clamp * d_xz_clamp + d_y_clamp * d_y_clamp).sqrt();
    d_xz.max(d_y).min(0.0) + len_d_clamp
}

pub fn sd_cylinder_rounded(
    p: &Vec3,
    radius: VecFloat,
    height: VecFloat,
    offset: VecFloat,
) -> VecFloat {
    sd_cylinder(p, radius - offset, height - offset) - offset
}

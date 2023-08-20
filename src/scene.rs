use crate::vector::{Vec3, vec3, to_radian};

use crate::sdf::{op_shift, op_elongate_y, sd_plane, sd_sphere};

pub fn scene(p: &Vec3) -> f32 {
    let base = sd_plane(p, &vec3::from_values(0.0, 1.0, 0.0), 0.0);
    let bg_tilt = to_radian(-30.0);
    let background = sd_plane(p, &vec3::from_values(bg_tilt.sin(), 0.0, bg_tilt.cos()), -8.0);

    let capsule1 = sd_sphere(
        &op_elongate_y(
            &op_shift(p, &vec3::from_values(1.0, 0.0, 0.25)),
            0.45),
        1.0);
    let capsule2 = sd_sphere(
        &op_elongate_y(
            &op_shift(p, &vec3::from_values(-1.5, 0.0, 0.0)),
            0.6),
        0.9);
    let capsule3 = sd_sphere(
        &op_elongate_y(
            &op_shift(p, &vec3::from_values(-0.2, 0.0, -2.0)),
            1.5),
        0.8);
    let capsule4 = sd_sphere(
        &op_elongate_y(
            &op_shift(p, &vec3::from_values(2.0, 0.0, -2.0)),
            2.1),
        0.8);

    base.min(background).min(capsule1).min(capsule2).min(capsule3).min(capsule4)
}

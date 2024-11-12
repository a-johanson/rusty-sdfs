use crate::vector::{vec2, vec3, vec4, Vec2, Vec3, Vec4, VecFloat};

#[derive(Clone, Copy)]
pub struct ReflectiveProperties {
    pub ambient_weight: VecFloat,
    pub ao_weight: VecFloat,
    pub visibility_weight: VecFloat,
    pub diffuse_weight: VecFloat,
    pub specular_weight: VecFloat,
    pub specular_exponent: VecFloat,
    pub ao_steps: u32,
    pub ao_step_size: VecFloat,
    pub penumbra: VecFloat,
}

impl ReflectiveProperties {
    pub fn new(
        ambient_weight: VecFloat,
        ao_weight: VecFloat,
        visibility_weight: VecFloat,
        diffuse_weight: VecFloat,
        specular_weight: VecFloat,
        specular_exponent: Option<VecFloat>,
        ao_steps: Option<u32>,
        ao_step_size: Option<VecFloat>,
        penumbra: Option<VecFloat>,
    ) -> ReflectiveProperties {
        ReflectiveProperties {
            ambient_weight,
            ao_weight,
            visibility_weight,
            diffuse_weight,
            specular_weight,
            specular_exponent: specular_exponent.unwrap_or(32.0),
            ao_steps: ao_steps.unwrap_or(5),
            ao_step_size: ao_step_size.unwrap_or(0.01),
            penumbra: penumbra.unwrap_or(48.0),
        }
    }

    pub fn default() -> ReflectiveProperties {
        Self::new(0.1, 0.1, 0.0, 0.8, 1.0, None, None, None, None)
    }

    pub fn lerp(&self, other: &ReflectiveProperties, t: VecFloat) -> ReflectiveProperties {
        fn float_lerp(a: VecFloat, b: VecFloat, t: VecFloat) -> VecFloat {
            a + (b - a) * t
        }
        ReflectiveProperties {
            ambient_weight: float_lerp(self.ambient_weight, other.ambient_weight, t),
            ao_weight: float_lerp(self.ao_weight, other.ao_weight, t),
            visibility_weight: float_lerp(self.visibility_weight, other.visibility_weight, t),
            diffuse_weight: float_lerp(self.diffuse_weight, other.diffuse_weight, t),
            specular_weight: float_lerp(self.specular_weight, other.specular_weight, t),
            specular_exponent: float_lerp(self.specular_exponent, other.specular_exponent, t),
            ao_steps: float_lerp(self.ao_steps as VecFloat, other.ao_steps as VecFloat, t).round()
                as u32,
            ao_step_size: float_lerp(self.ao_step_size, other.ao_step_size, t),
            penumbra: float_lerp(self.penumbra, other.penumbra, t),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Material {
    pub light_source: Vec3,
    pub reflective_properties: ReflectiveProperties,
    pub bg_hsl: Vec3,
    pub is_shaded: bool,
    pub is_hatched: bool,
}

impl Material {
    pub fn new(
        light_source: &Vec3,
        reflective_properties: Option<&ReflectiveProperties>,
        bg_hsl: Option<&Vec3>,
        is_shaded: bool,
        is_hatched: bool,
    ) -> Material {
        Material {
            light_source: *light_source,
            reflective_properties: *reflective_properties
                .unwrap_or(&ReflectiveProperties::default()),
            bg_hsl: *bg_hsl.unwrap_or(&vec3::from_values(0.0, 0.0, 1.0)),
            is_shaded,
            is_hatched,
        }
    }

    pub fn lerp(&self, other: &Material, t: VecFloat) -> Material {
        Material {
            light_source: vec3::lerp(&self.light_source, &other.light_source, t),
            reflective_properties: self
                .reflective_properties
                .lerp(&other.reflective_properties, t),
            bg_hsl: vec3::lerp_hsl(&self.bg_hsl, &other.bg_hsl, t),
            is_shaded: if t < 0.5 {
                self.is_shaded
            } else {
                other.is_shaded
            },
            is_hatched: if t < 0.5 {
                self.is_hatched
            } else {
                other.is_hatched
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct SdfOutput {
    pub distance: VecFloat,
    pub material: Material,
}

impl SdfOutput {
    pub fn new(distance: VecFloat, material: Material) -> SdfOutput {
        SdfOutput {
            distance,
            material: material,
        }
    }

    pub fn min(&self, other: &SdfOutput) -> SdfOutput {
        if self.distance < other.distance {
            *self
        } else {
            *other
        }
    }
}

pub mod sdf_op {
    use super::*;

    pub fn op_onion(d: VecFloat, thickness: VecFloat) -> VecFloat {
        d.abs() - thickness
    }

    // See https://iquilezles.org/articles/smin/
    pub fn op_smooth_union(
        dist1: VecFloat,
        dist2: VecFloat,
        smoothing_width: VecFloat,
    ) -> (VecFloat, VecFloat) {
        let h = (smoothing_width - (dist1 - dist2).abs()).max(0.0) / smoothing_width;
        let mixing = 0.5 * h * h * h;
        let smoothing = (1.0 / 3.0) * mixing * smoothing_width;
        if dist1 < dist2 {
            (dist1 - smoothing, mixing)
        } else {
            (dist2 - smoothing, 1.0 - mixing)
        }
    }

    pub fn op_smooth_difference(
        dist1: VecFloat,
        dist2: VecFloat,
        smoothing_width: VecFloat,
    ) -> (VecFloat, VecFloat) {
        let h = (smoothing_width - (dist1 + dist2).abs()).max(0.0) / smoothing_width;
        let mixing = 0.5 * h * h * h;
        let smoothing = (1.0 / 3.0) * mixing * smoothing_width;
        if dist1 > -dist2 {
            (dist1 + smoothing, mixing)
        } else {
            (-dist2 + smoothing, 1.0 - mixing)
        }
    }

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

    pub fn op_rotate_quaternion(p: &Vec3, q: &Vec4) -> Vec3 {
        vec4::apply_quaternion_rotation(q, p)
    }

    pub fn op_repeat_xz<F>(sdf: F, p: &Vec3, cell_size: &Vec2) -> SdfOutput
    where
        F: Fn(&Vec3, &Vec2) -> SdfOutput,
    {
        // See https://iquilezles.org/articles/sdfrepetition/
        let p_xz = vec2::from_values(p.0, p.2);
        let cell_id = vec2::round_inplace(vec2::div(&p_xz, cell_size));
        let local_p = vec2::sub(&p_xz, &vec2::mul(&cell_id, cell_size));
        let neighbor_dir = vec2::sign(&local_p);
        [
            vec2::from_values(cell_id.0, cell_id.1 + neighbor_dir.1),
            vec2::from_values(cell_id.0 + neighbor_dir.0, cell_id.1),
            vec2::from_values(cell_id.0 + neighbor_dir.0, cell_id.1 + neighbor_dir.1),
        ]
        .iter()
        .fold(
            sdf(&vec3::from_values(local_p.0, p.1, local_p.1), &cell_id),
            |prev_output, id| {
                let local_p = vec2::sub(&p_xz, &vec2::mul(id, cell_size));
                sdf(&vec3::from_values(local_p.0, p.1, local_p.1), id).min(&prev_output)
            },
        )
    }

    pub fn op_repeat<F>(sdf: F, p: &Vec3, cell_size: &Vec3) -> SdfOutput
    where
        F: Fn(&Vec3, &Vec3) -> SdfOutput,
    {
        // See https://iquilezles.org/articles/sdfrepetition/
        let cell_id = vec3::round_inplace(vec3::div(p, cell_size));
        let local_p = vec3::sub(p, &vec3::mul(&cell_id, cell_size));
        let neighbor_dir = vec3::sign(&local_p);
        [
            vec3::from_values(cell_id.0, cell_id.1, cell_id.2 + neighbor_dir.2),
            vec3::from_values(cell_id.0, cell_id.1 + neighbor_dir.1, cell_id.2),
            vec3::from_values(
                cell_id.0,
                cell_id.1 + neighbor_dir.1,
                cell_id.2 + neighbor_dir.2,
            ),
            vec3::from_values(cell_id.0 + neighbor_dir.0, cell_id.1, cell_id.2),
            vec3::from_values(
                cell_id.0 + neighbor_dir.0,
                cell_id.1,
                cell_id.2 + neighbor_dir.2,
            ),
            vec3::from_values(
                cell_id.0 + neighbor_dir.0,
                cell_id.1 + neighbor_dir.1,
                cell_id.2,
            ),
            vec3::from_values(
                cell_id.0 + neighbor_dir.0,
                cell_id.1 + neighbor_dir.1,
                cell_id.2 + neighbor_dir.2,
            ),
        ]
        .iter()
        .fold(sdf(&local_p, &cell_id), |prev_output, id| {
            let local_p = vec3::sub(p, &vec3::mul(id, cell_size));
            sdf(&local_p, id).min(&prev_output)
        })
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

    pub fn sd_triangle(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> VecFloat {
        // Assume ABC enumerate the vertices of the triangle in a counter-clockwise fashion.
        // Extrude a prism from the triangle ABC.
        // If P is within the volume of the prism, the distance to the triangle is the distance of P to the plane spanned by ABC.
        // Otherwise, the distance to the triangle is the smallest distance of the distances from P to the line segments AB, BC, CA.

        // How to check whether P is inside of the prism:
        // Compute n = normalize(AB x BC), the normal pointing upwards.
        // Then, we want to compute normals n_AB, n_BC, n_CA for the 3 planes perpendicular to n and containing each of the lines segments AB, BC, CA
        // These normals should point to the inside of the prism so n_AB = n x AB etc.
        // Let Q be any point on one of the planes, then <QP, n_plane> is the signed distance from P to the plane.
        // Hence, P is inside of the prism iff sign <AP, n_AB> + sign <BP, n_BC> + sign <CP, n_CA> = 3
        // In this case, the distance of P to the triangle ABC is |<AP, n>|

        // In case P is outside the volume of the prism, we need to determine the distance of P to the three line segments AB, BC, CA.
        // Let's look at the line segment AB first and let's first find the distance from P to the line through A and B.
        // We find this distance by projecting AP onto AB: Q = <AP, AB> / |AB|^2 * AB
        // Then, the distance we are looking for is the distance from Q to (P-A) = AP.
        // If we clamp the factor <AP, AB> / |AB|^2 between [0, 1], we ensure that Q is on the line segment AB. Hence, the distance from Q to AP is the distance to the line segment.
        // To save on sqrt() operations, we compute the square of this distance for each line segment, find the minimum of the squared distance among all line segments and only then take the sqrt() to compute the final answer.
        // This is possible because sqrt() is monotonic.

        let ab = vec3::sub(&b, &a);
        let bc = vec3::sub(&c, &b);
        let ca = vec3::sub(&a, &c);

        let n = vec3::normalize_inplace(vec3::cross(&ab, &bc));
        let n_ab = vec3::normalize_inplace(vec3::cross(&n, &ab));
        let n_bc = vec3::normalize_inplace(vec3::cross(&n, &bc));
        let n_ca = vec3::normalize_inplace(vec3::cross(&n, &ca));

        let ap = vec3::sub(&p, &a);
        let bp = vec3::sub(&p, &b);
        let cp = vec3::sub(&p, &c);

        let is_inside_prism = vec3::dot(&ap, &n_ab) >= 0.0
            && vec3::dot(&bp, &n_bc) >= 0.0
            && vec3::dot(&cp, &n_ca) >= 0.0;

        if is_inside_prism {
            let distance_to_plane = vec3::dot(&ap, &n).abs();
            distance_to_plane
        } else {
            let q_ab = vec3::scale(
                &ab,
                (vec3::dot(&ap, &ab) / vec3::len_squared(&ab)).clamp(0.0, 1.0),
            );
            let dist_squared_ab = vec3::len_squared(&vec3::sub(&ap, &q_ab));
            let q_bc = vec3::scale(
                &bc,
                (vec3::dot(&bp, &bc) / vec3::len_squared(&bc)).clamp(0.0, 1.0),
            );
            let dist_squared_bc = vec3::len_squared(&vec3::sub(&bp, &q_bc));
            let q_ca = vec3::scale(
                &ca,
                (vec3::dot(&cp, &ca) / vec3::len_squared(&ca)).clamp(0.0, 1.0),
            );
            let dist_squared_ca = vec3::len_squared(&vec3::sub(&cp, &q_ca));
            let distance_to_circumference = dist_squared_ab
                .min(dist_squared_bc)
                .min(dist_squared_ca)
                .sqrt();
            distance_to_circumference
        }
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

    pub fn sd_capped_cone(p: &Vec3, radius_bottom: VecFloat, radius_top: VecFloat, half_height: VecFloat) -> VecFloat {
        // The capped cone is rotationally symmetric around the y-axis. Hence, we can find the distance to the cone in the plane through the y-axis and p.
        let q = vec2::from_values(vec2::len(&vec2::from_values(p.0, p.2)), p.1);

        // Compute the distance to the mantle of the cone.
        // Let r1 = radius_bottom, r2 = radius_top, h = half_height, Q = q, A = (r1, -h) and B = (r2, h).
        // Find the distance of Q and the line through A = (r1, -h) and B = (r2, h) given by s(t) = A + t * AB = (r1, -h) + t * (r2 - r1, 2h).
        // Project AQ onto AB: proj_AB(AQ) = <AQ, AB> / |AB|^2 * AB
        // t = <AQ, AB> / <AB, AB> = <(q_x - r1, q_y + h), (r2 - r1, 2h)> / <(r2 - r1, 2h), (r2 - r1, 2h)>
        let aq = vec2::from_values(q.0 - radius_bottom, q.1 + half_height);
        let ab = vec2::from_values(radius_top - radius_bottom, 2.0 * half_height);
        // t must be in [0, 1] since the mantle of the cone only extends between A and B.
        let t_0 = (vec2::dot(&aq, &ab) / vec2::len_squared(&ab)).clamp(0.0, 1.0);
        // compute point on s(t_0)
        let s = vec2::scale_and_add(&vec2::from_values(radius_bottom, -half_height), &ab, t_0);
        // find the distance from s(t_0) to Q
        let sq = vec2::sub(&q, &s);
        let distance_mantle = vec2::len(&sq);

        // Compute the distance to the caps.
        // Which cap is closer?
        let closest_radius = if q.1 < 0.0 { radius_bottom } else { radius_top };
        let cap_to_q = vec2::from_values((q.0 - closest_radius).max(0.0), q.1.abs() - half_height);
        let distance_caps = vec2::len(&cap_to_q);

        // p is only inside the cone if it inside the mantle and between the caps.
        let sign = if sq.0 < 0.0 && cap_to_q.1 < 0.0 { -1.0 } else { 1.0 };

        sign * distance_mantle.min(distance_caps)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use assert_approx_eq::assert_approx_eq;

        #[test]
        fn test_sd_rectangle() {
            let a = vec3::from_values(1.0, 0.0, -1.0);
            let b = vec3::from_values(0.0, 0.0, 1.0);
            let c = vec3::from_values(-1.0, 0.0, -1.0);

            assert_approx_eq!(
                4.0,
                sd_triangle(&vec3::from_values(0.25, 4.0, 0.1), &a, &b, &c)
            );
            assert_approx_eq!(
                3.0,
                sd_triangle(&vec3::from_values(-0.25, -3.0, -0.1), &a, &b, &c)
            );
            assert_approx_eq!(
                0.25,
                sd_triangle(&vec3::from_values(1.25, 0.0, -1.0), &a, &b, &c)
            );
            assert_approx_eq!(
                0.5,
                sd_triangle(&vec3::from_values(0.1, 0.0, -1.5), &a, &b, &c)
            );
            assert_approx_eq!(
                0.5,
                sd_triangle(&vec3::from_values(0.0, 0.0, 1.5), &a, &b, &c)
            );
            assert_approx_eq!(
                2.0f32.sqrt(),
                sd_triangle(&vec3::from_values(0.0, 1.0, 2.0), &a, &b, &c)
            );
            assert_approx_eq!(
                0.25,
                sd_triangle(&vec3::from_values(-1.25, 0.0, -1.0), &a, &b, &c)
            );
        }
    }
}

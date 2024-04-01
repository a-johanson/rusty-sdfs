pub type VecFloat = f32;
pub const EPSILON: VecFloat = 1.0e-6;

pub type Vec2 = (VecFloat, VecFloat);
pub type Vec3 = (VecFloat, VecFloat, VecFloat);

pub mod vec2 {
    use super::*;

    pub fn from_values(x: VecFloat, y: VecFloat) -> Vec2 {
        (x, y)
    }

    pub fn scale(a: &Vec2, scale: VecFloat) -> Vec2 {
        (scale * a.0, scale * a.1)
    }

    pub fn dot(a: &Vec2, b: &Vec2) -> VecFloat {
        a.0 * b.0 + a.1 * b.1
    }

    pub fn len_squared(a: &Vec2) -> VecFloat {
        a.0 * a.0 + a.1 * a.1
    }

    pub fn len(a: &Vec2) -> VecFloat {
        len_squared(a).sqrt()
    }

    pub fn dist(a: &Vec2, b: &Vec2) -> VecFloat {
        let diff = sub(a, b);
        len(&diff)
    }

    pub fn scale_and_add(a: &Vec2, b: &Vec2, scale: VecFloat) -> Vec2 {
        (a.0 + scale * b.0, a.1 + scale * b.1)
    }

    pub fn sub(a: &Vec2, b: &Vec2) -> Vec2 {
        (a.0 - b.0, a.1 - b.1)
    }

    pub fn mul(a: &Vec2, b: &Vec2) -> Vec2 {
        (a.0 * b.0, a.1 * b.1)
    }

    pub fn div(a: &Vec2, b: &Vec2) -> Vec2 {
        (a.0 / b.0, a.1 / b.1)
    }

    pub fn sign(a: &Vec2) -> Vec2 {
        (a.0.signum(), a.1.signum())
    }

    pub fn round_inplace(a: Vec2) -> Vec2 {
        (a.0.round(), a.1.round())
    }

    pub fn polar_angle(a: &Vec2) -> VecFloat {
        a.1.atan2(a.0)
    }

    pub fn polar_angle_to_unit_vector(angle: VecFloat) -> Vec2 {
        (angle.cos(), angle.sin())
    }

    pub fn rotate_trig_inplace(mut a: Vec2, angle_cos: VecFloat, angle_sin: VecFloat) -> Vec2 {
        let x = angle_cos * a.0 - angle_sin * a.1;
        let y = angle_sin * a.0 + angle_cos * a.1;
        a.0 = x;
        a.1 = y;
        a
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use assert_approx_eq::assert_approx_eq;
        use std::f32::consts::PI;

        #[test]
        fn test_vec3_scale() {
            let a = from_values(1.0, -2.0);
            assert_eq!((-2.0, 4.0), scale(&a, -2.0));
        }

        #[test]
        fn test_vec2_dot() {
            let a = from_values(1.0, 2.0);
            let b = from_values(-3.0, 5.0);
            assert_eq!(7.0, dot(&a, &b));
        }

        #[test]
        fn test_vec2_len_squared() {
            let a = from_values(2.0, -3.0);
            assert_eq!(13.0, len_squared(&a));
        }

        #[test]
        fn test_vec2_len() {
            let a = from_values(2.0, -4.0);
            assert_approx_eq!(20.0f32.sqrt(), len(&a));
        }

        #[test]
        fn test_vec2_dist() {
            let a = from_values(1.0, 1.0);
            let b = from_values(0.0, 1.0);
            assert_eq!(1.0, dist(&a, &b));
        }

        #[test]
        fn test_vec2_scale_and_add() {
            let a = from_values(1.0, 2.0);
            let b = from_values(-3.0, 1.0);
            assert_eq!((7.0, 0.0), scale_and_add(&a, &b, -2.0));
        }

        #[test]
        fn test_vec2_sub() {
            let a = from_values(1.0, 2.0);
            let b = from_values(-3.0, 1.0);
            assert_eq!((4.0, 1.0), sub(&a, &b));
        }

        #[test]
        fn test_vec2_mul() {
            let a = from_values(2.0, 1.0);
            let b = from_values(-3.0, 0.0);
            assert_eq!((-6.0, 0.0), mul(&a, &b));
        }

        #[test]
        fn test_vec2_div() {
            let a = from_values(-4.0, -9.0);
            let b = from_values(2.0, -3.0);
            assert_eq!((-2.0, 3.0), div(&a, &b));
        }

        #[test]
        fn test_vec2_sign() {
            let a = from_values(-2.0, 1.0);
            assert_eq!((-1.0, 1.0), sign(&a));
            let a = from_values(0.0, -0.0);
            assert_eq!((1.0, -1.0), sign(&a));
        }

        #[test]
        fn test_vec2_round_inplace() {
            let a = from_values(-3.51, 3.49);
            assert_eq!((-4.0, 3.0), round_inplace(a));
        }

        #[test]
        fn test_vec2_polar_angle() {
            assert_eq!(0.0, polar_angle(&from_values(0.0, 0.0)));
            assert_eq!(0.0, polar_angle(&from_values(1.0, 0.0)));
            assert_approx_eq!(0.25 * PI, polar_angle(&from_values(1.0, 1.0)));
            assert_approx_eq!(0.5 * PI, polar_angle(&from_values(0.0, 1.0)));
            assert_approx_eq!(0.75 * PI, polar_angle(&from_values(-1.0, 1.0)));
            assert_approx_eq!(PI, polar_angle(&from_values(-1.0, 0.0)));
            assert_approx_eq!(-0.25 * PI, polar_angle(&from_values(1.0, -1.0)));
            assert_approx_eq!(-0.5 * PI, polar_angle(&from_values(0.0, -1.0)));
            assert_approx_eq!(-0.75 * PI, polar_angle(&from_values(-1.0, -1.0)));
        }

        #[test]
        fn test_vec2_rotate_trig_inplace() {
            let half: VecFloat = 0.5;
            let mut a = from_values(half.sqrt(), half.sqrt());
            let angle: VecFloat = 0.25 * PI;
            let angle_cos = angle.cos();
            let angle_sin = angle.sin();
            a = rotate_trig_inplace(a, 2.0 * angle_cos, 2.0 * angle_sin);
            assert_approx_eq!(0.0, a.0);
            assert_approx_eq!(2.0, a.1);
        }
    }
}

pub mod vec3 {
    use super::*;
    use std::f32::consts::PI;

    pub fn from_values(x: VecFloat, y: VecFloat, z: VecFloat) -> Vec3 {
        (x, y, z)
    }

    pub fn scale(a: &Vec3, scale: VecFloat) -> Vec3 {
        (scale * a.0, scale * a.1, scale * a.2)
    }

    pub fn scale_inplace(mut a: Vec3, scale: VecFloat) -> Vec3 {
        a.0 *= scale;
        a.1 *= scale;
        a.2 *= scale;
        a
    }

    pub fn scale_and_add(a: &Vec3, b: &Vec3, scale: VecFloat) -> Vec3 {
        (a.0 + scale * b.0, a.1 + scale * b.1, a.2 + scale * b.2)
    }

    pub fn scale_and_add_inplace(mut a: Vec3, b: &Vec3, scale: VecFloat) -> Vec3 {
        a.0 += scale * b.0;
        a.1 += scale * b.1;
        a.2 += scale * b.2;
        a
    }

    pub fn add(a: &Vec3, b: &Vec3) -> Vec3 {
        (a.0 + b.0, a.1 + b.1, a.2 + b.2)
    }

    pub fn sub(a: &Vec3, b: &Vec3) -> Vec3 {
        (a.0 - b.0, a.1 - b.1, a.2 - b.2)
    }

    pub fn mul(a: &Vec3, b: &Vec3) -> Vec3 {
        (a.0 * b.0, a.1 * b.1, a.2 * b.2)
    }

    pub fn div(a: &Vec3, b: &Vec3) -> Vec3 {
        (a.0 / b.0, a.1 / b.1, a.2 / b.2)
    }

    pub fn sign(a: &Vec3) -> Vec3 {
        (a.0.signum(), a.1.signum(), a.2.signum())
    }

    pub fn dot(a: &Vec3, b: &Vec3) -> VecFloat {
        a.0 * b.0 + a.1 * b.1 + a.2 * b.2
    }

    pub fn max_float(a: &Vec3, b: VecFloat) -> Vec3 {
        (a.0.max(b), a.1.max(b), a.2.max(b))
    }

    pub fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
        (
            a.1 * b.2 - a.2 * b.1,
            a.2 * b.0 - a.0 * b.2,
            a.0 * b.1 - a.1 * b.0,
        )
    }

    pub fn len_squared(a: &Vec3) -> VecFloat {
        a.0 * a.0 + a.1 * a.1 + a.2 * a.2
    }

    pub fn len(a: &Vec3) -> VecFloat {
        len_squared(a).sqrt()
    }

    pub fn normalize(a: &Vec3) -> Vec3 {
        let len_sq = len_squared(a);
        let scale = if len_sq > 0.0 {
            1.0 / len_sq.sqrt()
        } else {
            0.0
        };
        (scale * a.0, scale * a.1, scale * a.2)
    }

    pub fn normalize_inplace(mut a: Vec3) -> Vec3 {
        let len_sq = len_squared(&a);
        let scale = if len_sq > 0.0 {
            1.0 / len_sq.sqrt()
        } else {
            0.0
        };
        a.0 *= scale;
        a.1 *= scale;
        a.2 *= scale;
        a
    }

    pub fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
        scale_and_add(incident, normal, -2.0 * dot(incident, normal))
    }

    pub fn lerp(a: &Vec3, b: &Vec3, t: VecFloat) -> Vec3 {
        (
            a.0 + t * (b.0 - a.0),
            a.1 + t * (b.1 - a.1),
            a.2 + t * (b.2 - a.2),
        )
    }

    pub fn round_inplace(a: Vec3) -> Vec3 {
        (a.0.round(), a.1.round(), a.2.round())
    }

    pub fn orthonormal_basis_of_plane(
        normal: &Vec3,
        primary_direction: &Vec3,
    ) -> Option<(Vec3, Vec3)> {
        let normal_component = dot(primary_direction, normal);
        let u = scale_and_add(primary_direction, normal, -normal_component);
        let u_len = len(&u);
        if u_len <= EPSILON {
            return None;
        }
        let u = scale_inplace(u, 1.0 / u_len);

        let v = cross(&u, normal);
        Some((u, v))
    }

    pub fn hsl_to_rgb(hsl: &Vec3) -> Vec3 {
        let hue = hsl.0;
        let saturation = hsl.1;
        let lightness = hsl.2;

        let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let hue_bucket = hue / (60.0 * PI / 180.0);
        let bucket_position = chroma * (1.0 - (hue_bucket % 2.0 - 1.0).abs());
        let (r1, g1, b1) = match hue_bucket {
            b if b < 1.0 => (chroma, bucket_position, 0.0),
            b if b < 2.0 => (bucket_position, chroma, 0.0),
            b if b < 3.0 => (0.0, chroma, bucket_position),
            b if b < 4.0 => (0.0, bucket_position, chroma),
            b if b < 5.0 => (bucket_position, 0.0, chroma),
            _ => (chroma, 0.0, bucket_position),
        };
        let diff_lightness = lightness - 0.5 * chroma;
        (
            r1 + diff_lightness,
            g1 + diff_lightness,
            b1 + diff_lightness,
        )
    }

    pub fn lerp_hsl(hsl_a: &Vec3, hsl_b: &Vec3, t: VecFloat) -> Vec3 {
        let mut hue_delta = hsl_b.0 - hsl_a.0;
        if hue_delta > PI {
            hue_delta -= 2.0 * PI;
        } else if hue_delta < -PI {
            hue_delta += 2.0 * PI;
        }

        let mut hue = (hsl_a.0 + t * hue_delta) % (2.0 * PI);
        if hue < 0.0 {
            hue += 2.0 * PI;
        }

        (
            hue,
            hsl_a.1 + t * (hsl_b.1 - hsl_a.1),
            hsl_a.2 + t * (hsl_b.2 - hsl_a.2),
        )
    }

    pub fn hsl_to_rgb_u8(hsl: &Vec3) -> [u8; 3] {
        let (r, g, b) = hsl_to_rgb(hsl);
        [
            (r * 255.0).clamp(0.0, 255.0) as u8,
            (g * 255.0).clamp(0.0, 255.0) as u8,
            (b * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    pub fn hsl_to_rgba_u8(hsl: &Vec3) -> [u8; 4] {
        let [r, g, b] = hsl_to_rgb_u8(hsl);
        [r, g, b, 255]
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use assert_approx_eq::assert_approx_eq;

        #[test]
        fn test_vec3_scale() {
            let a = from_values(1.0, -2.0, 3.0);
            assert_eq!((-2.0, 4.0, -6.0), scale(&a, -2.0));
        }

        #[test]
        fn test_vec3_scale_inplace() {
            let a = from_values(1.0, -2.0, 3.0);
            assert_eq!((-2.0, 4.0, -6.0), scale_inplace(a, -2.0));
        }

        #[test]
        fn test_vec3_scale_and_add() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!((7.0, 0.0, 9.0), scale_and_add(&a, &b, -2.0));
        }

        #[test]
        fn test_vec3_scale_and_add_inplace() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!((7.0, 0.0, 9.0), scale_and_add_inplace(a, &b, -2.0));
        }

        #[test]
        fn test_vec3_add() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!((-2.0, 3.0, 0.0), add(&a, &b));
        }

        #[test]
        fn test_vec3_sub() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!((4.0, 1.0, 6.0), sub(&a, &b));
        }

        #[test]
        fn test_vec3_mul() {
            let a = from_values(1.0, 2.0, -3.0);
            let b = from_values(-3.0, 0.0, -3.0);
            assert_eq!((-3.0, 0.0, 9.0), mul(&a, &b));
        }

        #[test]
        fn test_vec3_div() {
            let a = from_values(-4.0, 0.0, -9.0);
            let b = from_values(2.0, 1.0, -3.0);
            assert_eq!((-2.0, 0.0, 3.0), div(&a, &b));
        }

        #[test]
        fn test_vec3_sign() {
            let a = from_values(-2.0, 1.0, -1.0);
            assert_eq!((-1.0, 1.0, -1.0), sign(&a));
            let a = from_values(0.0, -0.0, -0.0);
            assert_eq!((1.0, -1.0, -1.0), sign(&a));
        }

        #[test]
        fn test_vec3_dot() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!(-10.0, dot(&a, &b));
        }

        #[test]
        fn test_vec3_max_float() {
            let a = from_values(-3.0, 2.0, 3.0);
            assert_eq!((-1.0, 2.0, 3.0), max_float(&a, -1.0));
            assert_eq!((6.0, 6.0, 6.0), max_float(&a, 6.0));
        }

        #[test]
        fn test_vec3_cross() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(4.0, 5.0, 6.0);
            assert_eq!((-3.0, 6.0, -3.0), cross(&a, &b));
        }

        #[test]
        fn test_vec3_len() {
            let a = from_values(1.0, -2.0, 3.0);
            assert_approx_eq!(3.74165738677394138558, len(&a));
        }

        #[test]
        fn test_vec3_normalize() {
            let a = normalize(&from_values(1.0, -2.0, 3.0));
            assert_approx_eq!(0.26726124191242438468, a.0);
            assert_approx_eq!(-0.53452248382484876937, a.1);
            assert_approx_eq!(0.80178372573727315405, a.2);

            let b = normalize(&from_values(0.0, 0.0, 0.0));
            assert_eq!((0.0, 0.0, 0.0), b);
        }

        #[test]
        fn test_vec3_normalize_inplace() {
            let a = normalize_inplace(from_values(1.0, -2.0, 3.0));
            assert_approx_eq!(0.26726124191242438468, a.0);
            assert_approx_eq!(-0.53452248382484876937, a.1);
            assert_approx_eq!(0.80178372573727315405, a.2);

            let b = normalize_inplace(from_values(0.0, 0.0, 0.0));
            assert_eq!((0.0, 0.0, 0.0), b);
        }

        #[test]
        fn test_vec3_reflect() {
            let incident = normalize_inplace(from_values(-1.0, -1.0, -1.0));
            let n = from_values(0.0, 1.0, 0.0);
            let r = reflect(&incident, &n);
            let expected = normalize_inplace(from_values(-1.0, 1.0, -1.0));
            assert_approx_eq!(expected.0, r.0);
            assert_approx_eq!(expected.1, r.1);
            assert_approx_eq!(expected.2, r.2);
        }

        #[test]
        fn test_vec3_round_inplace() {
            let a = from_values(-3.51, -2.1, 3.5);
            assert_eq!((-4.0, -2.0, 4.0), round_inplace(a));
        }

        #[test]
        fn test_vec3_orthonormal_basis_of_plane() {
            let n = from_values(0.0, 1.0, 0.0);
            let dir = from_values(1.0e10, 2.0e10, 1.0e10);
            let (u, v) = orthonormal_basis_of_plane(&n, &dir).unwrap();
            let sqrt_half = (0.5 as VecFloat).sqrt();
            assert_approx_eq!(sqrt_half, u.0);
            assert_approx_eq!(0.0, u.1);
            assert_approx_eq!(sqrt_half, u.2);
            assert_approx_eq!(-sqrt_half, v.0);
            assert_approx_eq!(0.0, v.1);
            assert_approx_eq!(sqrt_half, v.2);

            assert!(orthonormal_basis_of_plane(&n, &scale(&n, -2.0)).is_none());
        }
    }
}

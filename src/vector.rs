
pub type VecFloat = f32;
pub const EPSILON: VecFloat = 1.0e-6;

pub type Vec2 = (VecFloat, VecFloat);
pub type Vec3 = (VecFloat, VecFloat, VecFloat);

#[cfg(test)]
fn equals(a: VecFloat, b: VecFloat) -> bool {
    return (a - b).abs() <= EPSILON * a.abs().max(b.abs().max(1.0));
}

pub mod vec2 {
    use super::*;

    pub fn from_values(x: VecFloat, y: VecFloat) -> Vec2 {
        (x, y)
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
        (
            a.0 + scale * b.0,
            a.1 + scale * b.1
        )
    }

    pub fn sub(a: &Vec2, b: &Vec2) -> Vec2 {
        (
            a.0 - b.0,
            a.1 - b.1
        )
    }

    pub fn polar_angle(a: &Vec2) -> VecFloat {
        a.1.atan2(a.0)
    }

    pub fn polar_angle_to_unit_vector(angle: VecFloat) -> Vec2 {
        (
            angle.cos(),
            angle.sin()
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::f32::consts::PI;

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
            assert!(equals(20.0f32.sqrt(), len(&a)));
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
        fn test_vec2_polar_angle() {
            assert_eq!(0.0, polar_angle(&from_values(0.0, 0.0)));
            assert_eq!(0.0, polar_angle(&from_values(1.0, 0.0)));
            assert!(equals(0.25 * PI, polar_angle(&from_values(1.0, 1.0))));
            assert!(equals(0.5 * PI, polar_angle(&from_values(0.0, 1.0))));
            assert!(equals(0.75 * PI, polar_angle(&from_values(-1.0, 1.0))));
            assert!(equals(PI, polar_angle(&from_values(-1.0, 0.0))));
            assert!(equals(-0.25 * PI, polar_angle(&from_values(1.0, -1.0))));
            assert!(equals(-0.5 * PI, polar_angle(&from_values(0.0, -1.0))));
            assert!(equals(-0.75 * PI, polar_angle(&from_values(-1.0, -1.0))));
        }
    }
}

pub mod vec3 {
    use super::*;

    pub fn from_values(x: VecFloat, y: VecFloat, z: VecFloat) -> Vec3 {
        (x, y, z)
    }

    pub fn add(a: &Vec3, b: &Vec3) -> Vec3 {
        (
            a.0 + b.0,
            a.1 + b.1,
            a.2 + b.2
        )
    }

    pub fn scale(a: &Vec3, scale: VecFloat) -> Vec3 {
        (
            scale * a.0,
            scale * a.1,
            scale * a.2
        )
    }

    pub fn scale_inplace(mut a: Vec3, scale:VecFloat) -> Vec3 {
        a.0 *= scale;
        a.1 *= scale;
        a.2 *= scale;
        a
    }

    pub fn scale_and_add(a: &Vec3, b: &Vec3, scale: VecFloat) -> Vec3 {
        (
            a.0 + scale * b.0,
            a.1 + scale * b.1,
            a.2 + scale * b.2
        )
    }

    pub fn scale_and_add_inplace(mut a: Vec3, b: &Vec3, scale: VecFloat) -> Vec3 {
        a.0 += scale * b.0;
        a.1 += scale * b.1;
        a.2 += scale * b.2;
        a
    }

    pub fn sub(a: &Vec3, b: &Vec3) -> Vec3 {
        (
            a.0 - b.0,
            a.1 - b.1,
            a.2 - b.2
        )
    }

    pub fn dot(a: &Vec3, b: &Vec3) -> VecFloat {
        a.0 * b.0 + a.1 * b.1 + a.2 * b.2
    }

    pub fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
        (
            a.1 * b.2 - a.2 * b.1,
            a.2 * b.0 - a.0 * b.2,
            a.0 * b.1 - a.1 * b.0
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
        (scale * a.0, scale * a.1, scale * a.2,)
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

    pub fn orthonormal_basis_of_plane(normal: &Vec3, primary_direction: &Vec3) -> Option<(Vec3, Vec3)> {
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_vec3_add() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!((-2.0, 3.0, 0.0), add(&a, &b));
        }

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
        fn test_vec3_sub() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!((4.0, 1.0, 6.0), sub(&a, &b));
        }

        #[test]
        fn test_vec3_dot() {
            let a = from_values(1.0, 2.0, 3.0);
            let b = from_values(-3.0, 1.0, -3.0);
            assert_eq!(-10.0, dot(&a, &b));
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
            assert!(equals(3.74165738677394138558, len(&a)));
        }

        #[test]
        fn test_vec3_normalize() {
            let a = normalize(&from_values(1.0, -2.0, 3.0));
            assert!(equals(0.26726124191242438468, a.0));
            assert!(equals(-0.53452248382484876937, a.1));
            assert!(equals(0.80178372573727315405, a.2));

            let b = normalize(&from_values(0.0, 0.0, 0.0));
            assert_eq!((0.0, 0.0, 0.0), b);
        }

        #[test]
        fn test_vec3_normalize_inplace() {
            let a = normalize_inplace(from_values(1.0, -2.0, 3.0));
            assert!(equals(0.26726124191242438468, a.0));
            assert!(equals(-0.53452248382484876937, a.1));
            assert!(equals(0.80178372573727315405, a.2));

            let b = normalize_inplace(from_values(0.0, 0.0, 0.0));
            assert_eq!((0.0, 0.0, 0.0), b);
        }

        #[test]
        fn test_vec3_orthonormal_basis_of_plane() {
            let n = from_values(0.0, 1.0, 0.0);
            let dir = from_values(1.0e10, 2.0e10, 1.0e10);
            let (u, v) = orthonormal_basis_of_plane(&n, &dir).unwrap();
            let sqrt_half = (0.5 as VecFloat).sqrt();
            assert!(equals(sqrt_half, u.0));
            assert!(equals(0.0, u.1));
            assert!(equals(sqrt_half, u.2));
            assert!(equals(-sqrt_half, v.0));
            assert!(equals(0.0, v.1));
            assert!(equals(sqrt_half, v.2));

            assert!(orthonormal_basis_of_plane(&n, &scale(&n, -2.0)).is_none());
        }
    }
}

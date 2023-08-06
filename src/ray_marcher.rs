use gl_matrix::common::{Vec2, Vec3, to_radian};
use gl_matrix::vec3;

use crate::sdf::Sdf;

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
    const MAX_RAY_ITER: u32 = 250;
    const MIN_SCENE_DIST: f32 = 0.001;
    const INITIAL_SCENE_DIST: f32 = 25.0 * Self::MIN_SCENE_DIST;
    const FINITE_DIFF_H: f32 = 0.001;
    const PENUMBRA: f32 = 48.0;

    pub fn new(
        camera: &Vec3,
        look_at: &Vec3,
        up: &Vec3,
        fov_y_degrees: f32,
        aspect_ratio: f32
    ) -> RayMarcher {
        let fov_y = to_radian(fov_y_degrees);
        let half_screen_length_y = (0.5 * fov_y).tan();
        let w = vec3::normalize(&mut vec3::create(), &vec3::subtract(&mut vec3::create(), look_at, camera)); // w = normalize(lookAt - camera)
        let v = vec3::normalize(&mut vec3::create(), &vec3::scale_and_add(&mut vec3::create(), up, &w, -vec3::dot(up, &w))); // v = normalize(up - dot(up, w) * w)
        let u = vec3::cross(&mut vec3::create(), &w, &v); // u = cross(w, v)

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

    // screen_coordinates \in [-1, 1]^2
    pub fn intersection_with_scene(&self, sdf: Sdf, screen_coordinates: &Vec2) -> Option<Vec3> {
        let dir = self.screen_direction(screen_coordinates);
        let mut p = vec3::create();
        let mut len: f32 = 0.0;
        let mut i: u32 = 0;
        while i < Self::MAX_RAY_ITER {
            vec3::scale_and_add(&mut p, &self.camera, &dir, len); // p = camera + len * dir
            let dist = sdf(&p);
            if dist < Self::MIN_SCENE_DIST {
                // we could return len and dir as well
                return Some(p);
            }
            len += dist;
            i += 1;
        }
        None
    }

    pub fn scene_normal(sdf: Sdf, p: &Vec3) -> Vec3 {
        let d_x = vec3::from_values(Self::FINITE_DIFF_H, 0.0, 0.0);
        let d_y = vec3::from_values(0.0, Self::FINITE_DIFF_H, 0.0);
        let d_z = vec3::from_values(0.0, 0.0, Self::FINITE_DIFF_H);

        let ppd_x = vec3::add(&mut vec3::create(), p, &d_x);
        let pmd_x = vec3::sub(&mut vec3::create(), p, &d_x);
        let ppd_y = vec3::add(&mut vec3::create(), p, &d_y);
        let pmd_y = vec3::sub(&mut vec3::create(), p, &d_y);
        let ppd_z = vec3::add(&mut vec3::create(), p, &d_z);
        let pmd_z = vec3::sub(&mut vec3::create(), p, &d_z);

        vec3::normalize(&mut vec3::create(), &vec3::from_values(
            sdf(&ppd_x) - sdf(&pmd_x),
            sdf(&ppd_y) - sdf(&pmd_y),
            sdf(&ppd_z) - sdf(&pmd_z),
        ))
    }

    pub fn light_intensity(sdf: Sdf, p: &Vec3, normal: &Vec3, point_source: &Vec3) -> f32 {
        const GLOBAL_INTENSITY: f32 = 0.1;
        let mut intensity = GLOBAL_INTENSITY;
        let visibility_factor = Self::visibility_factor(sdf, point_source, p, Some(normal));
        if visibility_factor > 0.0 {
            let point_intensity = vec3::dot(
                &vec3::normalize(&mut vec3::create(), &vec3::sub(&mut vec3::create(), &point_source, &p)),
                &normal
            ).max(0.0); // = max(dot(normalize(light - p), n), 0.0)
            intensity += (1.0 - intensity) * visibility_factor * point_intensity;
        }
        return intensity;
    }

    fn visibility_factor(sdf: Sdf, eye: &Vec3, p: &Vec3, point_normal: Option<&Vec3>) -> f32 {
        let to_eye = vec3::sub(&mut vec3::create(), &eye, &p);
        if point_normal.is_some_and(|n| vec3::dot(&to_eye, &n) < 0.0) { // is the normal pointing away from the eye point?
            return 0.0;
        }

        // if we walk from p towards eye, do we reach eye or hit the scene before?
        let dist_to_eye = vec3::len(&to_eye);
        let to_eye = vec3::normalize(&mut vec3::create(), &to_eye);

        let mut len = Self::INITIAL_SCENE_DIST;
        let mut q = vec3::create();
        let mut closest_miss_ratio: f32 = 1.0;
        let mut i: u32 = 0;
        while i < Self::MAX_RAY_ITER {
            if len >= dist_to_eye {
                return closest_miss_ratio;
            }

            vec3::scale_and_add(&mut q, &p, &to_eye, len); // q = p + len * dir

            let dist_to_scene = sdf(&q);
            if dist_to_scene < Self::MIN_SCENE_DIST {
                return 0.0;
            }

            closest_miss_ratio = closest_miss_ratio.min(Self::PENUMBRA * dist_to_scene / len);
            len += dist_to_scene;
            i += 1;
        }
        0.0
    }

    // screen_coordinates \in [-1, 1]^2
    fn screen_direction(&self, screen_coordinates: &Vec2) -> Vec3 {
        let p_u = screen_coordinates[0] * self.aspect_ratio * self.half_screen_length_y;
        let p_v = screen_coordinates[1] * self.half_screen_length_y;
        vec3::normalize(
            &mut vec3::create(),
            &vec3::scale_and_add(
                &mut vec3::create(),
                &vec3::scale_and_add(&mut vec3::create(), &self.w, &self.v, p_v),
                &self.u,
                p_u
            )
        ) // screen_direction = normalize(screen_coordinates.x * u + screen_coordinates.y * v + w)
    }
}

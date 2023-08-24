use crate::vector::{Vec2, Vec3, vec2, vec3, VecFloat};

use crate::sdf::Sdf;

pub struct RayMarcher {
    pub camera: Vec3,
    look_at: Vec3,
    up: Vec3,
    fov_y: f32,
    aspect_ratio: f32,
    half_screen_length_y: f32, // assuming half_screen_length_x = 1
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
        let fov_y = fov_y_degrees.to_radians();
        let half_screen_length_y = (0.5 * fov_y).tan();
        let w = vec3::normalize(&vec3::sub(look_at, camera)); // w = normalize(lookAt - camera)
        let v = vec3::normalize(&vec3::scale_and_add(up, &w, -vec3::dot(up, &w))); // v = normalize(up - dot(up, w) * w)
        let u = vec3::cross(&w, &v); // u = cross(w, v)

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
    pub fn intersection_with_scene(&self, sdf: Sdf, screen_coordinates: &Vec2) -> Option<(Vec3, VecFloat)> {
        let dir = self.screen_direction(screen_coordinates);
        let mut len: f32 = 0.0;
        let mut i: u32 = 0;
        while i < Self::MAX_RAY_ITER {
            let p = vec3::scale_and_add(&self.camera, &dir, len); // p = camera + len * dir
            let dist = sdf(&p);
            if dist < Self::MIN_SCENE_DIST {
                return Some((p, len));
            }
            len += dist;
            i += 1;
        }
        None
    }

    pub fn to_screen_coordinates(&self, p_scene: &Vec3) -> Vec2 {
        let camera_coord = self.to_camera_coordinates(p_scene);
        vec2::from_values(
            (camera_coord.0 / camera_coord.2) / (self.aspect_ratio * self.half_screen_length_y),
            (camera_coord.1 / camera_coord.2) / self.half_screen_length_y
        )
    }

    fn to_camera_coordinates(&self, p_scene: &Vec3) -> Vec3 {
        let q = vec3::sub(p_scene, &self.camera);
        vec3::from_values(
            vec3::dot(&q, &self.u),
            vec3::dot(&q, &self.v),
            vec3::dot(&q, &self.w)
        )
    }

    pub fn scene_normal(sdf: Sdf, p: &Vec3) -> Vec3 {
        let d_x = vec3::from_values(Self::FINITE_DIFF_H, 0.0, 0.0);
        let d_y = vec3::from_values(0.0, Self::FINITE_DIFF_H, 0.0);
        let d_z = vec3::from_values(0.0, 0.0, Self::FINITE_DIFF_H);

        let ppd_x = vec3::add(p, &d_x);
        let pmd_x = vec3::sub(p, &d_x);
        let ppd_y = vec3::add(p, &d_y);
        let pmd_y = vec3::sub(p, &d_y);
        let ppd_z = vec3::add(p, &d_z);
        let pmd_z = vec3::sub(p, &d_z);

        vec3::normalize_inplace(vec3::from_values(
            sdf(&ppd_x) - sdf(&pmd_x),
            sdf(&ppd_y) - sdf(&pmd_y),
            sdf(&ppd_z) - sdf(&pmd_z)
        ))
    }

    pub fn scene_normal_tetrahedron_diff(sdf: Sdf, p: &Vec3) -> Vec3 {
        // See tetrahedron technique from https://iquilezles.org/articles/normalsSDF/
        // k0 = [1,-1,-1], k1 = [-1,-1,1], k2 = [-1,1,-1], k3 = [1,1,1]
        const H: f32 = RayMarcher::FINITE_DIFF_H;
        let f0 = sdf(&vec3::from_values(p.0 + H, p.1 - H, p.2 - H));
        let f1 = sdf(&vec3::from_values(p.0 - H, p.1 - H, p.2 + H));
        let f2 = sdf(&vec3::from_values(p.0 - H, p.1 + H, p.2 - H));
        let f3 = sdf(&vec3::from_values(p.0 + H, p.1 + H, p.2 + H));

        vec3::normalize_inplace(vec3::from_values(
            f0 - f1 - f2 + f3,
           -f0 - f1 + f2 + f3,
           -f0 + f1 - f2 + f3
       )) // = normalize(\sum_i k_i * f_i)
    }

    pub fn light_intensity(sdf: Sdf, p: &Vec3, normal: &Vec3, point_source: &Vec3) -> f32 {
        const GLOBAL_INTENSITY: f32 = 0.0;
        let mut intensity = GLOBAL_INTENSITY;
        let visibility_factor = Self::visibility_factor(sdf, point_source, p, Some(normal));
        if visibility_factor > 0.0 {
            let point_intensity = vec3::dot(
                &vec3::normalize_inplace(vec3::sub(point_source, p)),
                normal
            ).max(0.0); // = max(dot(normalize(light - p), n), 0.0)
            intensity += (1.0 - intensity) * visibility_factor * point_intensity;
        }
        return intensity;
    }

    pub fn visibility_factor(sdf: Sdf, eye: &Vec3, p: &Vec3, point_normal: Option<&Vec3>) -> f32 {
        let to_eye = vec3::sub(eye, p);
        if point_normal.is_some_and(|n| vec3::dot(&to_eye, n) < 0.0) { // is the normal pointing away from the eye point?
            return 0.0;
        }

        // if we walk from p towards eye, do we reach eye or hit the scene before?
        let dist_to_eye = vec3::len(&to_eye);
        let to_eye = vec3::normalize_inplace(to_eye);

        let mut len = Self::INITIAL_SCENE_DIST;
        let mut closest_miss_ratio: f32 = 1.0;
        let mut i: u32 = 0;
        while i < Self::MAX_RAY_ITER {
            if len >= dist_to_eye {
                return closest_miss_ratio;
            }

            let q = vec3::scale_and_add(p, &to_eye, len); // q = p + len * dir

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
        let p_u = screen_coordinates.0 * self.aspect_ratio * self.half_screen_length_y;
        let p_v = screen_coordinates.1 * self.half_screen_length_y;
        vec3::normalize_inplace(
            vec3::scale_and_add_inplace(
                vec3::scale_and_add(&self.w, &self.v, p_v),
                &self.u,
                p_u
            )
        ) // screen_direction = normalize(screen_coordinates.x * u + screen_coordinates.y * v + w)
    }
}

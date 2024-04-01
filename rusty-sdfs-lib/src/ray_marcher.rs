use crate::scene::Scene;
use crate::sdf::{Material, ReflectiveProperties};
use crate::vector::{vec2, vec3, Vec2, Vec3, VecFloat};

pub struct RayMarcher {
    max_ray_iter_steps: u32,
    min_scene_dist: VecFloat,
    initial_scene_dist: VecFloat,
    finite_diff_h: VecFloat,
    step_size_factor: VecFloat, // set to 1 / sqrt(max_x(dh(x)/dx)^2 + 1) so safely raymarch heightmap h(x)
    pub camera: Vec3,
    look_at: Vec3,
    up: Vec3,
    fov_y: VecFloat,
    aspect_ratio: VecFloat,
    half_screen_length_y: VecFloat, // assuming half_screen_length_x = 1
    // Orthonormal basis of the camera system
    u: Vec3, // pointing to the right
    v: Vec3, // pointing up
    w: Vec3, // pointing towards the scene
}

impl RayMarcher {
    pub fn new(
        step_size_factor: VecFloat,
        camera: &Vec3,
        look_at: &Vec3,
        up: &Vec3,
        fov_y_degrees: VecFloat,
        aspect_ratio: VecFloat,
    ) -> RayMarcher {
        let fov_y = fov_y_degrees.to_radians();
        let half_screen_length_y = (0.5 * fov_y).tan();
        let w = vec3::normalize(&vec3::sub(look_at, camera)); // w = normalize(lookAt - camera)
        let v = vec3::normalize(&vec3::scale_and_add(up, &w, -vec3::dot(up, &w))); // v = normalize(up - dot(up, w) * w)
        let u = vec3::cross(&w, &v); // u = cross(w, v)

        RayMarcher {
            max_ray_iter_steps: (250.0 / step_size_factor).ceil() as u32,
            min_scene_dist: 0.001,
            initial_scene_dist: 25.0 * 0.001,
            finite_diff_h: 0.001 * step_size_factor,
            step_size_factor,
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
    pub fn intersection_with_scene(
        &self,
        scene: &impl Scene,
        screen_coordinates: &Vec2,
    ) -> Option<(Vec3, VecFloat, Material)> {
        let dir = self.screen_direction(screen_coordinates);
        let mut len: VecFloat = 0.0;
        for _ in 0..self.max_ray_iter_steps {
            let p = vec3::scale_and_add(&self.camera, &dir, len); // p = camera + len * dir
            let out = scene.eval(&p);
            if out.distance < self.min_scene_dist {
                return Some((p, len, out.material));
            }
            len += self.step_size_factor * out.distance;
        }
        None
    }

    pub fn to_screen_coordinates(&self, p_scene: &Vec3) -> Vec2 {
        let camera_coord = self.to_camera_coordinates(p_scene);
        vec2::from_values(
            (camera_coord.0 / camera_coord.2) / (self.aspect_ratio * self.half_screen_length_y),
            (camera_coord.1 / camera_coord.2) / self.half_screen_length_y,
        )
    }

    fn to_camera_coordinates(&self, p_scene: &Vec3) -> Vec3 {
        let q = vec3::sub(p_scene, &self.camera);
        vec3::from_values(
            vec3::dot(&q, &self.u),
            vec3::dot(&q, &self.v),
            vec3::dot(&q, &self.w),
        )
    }

    pub fn scene_normal(&self, scene: &impl Scene, p: &Vec3) -> Vec3 {
        let d_x = vec3::from_values(self.finite_diff_h, 0.0, 0.0);
        let d_y = vec3::from_values(0.0, self.finite_diff_h, 0.0);
        let d_z = vec3::from_values(0.0, 0.0, self.finite_diff_h);

        let ppd_x = vec3::add(p, &d_x);
        let pmd_x = vec3::sub(p, &d_x);
        let ppd_y = vec3::add(p, &d_y);
        let pmd_y = vec3::sub(p, &d_y);
        let ppd_z = vec3::add(p, &d_z);
        let pmd_z = vec3::sub(p, &d_z);

        vec3::normalize_inplace(vec3::from_values(
            scene.eval(&ppd_x).distance - scene.eval(&pmd_x).distance,
            scene.eval(&ppd_y).distance - scene.eval(&pmd_y).distance,
            scene.eval(&ppd_z).distance - scene.eval(&pmd_z).distance,
        ))
    }

    pub fn scene_normal_heightmap(&self, scene: &impl Scene, p: &Vec3) -> Vec3 {
        let d_x = vec3::from_values(self.finite_diff_h, 0.0, 0.0);
        let d_z = vec3::from_values(0.0, 0.0, self.finite_diff_h);

        let ppd_x = vec3::add(p, &d_x);
        let pmd_x = vec3::sub(p, &d_x);
        let ppd_z = vec3::add(p, &d_z);
        let pmd_z = vec3::sub(p, &d_z);

        vec3::normalize_inplace(vec3::from_values(
            (scene.eval(&ppd_x).distance - p.1) - (scene.eval(&pmd_x).distance - p.1),
            2.0 * self.finite_diff_h,
            (scene.eval(&ppd_z).distance - p.1) - (scene.eval(&pmd_z).distance - p.1),
        ))
    }

    pub fn scene_normal_tetrahedron_diff(&self, scene: &impl Scene, p: &Vec3) -> Vec3 {
        // See tetrahedron technique from https://iquilezles.org/articles/normalsSDF/
        // k0 = [1,-1,-1], k1 = [-1,-1,1], k2 = [-1,1,-1], k3 = [1,1,1]
        let h = self.finite_diff_h;
        let f0 = scene
            .eval(&vec3::from_values(p.0 + h, p.1 - h, p.2 - h))
            .distance;
        let f1 = scene
            .eval(&vec3::from_values(p.0 - h, p.1 - h, p.2 + h))
            .distance;
        let f2 = scene
            .eval(&vec3::from_values(p.0 - h, p.1 + h, p.2 - h))
            .distance;
        let f3 = scene
            .eval(&vec3::from_values(p.0 + h, p.1 + h, p.2 + h))
            .distance;

        vec3::normalize_inplace(vec3::from_values(
            f0 - f1 - f2 + f3,
            -f0 - f1 + f2 + f3,
            -f0 + f1 - f2 + f3,
        )) // = normalize(\sum_i k_i * f_i)
    }

    fn ambient_visibility(
        scene: &impl Scene,
        p: &Vec3,
        normal: &Vec3,
        step_count: u32,
        step_size: VecFloat,
    ) -> VecFloat {
        let mut acc_occlusion: VecFloat = 0.0;
        for step in 1..=step_count {
            let dist_step = step as VecFloat * step_size;
            let p_step = vec3::scale_and_add(p, normal, dist_step);
            let dist_sdf = scene.eval(&p_step).distance;
            let occlusion = (dist_step - dist_sdf.clamp(0.0, dist_step)) / dist_step;
            let weight = 0.5f32.powi(step as i32);
            acc_occlusion += weight * occlusion;
        }
        let max_acc_occlusion: VecFloat = 1.0 - 0.5f32.powi(step_count as i32); // cf. partial geometric series
        let occlusion = acc_occlusion / max_acc_occlusion;
        1.0 - occlusion
    }

    pub fn light_intensity(
        &self,
        scene: &impl Scene,
        properties: &ReflectiveProperties,
        p: &Vec3,
        normal: &Vec3,
        light: &Vec3,
    ) -> VecFloat {
        let ambient = properties.ambient_weight;
        let ao = if properties.ao_weight > 0.0 {
            properties.ao_weight
                * Self::ambient_visibility(
                    scene,
                    p,
                    normal,
                    properties.ao_steps,
                    properties.ao_step_size,
                )
        } else {
            0.0
        };
        let visibility_factor =
            self.visibility_factor(scene, light, p, Some(normal), properties.penumbra);
        let visibility = properties.visibility_weight * visibility_factor;
        let (diffuse, specular) = if visibility_factor > 0.0 {
            let to_light = vec3::normalize_inplace(vec3::sub(light, p));
            let diffuse = properties.diffuse_weight
                * visibility_factor
                * vec3::dot(&to_light, normal).max(0.0); // = max(dot(normalize(light - p), n), 0.0)

            let from_light = vec3::scale(&to_light, -1.0);
            let to_camera = vec3::normalize_inplace(vec3::sub(&self.camera, p));
            let specular = properties.specular_weight
                * visibility_factor
                * vec3::dot(&vec3::reflect(&from_light, normal), &to_camera)
                    .max(0.0)
                    .powf(properties.specular_exponent);

            (diffuse, specular)
        } else {
            (0.0, 0.0)
        };

        ambient + ao + visibility + diffuse + specular
    }

    pub fn visibility_factor(
        &self,
        scene: &impl Scene,
        eye: &Vec3,
        p: &Vec3,
        point_normal: Option<&Vec3>,
        penumbra: VecFloat,
    ) -> VecFloat {
        let to_eye = vec3::sub(eye, p);
        if point_normal.is_some_and(|n| vec3::dot(&to_eye, n) < 0.0) {
            // if the normal is pointing away from the eye point...
            return 0.0;
        }

        // if we walk from p towards eye, do we reach eye or hit the scene before?
        let dist_to_eye = vec3::len(&to_eye);
        let to_eye = vec3::normalize_inplace(to_eye);

        let mut len = self.initial_scene_dist;
        let mut closest_miss_ratio: VecFloat = 1.0;
        for _ in 0..self.max_ray_iter_steps {
            if len >= dist_to_eye {
                return closest_miss_ratio;
            }

            let q = vec3::scale_and_add(p, &to_eye, len); // q = p + len * dir

            let dist_to_scene = scene.eval(&q).distance;
            if dist_to_scene < self.min_scene_dist {
                return 0.0;
            }

            closest_miss_ratio = closest_miss_ratio.min(penumbra * dist_to_scene / len);
            len += dist_to_scene;
        }
        0.0
    }

    // screen_coordinates \in [-1, 1]^2
    fn screen_direction(&self, screen_coordinates: &Vec2) -> Vec3 {
        let p_u = screen_coordinates.0 * self.aspect_ratio * self.half_screen_length_y;
        let p_v = screen_coordinates.1 * self.half_screen_length_y;
        vec3::normalize_inplace(vec3::scale_and_add_inplace(
            vec3::scale_and_add(&self.w, &self.v, p_v),
            &self.u,
            p_u,
        )) // screen_direction = normalize(screen_coordinates.x * u + screen_coordinates.y * v + w)
    }
}

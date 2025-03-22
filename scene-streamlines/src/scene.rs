use core::{f32, prelude};
use std::cell;
use std::f32::consts::{PI};

use rusty_sdfs_lib::{vec2, vec3, vec4, Vec2, Vec3, Vec4, VecFloat};
use rusty_sdfs_lib::Scene;
use rusty_sdfs_lib::{Material, ReflectiveProperties, SdfOutput};
use rusty_sdfs_lib::smoothstep;
use rusty_sdfs_lib::noise_1d;
use rusty_sdfs_lib::sdf_op::{
    op_elongate_y, op_elongate_z, op_onion, op_repeat_rotated_y, op_repeat, op_repeat_finite, op_repeat_xz, op_rotate_quaternion, op_rotate_y, op_rotate_z, op_round, op_shift, op_smooth_difference, op_smooth_union, sd_box, sd_capped_cone, sd_cylinder, sd_cylinder_rounded, sd_plane, sd_sphere, sd_torus
};

const TO_RAD: VecFloat = PI / 180.0;

fn hash2d(v: &Vec2, offset: VecFloat) -> VecFloat {
    ((v.0 + 113.0 * v.1 + offset).sin() * 43758.5453123)
        .fract()
        .abs()
}

pub struct SceneOcean {
    light: Vec3,
    material_surface: Material,
}

impl SceneOcean {
    pub fn new() -> SceneOcean {
        let light = vec3::from_values(0.0, 8.0, 10.0);

        let surface_hsl = vec3::from_values(0.0f32.to_radians(), 0.0, 1.0);
        let surface_reflective_props = ReflectiveProperties::new(0.1, 0.0, 0.0, 0.8, 0.1, None, None, None, None);
        let material_surface = Material::new(
            &light,
            Some(&surface_reflective_props),
            Some(&surface_hsl),
            true,
            false,
        );

        SceneOcean {
            light,
            material_surface,
        }
    }

    pub fn camera(&self) -> Vec3 {
        vec3::from_values(0.0, 2.5, 5.0)
    }

    pub fn look_at(&self) -> Vec3 {
        vec3::from_values(0.0, 0.0, 0.0)
    }

    pub fn fov(&self) -> VecFloat {
        55.0
    }

    pub fn hsl_streamlines(&self) -> Vec3 {
        vec3::from_values(227.0f32.to_radians(), 1.0, 0.0)
    }

    fn height_map_octave(p: &Vec2) -> VecFloat {
        p.0.sin() * p.1.sin()
    }

    fn height_map(p: &Vec3) -> VecFloat {
        const MAX_ITER: u32 = 3;
        let uv = vec2::from_values(p.0, p.2);
        let mut freq = 1.0f32;
        let mut h = 0.0f32;
        for _ in 0.. MAX_ITER {
            h += (1.0 / freq) * Self::height_map_octave(&vec2::scale(&uv, freq));
            freq *= 4.0;
        }
        h
    }
}

impl Scene for SceneOcean {
    fn eval(&self, p: &Vec3) -> SdfOutput {
        let h = SceneOcean::height_map(p);
        SdfOutput {
            distance: (h - p.1).abs(),
            material: self.material_surface,
        }
    }
}


pub struct ScenePillars {
    light: Vec3,
    material_pillar: Material,
}

impl ScenePillars {
    pub fn new() -> ScenePillars {
        let light = vec3::scale_inplace(vec3::unit_polar_to_cartesian(0.29 * PI, 0.3 * PI), 1.0e5);

        let pillar_hsl = vec3::from_values(0.0f32.to_radians(), 0.0, 1.0);
        let pillar_reflective_props = ReflectiveProperties::new(
            0.1,
            0.0,
            0.0,
            0.9,
            0.0,
            None,
            None,
            None,
            None
        );
        let material_pillar = Material::new(
            &light,
            Some(&pillar_reflective_props),
            Some(&pillar_hsl),
            false,
            true,
        );

        ScenePillars {
            light,
            material_pillar,
        }
    }

    pub fn camera(&self) -> Vec3 {
        vec3::from_values(-1.0, 1.5, 6.0)
        // vec3::from_values(0.0, 4.0, 4.0)
    }

    pub fn look_at(&self) -> Vec3 {
        vec3::from_values(0.0, 4.0, 0.0)
        // vec3::from_values(0.0, 0.0, 0.0)
    }

    pub fn fov(&self) -> VecFloat {
        80.0
    }

    pub fn hsl_streamlines(&self) -> Vec3 {
        vec3::from_values(0.0, 0.0, 0.0)
    }

    fn sd_brick(&self, p: &Vec3, _cell_id: &Vec2) -> SdfOutput {
        // let dist = sd_box(p, &vec3::from_values(0.25, 0.1, 0.5)) - 0.1;
        let dist = sd_box(p, &vec3::from_values(0.5, 0.1, 0.25)) - 0.1;

        SdfOutput {
            distance: dist,
            material: self.material_pillar,
        }
    }
}

impl Scene for ScenePillars {

    fn eval(&self, p: &Vec3) -> SdfOutput {
        const Y_PERIOD: VecFloat = 1.0;
        let xz_angle = p.2.atan2(p.0);
        // let y_period = smoothstep(0.0, 6.0, p.1) + 0.5;
        let period_offset = xz_angle * (Y_PERIOD / (2.0 * PI));
        let y = p.1 + period_offset;
        let y_index = (y / Y_PERIOD).round();
        let y_offset = y - (y_index * Y_PERIOD);
        let x = p.0 + 1.2 * noise_1d(0.15 * p.1, 1);
        let z = p.2 + 2.0 * noise_1d(0.15 * p.1 + 370.0, 1);
        let q = vec3::from_values(x, y_offset, z);
        let helix = sd_torus(&q, 1.0, 0.1);

        let thickness_modifier = 0.31 * smoothstep(0.0, 10.0, p.1) - 0.2;
        let helix = helix + thickness_modifier;

        // let squashed_sphere = sd_sphere(&vec3::from_values(p.0, p.1, 2.0*p.2), 1.0);

        // let p_repeat = op_repeat_rotated_y(p, 20.0);
        // let squashed_sphere = sd_sphere(&vec3::from_values(p_repeat.0, p_repeat.1, 2.0*p_repeat.2), 1.0);
        // TODO: Intersect with hollow spheres

        let bricks = op_repeat_xz(
            |p: &Vec3, cell_id: &Vec2| self.sd_brick(p, cell_id),
            p,
            &vec2::from_values(1.15, 0.65),//&vec2::from_values(0.65, 1.15)
        );

        // let sphere_0 = sd_sphere(&op_shift(p, &vec3::from_values(1.0, 0.0, 0.0)), 0.2);
        // let sphere_1 = sd_sphere(&op_shift(p, &vec3::from_values(1.0, 2.0, 0.0)), 0.2);

        // let plane = sd_plane(p, &vec3::from_values(0.0, 1.0, 0.0), 0.0);

        let (scene, _) = op_smooth_union(bricks.distance, helix, 0.5);

        SdfOutput {
            distance: scene,//.min(sphere_0).min(sphere_1),
            material: self.material_pillar,
        }
        // bricks
    }
}


pub struct SceneTrees {
    light: Vec3,
    material_tree: Material,
    trees: Vec<Vec<TreeTrunk>>,
}

struct TreeTrunk {
    base: Vec3,
    direction: Vec3,
    length: VecFloat,
    radius_base: VecFloat,
    radius_reduction_factor: VecFloat,
    q_rotation: Vec4,
}

impl TreeTrunk {
    fn new(base: Vec3, azimuth: VecFloat, inclination: VecFloat, length: VecFloat, radius_base: VecFloat, radius_reduction_factor: VecFloat) -> TreeTrunk {
        let direction = vec3::unit_polar_to_cartesian(azimuth, inclination);;
        let q_rotation = vec4::quaternion_rotation_to_direction(&direction, &vec3::from_values(0.0, 1.0, 0.0));
        TreeTrunk {
            base,
            direction,
            length,
            radius_base,
            radius_reduction_factor,
            q_rotation,
        }
    }

    fn from_points(base: Vec3, top: Vec3, radius_base: VecFloat, radius_reduction_factor: VecFloat) -> TreeTrunk {
        let direction = vec3::sub(&top, &base);
        let length = vec3::len(&direction);
        let direction = vec3::scale(&direction, 1.0 / length);
        let q_rotation = vec4::quaternion_rotation_to_direction(&direction, &vec3::from_values(0.0, 1.0, 0.0));
        TreeTrunk {
            base,
            direction,
            length,
            radius_base,
            radius_reduction_factor,
            q_rotation,
        }
    }

    fn branch(&self, base_level: VecFloat, azimuth: VecFloat, inclination: VecFloat, length_factor: VecFloat, radius_factor: VecFloat) -> TreeTrunk {
        let base = vec3::scale_and_add(&self.base, &self.direction, base_level * self.length);
        let length = length_factor * self.length;
        let radius_base = radius_factor * self.radius_base;
        Self::new(base, azimuth, inclination, length, radius_base, 0.8 * self.radius_reduction_factor)
    }

    fn sd(&self, p: &Vec3) -> VecFloat {
        let p_base = op_shift(p, &self.base);
        let p_rotated = op_rotate_quaternion(&p_base, &self.q_rotation);
        let half_length = 0.5 * self.length;
        let p_shifted = op_shift(&p_rotated, &vec3::from_values(0.0, half_length, 0.0));
        sd_capped_cone(&p_shifted, self.radius_base, self.radius_base * self.radius_reduction_factor, half_length)
    }
}

impl SceneTrees {
    pub fn new() -> SceneTrees {
        let light = vec3::scale_inplace(vec3::unit_polar_to_cartesian(0.57 * PI, 0.45 * PI), 1.0e5);

        let tree_hsl = vec3::from_values(0.0f32.to_radians(), 0.0, 1.0);
        let tree_reflective_props = ReflectiveProperties::new(0.2, 0.0, 0.0, 0.8, 0.0, None, None, None, None);
        let material_tree = Material::new(
            &light,
            Some(&tree_reflective_props),
            Some(&tree_hsl),
            true,
            false,
        );

        let mut trees = vec![];

        { // left of middle tree
            const T: VecFloat = 32.0;
            let trunk1 = TreeTrunk::from_points(
                vec3::from_values(-4.5, 0.0, -3.0),
                vec3::from_values(0e0 + T * -4.1216892e-1, 3e0 + T * 7.017978e-1, 1.5e1 + T * -5.810306e-1),
                1.6,
                0.85
            );
            let branch11 = trunk1.branch(0.45, 0.65 * PI, 0.77 * PI, 0.2, 0.29);
            trees.push(vec![trunk1, branch11]);
        }

        { // middle tree
            const T: VecFloat = 10.0;
            let trunk2 = TreeTrunk::from_points(
                vec3::from_values(-2.5, 0.0, 12.0),
                vec3::from_values(0e0 + T * 3.9876002e-1, 3e0 + T * 6.116195e-1, 1.5e1 + T * -6.833096e-1),
                1.5,
                0.85
            );
            let branch21 = trunk2.branch(0.68, 0.7 * PI, 0.23 * PI, 0.23, 0.1);
            trees.push(vec![trunk2, branch21]);
        }

        { // far left
            const T: VecFloat = 35.0;
            let trunk3 = TreeTrunk::from_points(
                vec3::from_values(-13.1, -5.0, -8.0),
                vec3::from_values(0e0 + T * -6.887686e-1, 3e0 + T * 2.0714737e-1, 1.5e1 + T * -6.9475734e-1),
                1.4,
                0.6
            );
            trees.push(vec![trunk3]);
        }

        { // far right
            const T: VecFloat = 50.0;
            let trunk4 = TreeTrunk::from_points(
                vec3::from_values(22.5, -5.0, -20.0),
                vec3::from_values(0e0 + T * 6.6775227e-1, 3e0 + T * 2.2115262e-1, 1.5e1 + T * -7.107732e-1),
                1.4,
                0.7
            );
            let branch41 = trunk4.branch(0.55, 0.4 * PI, 0.35 * PI, 0.24, 0.28);
            trees.push(vec![trunk4, branch41]);
        }

        { // right of middle
            const T: VecFloat = 5.0;
            let trunk5 = TreeTrunk::from_points(
                vec3::from_values(8.5, -5.0, 0.0),
                vec3::from_values(0e0 + T * 2.4689893e-1, 3e0 + T * 3.7532985e-1, 1.5e1 + T * -8.9340276e-1),
                0.35,
                0.95
            );
            trees.push(vec![trunk5]);
        }

        SceneTrees {
            light,
            material_tree,
            trees,
        }
    }

    pub fn camera(&self) -> Vec3 {
        vec3::from_values(0.0, 3.0, 15.0)
        // vec3::from_values(0.0, 100.0, 1.0)
    }

    pub fn look_at(&self) -> Vec3 {
        vec3::from_values(0.0, 10.0, 0.0)
        // vec3::from_values(0.0, 0.0, 0.0)

    }

    pub fn fov(&self) -> VecFloat {
        65.0
    }

    pub fn hsl_streamlines(&self) -> Vec3 {
        vec3::from_values(0.0, 0.0, 0.0)
    }

    fn sd_trunk(&self, p: &Vec3, base: &Vec3, direction: &Vec3, length: VecFloat, radius_base: VecFloat, radius_reduction_factor: VecFloat) -> VecFloat {
        let q = vec4::quaternion_rotation_to_direction(&direction, &vec3::from_values(0.0, 1.0, 0.0));
        let p_base = op_shift(p, base);
        let p_rotated = op_rotate_quaternion(&p_base, &q);
        let half_length = 0.5 * length;
        let p_shifted = op_shift(&p_rotated, &vec3::from_values(0.0, half_length, 0.0));
        sd_capped_cone(&p_shifted, radius_base, radius_base * radius_reduction_factor, half_length)
    }
}

impl Scene for SceneTrees {
    fn eval(&self, p: &Vec3) -> SdfOutput {
        // TODO:
        // * Wedge cutouts
        // * Clouds
        const SMOOTHING_WIDTH: VecFloat = 0.45;
        let scene = self.trees.iter().fold(f32::INFINITY, |acc_trees, tree| {
            let sd_tree = tree.iter().fold(f32::INFINITY, |acc_trunks, trunk| {
                let (sd, _) = op_smooth_union(acc_trunks, trunk.sd(p), SMOOTHING_WIDTH);
                sd
            });
            acc_trees.min(sd_tree)
        });
        SdfOutput { distance: scene, material: self.material_tree }
    }
}

pub struct SceneMeadow {
    light: Vec3,
    material_core: Material,
    material_shell: Material,
    material_floor: Material,
}

impl SceneMeadow {
    pub fn new() -> SceneMeadow {
        let light = vec3::from_values(1.75e5, 3.5e5, 1.5e5);
        let rp = ReflectiveProperties::new(0.0, 0.0, 0.0, 1.0, 0.0, None, None, None, None);
        let core_hsl = vec3::from_values(50.0f32.to_radians(), 1.0, 0.55);
        let material_core = Material::new(&light, Some(&rp), Some(&core_hsl), false, true);
        let shell_hsl = vec3::from_values(169.0f32.to_radians(), 0.96, 0.55);
        let material_shell = Material::new(&light, Some(&rp), Some(&shell_hsl), false, true);
        let floor_hsl = vec3::from_values(211.0f32.to_radians(), 0.73, 0.6);
        let material_floor = Material::new(&light, Some(&rp), Some(&floor_hsl), false, true);
        SceneMeadow {
            light,
            material_core,
            material_shell,
            material_floor,
        }
    }

    pub fn camera(&self) -> Vec3 {
        vec3::from_values(5.0, 7.0, 5.0)
    }

    pub fn look_at(&self) -> Vec3 {
        vec3::from_values(0.9, 1.35, -4.0)
    }

    pub fn fov(&self) -> VecFloat {
        45.0
    }

    pub fn hsl_streamlines(&self) -> Vec3 {
        vec3::from_values(0.0, 0.0, 0.0)
    }

    fn sd_flower(&self, p: &Vec3, cell_id: &Vec2) -> SdfOutput {
        const HASH_INC: VecFloat = 0.1;
        let x_jitter = 0.5 * (1.0 - 2.0 * hash2d(cell_id, 6.0 * HASH_INC));
        let z_jitter = 0.5 * (1.0 - 2.0 * hash2d(cell_id, 7.0 * HASH_INC));
        let sphere_radius = 0.45 + 0.55 * hash2d(cell_id, 0.0);
        let shell_radius = 1.1 * sphere_radius;
        let shell_thickness = 0.025 * sphere_radius;
        let opening_angle_xz = PI * (0.2 + 0.2 * hash2d(cell_id, 3.0 * HASH_INC));
        let opening_angle_y = PI * (0.2 + 0.1 * hash2d(cell_id, 4.0 * HASH_INC));
        let opening_distance = sphere_radius * (0.7 + 0.2 * hash2d(cell_id, 5.0 * HASH_INC));
        let opening_radius = shell_radius * (0.65 + 0.25 * hash2d(cell_id, 8.0 * HASH_INC));
        let shell_opening_k = 0.25 * sphere_radius;
        let shell_core_k = 0.1 * sphere_radius;
        let stem_height = 0.65 + sphere_radius * 0.7 * hash2d(cell_id, HASH_INC);
        let stem_radius = sphere_radius * (0.15 + 0.1 * hash2d(cell_id, 2.0 * HASH_INC));
        let stem_k = 0.9 * sphere_radius;

        let p_local = op_shift(
            p,
            &vec3::from_values(x_jitter, sphere_radius + 2.0 * stem_height, z_jitter),
        );
        let dir_opening = vec3::from_values(
            opening_distance * opening_angle_y.sin() * opening_angle_xz.cos(),
            opening_distance * opening_angle_y.cos(),
            opening_distance * opening_angle_y.sin() * opening_angle_xz.sin(),
        );

        let core = sd_sphere(&p_local, sphere_radius);
        let opening: f32 = sd_sphere(&op_shift(&p_local, &dir_opening), opening_radius);
        let (shell, _) = op_smooth_difference(
            op_onion(sd_sphere(&p_local, shell_radius), shell_thickness),
            opening,
            shell_opening_k,
        );
        let stem = sd_sphere(
            &op_elongate_y(
                &op_shift(&p_local, &vec3::from_values(0.0, -2.0 * stem_height, 0.0)),
                1.5 * stem_height,
            ),
            stem_radius,
        );

        let (bulb, bulb_t) = op_smooth_union(core, shell, shell_core_k);
        let material_flower = self.material_core.lerp(&self.material_shell, bulb_t);
        let (flower, _) = op_smooth_union(bulb, stem, stem_k);
        SdfOutput::new(flower, material_flower)
    }
}

impl Scene for SceneMeadow {
    fn eval(&self, p: &Vec3) -> SdfOutput {
        let cell_size = 2.75;

        let flowers = op_repeat_xz(
            |p: &Vec3, cell_id: &Vec2| self.sd_flower(p, cell_id),
            p,
            &vec2::from_values(cell_size, cell_size),
        );

        let floor_deformation = 0.03
            * ((2.0 * PI * p.0 / cell_size).cos()
                + (2.0 * PI * p.1 / cell_size).cos()
                + 0.5 * (3.0 * 2.0 * PI * p.0 / cell_size).cos()
                + 0.5 * (2.0 * 2.0 * PI * p.1 / cell_size).cos());
        let floor = sd_plane(
            p,
            &vec3::from_values(0.0, 1.0, 0.0),
            0.15 + floor_deformation,
        );
        let (scene, scene_t) = op_smooth_union(floor, flowers.distance, 0.65);
        SdfOutput::new(
            scene,
            self.material_floor.lerp(&flowers.material, scene_t.powi(2)),
        )
    }
}

pub fn scene_planet(p: &Vec3) -> SdfOutput {
    // let camera = vec3::from_values(0.0, 0.0, 5.0);
    // let look_at = vec3::from_values(0.0, 0.0, 0.0);
    // let up = vec3::from_values(0.0, 1.0, 0.0);
    let light = vec3::from_values(-1.0, 1.0, -20.0);
    const PLANET_RADIUS: VecFloat = 10.0;
    const PLANET_THICKNESS: VecFloat = 1.0;
    const OPENING_ANGLE_XZ: VecFloat = -60.0 * TO_RAD;
    const OPENING_ANGLE_Y: VecFloat = 90.0 * TO_RAD;
    const OPENING_DISTANCE: VecFloat = 0.5 * PLANET_RADIUS;

    let planet = op_onion(sd_sphere(p, PLANET_RADIUS), PLANET_THICKNESS);
    let dir_cutout = vec3::from_values(
        OPENING_DISTANCE * OPENING_ANGLE_Y.sin() * OPENING_ANGLE_XZ.cos(),
        OPENING_DISTANCE * OPENING_ANGLE_Y.cos(),
        OPENING_DISTANCE * OPENING_ANGLE_Y.sin() * OPENING_ANGLE_XZ.sin(),
    );
    let cutout = sd_sphere(&op_shift(p, &dir_cutout), 0.75 * PLANET_RADIUS);

    let material_planet = Material::new(&light, None, None, true, true);
    let (open_planet, _) = op_smooth_difference(planet, cutout, 1.0);
    SdfOutput::new(open_planet, material_planet)
}

pub fn scene_capsules(p: &Vec3) -> f32 {
    let base = sd_plane(p, &vec3::from_values(0.0, 1.0, 0.0), 0.0);
    let bg_tilt = -30.0f32.to_radians();
    let background = sd_plane(
        p,
        &vec3::from_values(bg_tilt.sin(), 0.0, bg_tilt.cos()),
        -8.0,
    );

    let capsule1 = sd_sphere(
        &op_elongate_y(&op_shift(p, &vec3::from_values(1.0, 0.0, 0.25)), 0.45),
        1.0,
    );
    let capsule2 = sd_sphere(
        &op_elongate_y(&op_shift(p, &vec3::from_values(-1.5, 0.0, 0.0)), 0.6),
        0.9,
    );
    let capsule3 = sd_sphere(
        &op_elongate_y(&op_shift(p, &vec3::from_values(-0.2, 0.0, -2.0)), 1.5),
        0.8,
    );
    let capsule4 = sd_sphere(
        &op_elongate_y(&op_shift(p, &vec3::from_values(2.0, 0.0, -2.0)), 2.1),
        0.8,
    );

    base.min(background)
        .min(capsule1)
        .min(capsule2)
        .min(capsule3)
        .min(capsule4)
}

fn sd_stacked_pillar(p: &Vec3) -> VecFloat {
    const STRETCH: f32 = 1.13;
    const HEIGHT: f32 = 0.55;
    const RADIUS: f32 = 1.0;
    let p_elongated = op_elongate_z(p, STRETCH);
    let p_repeated = op_repeat_finite(
        p,
        &vec3::from_values(1.0, 2.0 * (HEIGHT + 0.025), 1.0),
        &vec3::from_values(0.0, -5.0, 0.0),
        &vec3::from_values(0.0, -1.0, 0.0),
    );
    let sd_rounded_top = sd_cylinder_rounded(&p_elongated, RADIUS, HEIGHT, 0.15);
    let sd_sharp_bottom = sd_cylinder(
        &op_shift(&p_elongated, &vec3::from_values(0.0, -0.5 * HEIGHT, 0.0)),
        RADIUS,
        0.5 * HEIGHT,
    );
    let sd_stack = sd_cylinder(&op_elongate_z(&p_repeated, STRETCH), RADIUS, HEIGHT);
    sd_rounded_top.min(sd_sharp_bottom).min(sd_stack)
}

fn sd_cromwell_balcony(
    p: &Vec3,
    window_ledge_height: VecFloat,
    balcony_half_length: VecFloat,
) -> VecFloat {
    let balcony_half_height = 0.5 * (window_ledge_height + 0.18);

    sd_box(
        p,
        &vec3::from_values(1.0, 0.5 * window_ledge_height, balcony_half_length),
    )
    .min(sd_box(
        &op_rotate_z(
            &op_shift(p, &vec3::from_values(1.05, window_ledge_height - 0.1, 0.0)),
            28.0f32.to_radians(),
        ),
        &vec3::from_values(
            0.5 * 0.6 * window_ledge_height,
            0.5 * 2.1 * window_ledge_height,
            balcony_half_length,
        ),
    ))
    .max(sd_box(
        &op_shift(
            p,
            &vec3::from_values(0.0, balcony_half_height - 0.5 * window_ledge_height, 0.0),
        ),
        &vec3::from_values(1.25, balcony_half_height, balcony_half_length),
    ))
}

fn sd_cromwell_tower(p: &Vec3) -> VecFloat {
    const PILLAR_HALF_SIDE: VecFloat = 0.5 * 0.9;
    const PILLAR_HALF_HEIGHT: VecFloat = 0.5 * 0.55 * 4.0 * 20.5;
    const PILLAR_SPACING: VecFloat = 2.77;
    let p_repeated_pillars = op_repeat_finite(
        &op_shift(p, &vec3::from_values(0.0, PILLAR_HALF_HEIGHT, 0.0)),
        &vec3::from_values(1.0, 1.0, PILLAR_SPACING),
        &vec3::from_values(0.0, 0.0, -2.0),
        &vec3::from_values(0.0, 0.0, 2.0),
    );
    let pillars = sd_box(
        &p_repeated_pillars,
        &vec3::from_values(PILLAR_HALF_SIDE, PILLAR_HALF_HEIGHT, PILLAR_HALF_SIDE),
    );

    const STORY_HEIGHT: VecFloat = 0.895;
    const WINDOW_LEDGE_HEIGHT: VecFloat = 0.23;
    let windows = sd_box(
        &op_shift(
            p,
            &vec3::from_values(-1.0 * PILLAR_HALF_SIDE, PILLAR_HALF_HEIGHT, 0.0),
        ),
        &vec3::from_values(
            PILLAR_HALF_SIDE,
            PILLAR_HALF_HEIGHT - STORY_HEIGHT,
            0.5 * 4.0 * PILLAR_SPACING,
        ),
    );

    const HALF_STORY_COUNT: VecFloat = 21.0;
    let p_repeated_window_ledges = op_repeat_finite(
        &op_shift(
            p,
            &vec3::from_values(-0.25 * PILLAR_HALF_SIDE, PILLAR_HALF_HEIGHT, 0.0),
        ),
        &vec3::from_values(1.0, STORY_HEIGHT, 1.0),
        &vec3::from_values(0.0, -HALF_STORY_COUNT, 0.0),
        &vec3::from_values(0.0, HALF_STORY_COUNT, 0.0),
    );
    let window_ledges = sd_box(
        &p_repeated_window_ledges,
        &vec3::from_values(
            PILLAR_HALF_SIDE,
            0.5 * WINDOW_LEDGE_HEIGHT,
            0.5 * 4.0 * PILLAR_SPACING,
        ),
    );

    const SMALL_LEDGE_HEIGHT: VecFloat = 0.6 * WINDOW_LEDGE_HEIGHT;
    const SMALL_LEDGE_WIDTH: VecFloat = 3.44;
    let p_repeated_small_ledges = op_repeat_finite(
        &op_shift(
            p,
            &vec3::from_values(
                -0.25 * PILLAR_HALF_SIDE,
                PILLAR_HALF_HEIGHT - (WINDOW_LEDGE_HEIGHT - SMALL_LEDGE_HEIGHT),
                2.0 * PILLAR_SPACING + 0.5 * SMALL_LEDGE_WIDTH,
            ),
        ),
        &vec3::from_values(1.0, STORY_HEIGHT, 1.0),
        &vec3::from_values(0.0, -HALF_STORY_COUNT, 0.0),
        &vec3::from_values(0.0, HALF_STORY_COUNT + 1.0, 0.0),
    );
    let small_ledges = sd_box(
        &p_repeated_small_ledges,
        &vec3::from_values(
            PILLAR_HALF_SIDE,
            0.5 * SMALL_LEDGE_HEIGHT,
            0.5 * SMALL_LEDGE_WIDTH,
        ),
    );

    const WALL_ANGLE: VecFloat = -38.0 * PI / 180.0;
    let p_wall_shifted = op_shift(
        p,
        &vec3::from_values(
            0.0,
            PILLAR_HALF_HEIGHT,
            2.0 * PILLAR_SPACING + SMALL_LEDGE_WIDTH,
        ),
    );
    let p_wall_rotated = op_rotate_y(&p_wall_shifted, WALL_ANGLE);
    let balcony_wall = sd_box(
        &p_wall_rotated,
        &vec3::from_values(2.5, PILLAR_HALF_HEIGHT + STORY_HEIGHT, 0.25),
    )
    .max(sd_box(
        &p_wall_shifted,
        &vec3::from_values(1.75, PILLAR_HALF_HEIGHT + STORY_HEIGHT, 2.0),
    ));

    const BALCONY_HALF_LENGTH: VecFloat = 0.5 * 1.95 * PILLAR_SPACING;
    let p_shift_balconies = op_shift(
        p,
        &vec3::from_values(
            0.5 * 1.75 - 0.15,
            PILLAR_HALF_HEIGHT,
            2.0 * PILLAR_SPACING + SMALL_LEDGE_WIDTH + BALCONY_HALF_LENGTH + 1.15,
        ),
    );
    let p_repeated_balconies = op_repeat_finite(
        &p_shift_balconies,
        &vec3::from_values(1.0, STORY_HEIGHT, 1.0),
        &vec3::from_values(0.0, -HALF_STORY_COUNT, 0.0),
        &vec3::from_values(0.0, HALF_STORY_COUNT + 1.0, 0.0),
    );
    let balconies = sd_cromwell_balcony(
        &p_repeated_balconies,
        WINDOW_LEDGE_HEIGHT,
        BALCONY_HALF_LENGTH,
    )
    .max(sd_box(
        &op_rotate_y(
            &op_shift(&p_repeated_balconies, &vec3::from_values(0.0, 0.0, -1.25)),
            WALL_ANGLE,
        ),
        &vec3::from_values(3.5, STORY_HEIGHT, BALCONY_HALF_LENGTH - 0.4),
    ));

    let p_shift_side_balconies = op_shift(
        p,
        &vec3::from_values(0.0, PILLAR_HALF_HEIGHT, -2.0 * PILLAR_SPACING),
    );
    let p_repeated_side_balconies = op_repeat_finite(
        &p_shift_side_balconies,
        &vec3::from_values(1.0, STORY_HEIGHT, 1.0),
        &vec3::from_values(0.0, -HALF_STORY_COUNT, 0.0),
        &vec3::from_values(0.0, HALF_STORY_COUNT, 0.0),
    );
    let p_rotated_side_balconies = op_rotate_y(&p_repeated_side_balconies, PI * 0.5);
    let side_balconies = sd_cromwell_balcony(
        &p_rotated_side_balconies,
        WINDOW_LEDGE_HEIGHT,
        PILLAR_HALF_SIDE,
    );

    pillars
        .min(windows)
        .min(window_ledges)
        .min(small_ledges)
        .min(balcony_wall)
        .min(balconies)
        .min(side_balconies)
}

pub fn scene_cromwell_estate(p: &Vec3) -> VecFloat {
    let p_repeated = op_repeat_finite(
        p,
        &vec3::from_values(3.9, 1.0, 1.0),
        &vec3::from_values(-2.0, 0.0, 0.0),
        &vec3::from_values(1.0, 0.0, 0.0),
    );
    let sd_pillars = sd_stacked_pillar(&p_repeated);
    const SHIFT_SCALE: VecFloat = 1.15;
    let p_shifted = op_shift(
        p,
        &vec3::from_values(-16.0 * SHIFT_SCALE, 0.0, -16.5 * SHIFT_SCALE),
    );
    let sd_tower = sd_cromwell_tower(&p_shifted);
    sd_pillars.min(sd_tower)
}

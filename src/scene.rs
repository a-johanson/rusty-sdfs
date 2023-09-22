use std::f32::consts::PI;

use crate::vector::{vec2, vec3, Vec2, Vec3, VecFloat};

use crate::sdf::{
    op_elongate_y, op_elongate_z, op_onion, op_repeat_finite, op_repeat_xz, op_rotate_y,
    op_rotate_z, op_shift, op_smooth_difference, op_smooth_union, sd_box, sd_cylinder,
    sd_cylinder_rounded, sd_plane, sd_sphere, Material, SdfOutput, Sdf,
};

const TO_RAD: VecFloat = PI / 180.0;


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

    let material_planet = Material::new(&light, None);
    let (open_planet, _) = op_smooth_difference(planet, cutout, 1.0);
    SdfOutput::new(open_planet, material_planet)
}


fn sd_flower(p: &Vec3, cell_id: &Vec2, light: &Vec3) -> SdfOutput {
    fn hash(v: &Vec2, offset: VecFloat) -> VecFloat {
        ((v.0 + 113.0 * v.1 + offset).sin() * 43758.5453123)
            .fract()
            .abs()
    }
    const HASH_INC: VecFloat = 0.1;
    let x_jitter = 0.5 * (1.0 - 2.0 * hash(cell_id, 6.0 * HASH_INC));
    let z_jitter = 0.5 * (1.0 - 2.0 * hash(cell_id, 7.0 * HASH_INC));
    let sphere_radius = 0.45 + 0.55 * hash(cell_id, 0.0);
    let shell_radius = 1.1 * sphere_radius;
    let shell_thickness = 0.025 * sphere_radius;
    let opening_angle_xz = PI * (0.2 + 0.2 * hash(cell_id, 3.0 * HASH_INC));
    let opening_angle_y = PI * (0.2 + 0.1 * hash(cell_id, 4.0 * HASH_INC));
    let opening_distance = sphere_radius * (0.7 + 0.2 * hash(cell_id, 5.0 * HASH_INC));
    let opening_radius = shell_radius * (0.65 + 0.25 * hash(cell_id, 8.0 * HASH_INC));
    let shell_opening_k = 0.25 * sphere_radius;
    let shell_core_k = 0.1 * sphere_radius;
    let stem_height = 0.5 + sphere_radius * 0.7 * hash(cell_id, HASH_INC);
    let stem_radius = sphere_radius * (0.15 + 0.1 * hash(cell_id, 2.0 * HASH_INC));
    let stem_k = 0.9 * sphere_radius;

    let core_hsl = vec3::from_values(50.0f32.to_radians(), 1.0, 0.5);
    let material_core = Material::new(light, Some(&core_hsl));
    let material_shell = Material::new(light, None);

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
            stem_height,
        ),
        stem_radius,
    );

    let (bulb, bulb_t) = op_smooth_union(core, shell, shell_core_k);
    let material_flower = material_core.lerp(&material_shell, bulb_t);
    let (flower, _) = op_smooth_union(bulb, stem, stem_k);
    SdfOutput::new(flower, material_flower)
}

pub fn scene_meadow(p: &Vec3) -> SdfOutput {
    // let camera = vec3::from_values(5.0, 7.0, 5.0);
    // let look_at = vec3::from_values(0.9, 0.75, -4.0);
    // let up = vec3::from_values(0.0, 1.0, 0.0);
    let light = vec3::from_values(1.75e5, 3.5e5, 1.5e5);
    let cell_size = 2.75;

    let flowers = op_repeat_xz(
        |p: &Vec3, cell_id: &Vec2| sd_flower(p, cell_id, &light),
        p,
        &vec2::from_values(cell_size, cell_size),
    );

    let floor_deformation = 0.03
        * ((2.0 * PI * p.0 / cell_size).cos()
            + (2.0 * PI * p.1 / cell_size).cos()
            + 0.5 * (3.0 * 2.0 * PI * p.0 / cell_size).cos()
            + 0.5 * (2.0 * 2.0 * PI * p.1 / cell_size).cos());

    let material_floor = Material::new(&light, None);
    let floor = sd_plane(p, &vec3::from_values(0.0, 1.0, 0.0), floor_deformation);
    let (scene, scene_t) = op_smooth_union(floor, flowers.distance, 0.65);
    SdfOutput::new(scene, material_floor.lerp(&flowers.material, scene_t))
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

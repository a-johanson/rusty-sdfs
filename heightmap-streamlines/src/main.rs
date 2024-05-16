#![allow(dead_code)]

mod wave;

// use std::path::Path;
use std::f32::consts::PI;

use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use rusty_sdfs_lib::render_flow_field_streamlines;
use rusty_sdfs_lib::vec3;
use rusty_sdfs_lib::Material;
use rusty_sdfs_lib::PixelPropertyCanvas;
use rusty_sdfs_lib::RayMarcher;
use rusty_sdfs_lib::ReflectiveProperties;

use crate::wave::blob_heightmap;
use crate::wave::noise_heightmap;
use crate::wave::noisy_waves;

fn main() {
    const RNG_SEED: u64 = 62809543637;
    const WIDTH_IN_CM: f32 = 8.0;
    const HEIGHT_IN_CM: f32 = 6.0;
    const STROKE_WIDTH_IN_MM: f32 = 0.15;
    const D_SEP_MIN_IN_MM: f32 = 0.27;
    const D_SEP_MAX_IN_MM: f32 = 1.5;
    const D_TEST_FACTOR: f32 = 0.8;
    const D_STEP_IN_MM: f32 = 0.1;
    const MAX_DEPTH_STEP: f32 = 0.25;
    const MAX_ACCUM_ANGLE: f32 = 1.2 * PI;
    const MAX_STEPS: u32 = 450;
    const MIN_STEPS: u32 = 4;
    const SEED_BOX_SIZE_IN_MM: f32 = 2.0;
    const DPI: f32 = 150.0;

    const INCH_PER_CM: f32 = 1.0 / 2.54;
    const INCH_PER_MM: f32 = 0.1 / 2.54;
    const SEED_BOX_SIZE: u32 = (SEED_BOX_SIZE_IN_MM * INCH_PER_MM * DPI) as u32;
    const STROKE_WIDTH: f32 = STROKE_WIDTH_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MIN: f32 = D_SEP_MIN_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MAX: f32 = D_SEP_MAX_IN_MM * INCH_PER_MM * DPI;
    const D_STEP: f32 = D_STEP_IN_MM * INCH_PER_MM * DPI;
    let width = (WIDTH_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;

    let camera = vec3::from_values(0.5, 3.0, 2.0);
    let camera = vec3::scale(&camera, 3.0);
    let look_at = vec3::from_values(0.0, 0.0, -5.0);
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let fov = 40.0;
    const MAX_CHANGE_RATE: f32 = 2.0;
    let ray_marcher = RayMarcher::new(
        1.0 * 1.0 / (MAX_CHANGE_RATE * MAX_CHANGE_RATE + 1.0).sqrt(),
        &camera,
        &look_at,
        &up,
        fov,
        (width as f32) / (height as f32),
    );

    let rp = ReflectiveProperties::new(
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
    let material = Material::new(
        &vec3::from_values(1.0e5, 2.0e5, 5.0e5),
        Some(&rp),
        None,
        false,
        true
    );
    let pp_canvas = PixelPropertyCanvas::from_heightmap(&ray_marcher, &noisy_waves, &material, width, height, 0.0);
    let lightness_canvas = pp_canvas.lightness_to_skia_canvas();
    // lightness_canvas.save_png(Path::new("heightmap.png"));
    lightness_canvas.display_in_window("heightmap");

    // let mut rng = Xoshiro256PlusPlus::seed_from_u64(RNG_SEED);
    // let mut output_canvas = pp_canvas.bg_to_skia_canvas();
    // let streamline_color: [u8; 3] = [0, 0, 0];
    // render_flow_field_streamlines(
    //     &pp_canvas,
    //     &mut output_canvas,
    //     &mut rng,
    //     &streamline_color,
    //     STROKE_WIDTH,
    //     SEED_BOX_SIZE,
    //     D_SEP_MIN,
    //     D_SEP_MAX,
    //     D_TEST_FACTOR,
    //     D_STEP,
    //     MAX_DEPTH_STEP,
    //     MAX_ACCUM_ANGLE,
    //     MAX_STEPS,
    //     MIN_STEPS
    // );
    // output_canvas.display_in_window("streamlines");

}

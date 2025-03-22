#![allow(dead_code)]

mod scene;

use std::f32::consts::PI;
use std::path::Path;
use std::time::Instant;

use minifb::{MouseButton, MouseMode};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use rusty_sdfs_lib::Canvas;
use rusty_sdfs_lib::PixelPropertyCanvas;
use rusty_sdfs_lib::RayMarcher;
use rusty_sdfs_lib::{render_flow_field_streamlines, render_edges};
use rusty_sdfs_lib::vec3;
// use scene::SceneMeadow;
// use scene::SceneTrees;
use scene::ScenePillars;
use scene::SceneTrees;

fn main() {
    // TODO: put these parameters into config objects to be stored in the scene
    const RNG_SEED: u64 = 62809543637;
    const WIDTH_IN_CM: f32 = 13.0;
    const HEIGHT_IN_CM: f32 = 18.0;
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
    const DPI: f32 = 100.0;

    const INCH_PER_CM: f32 = 1.0 / 2.54;
    const INCH_PER_MM: f32 = 0.1 / 2.54;
    const SEED_BOX_SIZE: u32 = (SEED_BOX_SIZE_IN_MM * INCH_PER_MM * DPI) as u32;
    const STROKE_WIDTH: f32 = STROKE_WIDTH_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MIN: f32 = D_SEP_MIN_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MAX: f32 = D_SEP_MAX_IN_MM * INCH_PER_MM * DPI;
    const D_STEP: f32 = D_STEP_IN_MM * INCH_PER_MM * DPI;
    let width = (WIDTH_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;

    let scene = ScenePillars::new();
    let camera = scene.camera();
    let look_at = scene.look_at();
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let fov = scene.fov();
    const MAX_CHANGE_RATE: f32 = 2.0;
    let ray_marcher = RayMarcher::new(
        0.2,
        &camera,
        &look_at,
        &up,
        fov,
        (width as f32) / (height as f32),
    );

    let mut rng = Xoshiro256PlusPlus::seed_from_u64(RNG_SEED);

    println!(
        "Rendering on canvas of size {} px x {} px using a stroke width of {} px...",
        width, height, STROKE_WIDTH
    );
    println!(
        "Using a minimum separation of streamlines of {} px, a maximum of {} px, a test factor of {}, a step of {} px, and an initial seed box size of {} px...",
        D_SEP_MIN, D_SEP_MAX, D_TEST_FACTOR, D_STEP, SEED_BOX_SIZE
    );
    let start_instant = Instant::now();
    let pp_canvas = PixelPropertyCanvas::from_scene(&ray_marcher, &scene, width, height, 0.0);
    let duration_ldd = start_instant.elapsed();
    println!(
        "Finished raymarching the scene after {} seconds",
        duration_ldd.as_secs_f32()
    );

    let start_instant = Instant::now();
    let mut output_canvas = pp_canvas.bg_to_skia_canvas();
    let streamline_color = vec3::hsl_to_rgb_u8(&scene.hsl_streamlines());
    render_flow_field_streamlines(
        &pp_canvas,
        &mut output_canvas,
        &mut rng,
        &streamline_color,
        STROKE_WIDTH,
        SEED_BOX_SIZE,
        D_SEP_MIN,
        D_SEP_MAX,
        D_TEST_FACTOR,
        D_STEP,
        MAX_DEPTH_STEP,
        MAX_ACCUM_ANGLE,
        MAX_STEPS,
        MIN_STEPS
    );

    render_edges(
        &pp_canvas,
        &mut output_canvas,
        &streamline_color,
        STROKE_WIDTH,
    );


    let duraction_flow = start_instant.elapsed();
    println!(
        "Finished rendering the flowfield after {} seconds",
        duraction_flow.as_secs_f32()
    );

    println!("Outputting image(s) to disk/display...");
    // output_canvas.save_png(Path::new("trees.png"));
    // pp_canvas.to_file("trees.ppc").unwrap();
    output_canvas.display_in_window_with_event_handler("scene streamlines", |window| {
        if window.get_mouse_down(MouseButton::Left) {
            window.get_mouse_pos(MouseMode::Clamp).map(|mouse| {
                println!("Window Coordinates: ({}, {})", mouse.0, mouse.1);
                let screen_coordinates = output_canvas.to_screen_coordinates(mouse.0, mouse.1);
                let screen_direction = ray_marcher.screen_direction(&screen_coordinates);
                println!("({:e} + T * {:e}, {:e} + T * {:e}, {:e} + T * {:e})", ray_marcher.camera.0, screen_direction.0, ray_marcher.camera.1, screen_direction.1, ray_marcher.camera.2, screen_direction.2);
            });
        }
    });
    println!("Done");
}

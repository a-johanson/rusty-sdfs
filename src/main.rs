#![allow(dead_code)]

mod canvas;
mod grid;
mod ray_marcher;
mod scene;
mod sdf;
mod streamline;
mod vector;

use std::collections::VecDeque;
use std::f32::consts::PI;
use std::path::Path;
use std::time::Instant;

use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use canvas::PixelPropertyCanvas;
use ray_marcher::RayMarcher;
use scene::SceneSpikedSphere;
use streamline::{flow_field_streamline, streamline_d_sep_from_lightness, StreamlineRegistry};
use vector::{vec2, vec3, Vec2};

use crate::grid::on_jittered_grid;

fn main() {
    const RNG_SEED: u64 = 62809543637;
    const WIDTH_IN_CM: f32 = 10.0;
    const HEIGHT_IN_CM: f32 = 25.0;
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
    const DPI: f32 = 200.0;

    const INCH_PER_CM: f32 = 1.0 / 2.54;
    const INCH_PER_MM: f32 = 0.1 / 2.54;
    const SEED_BOX_SIZE: u32 = (SEED_BOX_SIZE_IN_MM * INCH_PER_MM * DPI) as u32;
    const STROKE_WIDTH: f32 = STROKE_WIDTH_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MIN: f32 = D_SEP_MIN_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MAX: f32 = D_SEP_MAX_IN_MM * INCH_PER_MM * DPI;
    const D_STEP: f32 = D_STEP_IN_MM * INCH_PER_MM * DPI;
    let width = (WIDTH_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;

    let scene = SceneSpikedSphere::new();
    let camera = scene.camera();
    let look_at = scene.look_at();
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let fov = scene.fov();
    let ray_marcher = RayMarcher::new(
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
    let mut streamline_registry = StreamlineRegistry::new(width, height, 0.5 * D_SEP_MAX);
    let mut streamline_queue: VecDeque<(u32, Vec<Vec2>)> = VecDeque::new();
    let streamline_color = vec3::hsl_to_rgb_u8(&scene.hsl_streamlines());

    on_jittered_grid(
        width as f32,
        height as f32,
        width / SEED_BOX_SIZE,
        height / SEED_BOX_SIZE,
        &mut rng,
        |seed_x, seed_y| {
            let seed_streamline_option = flow_field_streamline(
                &pp_canvas,
                &streamline_registry,
                0,
                &vec2::from_values(seed_x, seed_y),
                D_SEP_MIN,
                D_SEP_MAX,
                D_TEST_FACTOR,
                D_STEP,
                MAX_DEPTH_STEP,
                MAX_ACCUM_ANGLE,
                MAX_STEPS,
                MIN_STEPS,
            );
            if seed_streamline_option.is_some() {
                let seed_streamline = seed_streamline_option.unwrap();
                let seed_streamline_id = streamline_registry.add_streamline(&seed_streamline);
                output_canvas.stroke_line_segments(
                    &seed_streamline,
                    STROKE_WIDTH,
                    streamline_color,
                );
                streamline_queue.push_back((seed_streamline_id, seed_streamline));
            }
        },
    );

    while !streamline_queue.is_empty() {
        let (streamline_id, streamline) = streamline_queue.pop_front().unwrap();
        for (p, &sign) in streamline.iter().zip([-1.0f32, 1.0f32].iter().cycle()) {
            let pixel = pp_canvas.pixel_value(p.0, p.1).unwrap();
            let d_sep = streamline_d_sep_from_lightness(D_SEP_MIN, D_SEP_MAX, pixel.lightness);
            let new_seed = vec2::scale_and_add(
                p,
                &vec2::polar_angle_to_unit_vector(pixel.direction + 0.5 * PI),
                sign * d_sep,
            );
            let new_streamline = flow_field_streamline(
                &pp_canvas,
                &streamline_registry,
                streamline_id,
                &new_seed,
                D_SEP_MIN,
                D_SEP_MAX,
                D_TEST_FACTOR,
                D_STEP,
                MAX_DEPTH_STEP,
                MAX_ACCUM_ANGLE,
                MAX_STEPS,
                MIN_STEPS,
            );
            if new_streamline.is_some() {
                let sl = new_streamline.unwrap();
                let streamline_id = streamline_registry.add_streamline(&sl);
                output_canvas.stroke_line_segments(&sl, STROKE_WIDTH, streamline_color);
                streamline_queue.push_back((streamline_id, sl));
            }
        }
    }

    let duraction_flow = start_instant.elapsed();
    println!(
        "Finished rendering the flowfield after {} seconds",
        duraction_flow.as_secs_f32()
    );

    println!("Saving image(s) to disk...");
    output_canvas.save_png(Path::new("output.png"));
    println!("Done");
}

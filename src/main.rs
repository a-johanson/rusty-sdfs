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

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use canvas::{Canvas, LightDirectionDistanceCanvas, SkiaCanvas};
use ray_marcher::RayMarcher;
use scene::scene_spheres;
use streamline::{flow_field_streamline, streamline_d_sep_from_lightness, StreamlineRegistry};
use vector::{vec2, vec3, Vec2};

fn main() {
    const RNG_SEED: u64 = 6280954363;
    const WIDTH_IN_CM: f32 = 10.0;
    const HEIGHT_IN_CM: f32 = 10.0;
    const STROKE_WIDTH_IN_MM: f32 = 0.15;
    const D_SEP_MIN_IN_MM: f32 = 0.27;
    const D_SEP_MAX_IN_MM: f32 = 1.5;
    const D_TEST_FACTOR: f32 = 0.8;
    const D_STEP_IN_MM: f32 = 0.1;
    const MAX_DEPTH_STEP: f32 = 0.25;
    const MAX_ACCUM_ANGLE: f32 = 1.0 * PI;
    const MAX_STEPS: u32 = 450;
    const MIN_STEPS: u32 = 4;
    const SEED_STREAMLINES: u32 = 35;
    const DPI: f32 = 300.0;

    const INCH_PER_CM: f32 = 1.0 / 2.54;
    const INCH_PER_MM: f32 = 0.1 / 2.54;
    const STROKE_WIDTH: f32 = STROKE_WIDTH_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MIN: f32 = D_SEP_MIN_IN_MM * INCH_PER_MM * DPI;
    const D_SEP_MAX: f32 = D_SEP_MAX_IN_MM * INCH_PER_MM * DPI;
    const D_STEP: f32 = D_STEP_IN_MM * INCH_PER_MM * DPI;
    let width = (WIDTH_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let mut streamline_canvas = SkiaCanvas::new(width, height);

    let camera = vec3::from_values(0.0, 0.0, 5.0);
    let look_at = vec3::from_values(0.0, -0.5, 0.0);
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let ray_marcher = RayMarcher::new(
        &camera,
        &look_at,
        &up,
        50.0,
        streamline_canvas.aspect_ratio(),
    );
    let light_point_source = vec3::from_values(1.0, 0.0, 7.0);

    let mut rng = Xoshiro256PlusPlus::seed_from_u64(RNG_SEED);

    println!(
        "Rendering on canvas of size {} px x {} px using a stroke width of {} px...",
        width, height, STROKE_WIDTH
    );
    println!(
        "Using a minimum separation of streamlines of {} px, a maximum of {} px, a test factor of {}, and a step of {} px...",
        D_SEP_MIN, D_SEP_MAX, D_TEST_FACTOR, D_STEP
    );
    let start_instant = Instant::now();
    let ldd_canvas = LightDirectionDistanceCanvas::from_sdf_scene(
        &ray_marcher,
        scene_spheres,
        width,
        height,
        &light_point_source,
        0.0,
    );
    let duration_ldd = start_instant.elapsed();
    println!(
        "Finished raymarching the scene after {} seconds",
        duration_ldd.as_secs_f32()
    );

    let start_instant = Instant::now();
    let mut lightness_canvas = ldd_canvas.lightness_to_skia_canvas();
    let mut streamline_registry = StreamlineRegistry::new(width, height, 0.5 * D_SEP_MAX);
    let mut streamline_queue: VecDeque<(u32, Vec<Vec2>)> = VecDeque::new();
    const DEBUG_LINE_COLOR: [u8; 3] = [2, 70, 217];
    for _ in 0..SEED_STREAMLINES {
        let seed_streamline_option = flow_field_streamline(
            &ldd_canvas,
            &streamline_registry,
            0,
            &vec2::from_values(
                (0.99 * rng.gen::<f32>() + 0.005) * width as f32,
                (0.99 * rng.gen::<f32>() + 0.005) * height as f32,
            ),
            D_SEP_MIN,
            D_SEP_MAX,
            D_TEST_FACTOR,
            D_STEP,
            MAX_DEPTH_STEP,
            MAX_ACCUM_ANGLE,
            MAX_STEPS,
            MIN_STEPS,
        );
        if seed_streamline_option.is_none() {
            continue;
        }
        let seed_streamline = seed_streamline_option.unwrap();
        let seed_streamline_id = streamline_registry.add_streamline(&seed_streamline);
        lightness_canvas.stroke_line_segments(&seed_streamline, STROKE_WIDTH, DEBUG_LINE_COLOR);
        streamline_canvas.stroke_line_segments(&seed_streamline, STROKE_WIDTH, [0, 0, 0]);
        streamline_queue.push_back((seed_streamline_id, seed_streamline));
    }

    while !streamline_queue.is_empty() {
        let (streamline_id, streamline) = streamline_queue.pop_front().unwrap();
        for (p, &sign) in streamline.iter().zip([-1.0f32, 1.0f32].iter().cycle()) {
            let (lightness, direction, _) = ldd_canvas.pixel_value(p.0, p.1).unwrap();
            let d_sep = streamline_d_sep_from_lightness(D_SEP_MIN, D_SEP_MAX, lightness);
            let new_seed = vec2::scale_and_add(
                p,
                &vec2::polar_angle_to_unit_vector(direction + 0.5 * PI),
                sign * d_sep,
            );
            let new_streamline = flow_field_streamline(
                &ldd_canvas,
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
                lightness_canvas.stroke_line_segments(&sl, STROKE_WIDTH, DEBUG_LINE_COLOR);
                streamline_canvas.stroke_line_segments(&sl, STROKE_WIDTH, [0, 0, 0]);
                streamline_queue.push_back((streamline_id, sl));
            }
        }
    }

    let duraction_flow = start_instant.elapsed();
    println!(
        "Finished rendering flowfield after {} seconds",
        duraction_flow.as_secs_f32()
    );

    println!("Saving image(s) to disk...");
    streamline_canvas.save_png(Path::new("out_streamline.png"));
    lightness_canvas.save_png(Path::new("out_lightness.png"));
    let direction_canvas = ldd_canvas.direction_to_skia_canvas();
    direction_canvas.save_png(Path::new("out_direction.png"));
    let distance_canvas = ldd_canvas.distance_to_skia_canvas();
    distance_canvas.save_png(Path::new("out_distance.png"));
    println!("Done");
}

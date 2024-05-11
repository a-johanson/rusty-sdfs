use std::collections::VecDeque;
use std::f32::consts::PI;

use rand::RngCore;

use crate::canvas::{Canvas, PixelPropertyCanvas, SkiaCanvas};
use crate::grid::on_jittered_grid;
use crate::streamline::{StreamlineRegistry, flow_field_streamline, streamline_d_sep_from_lightness};
use crate::vector::{vec2, Vec2};

pub fn render_flow_field_streamlines(
    input_canvas: &PixelPropertyCanvas,
    output_canvas: &mut SkiaCanvas,
    rng: &mut dyn RngCore,
    streamline_color: &[u8; 3],
    stroke_width: f32,
    seed_box_size: u32,
    d_sep_min: f32,
    d_sep_max: f32,
    d_test_factor: f32,
    d_step: f32,
    max_depth_step: f32,
    max_accum_angle: f32,
    max_steps: u32,
    min_steps: u32
) {
    let width = input_canvas.width();
    let height = input_canvas.height();
    let mut streamline_registry = StreamlineRegistry::new(width, height, 0.5 * d_sep_max);
    let mut streamline_queue: VecDeque<(u32, Vec<Vec2>)> = VecDeque::new();

    on_jittered_grid(
        width as f32,
        height as f32,
        width / seed_box_size,
        height / seed_box_size,
        rng,
        |seed_x, seed_y| {
            let seed_streamline_option = flow_field_streamline(
                input_canvas,
                &streamline_registry,
                0,
                &vec2::from_values(seed_x, seed_y),
                d_sep_min,
                d_sep_max,
                d_test_factor,
                d_step,
                max_depth_step,
                max_accum_angle,
                max_steps,
                min_steps,
            );
            if seed_streamline_option.is_some() {
                let seed_streamline = seed_streamline_option.unwrap();
                let seed_streamline_id = streamline_registry.add_streamline(&seed_streamline);
                let path = SkiaCanvas::linear_path(&seed_streamline);
                if path.is_some() {
                    output_canvas.stroke_path(
                        &path.unwrap(),
                        stroke_width,
                        streamline_color,
                    );
                }
                streamline_queue.push_back((seed_streamline_id, seed_streamline));
            }
        },
    );

    while !streamline_queue.is_empty() {
        let (streamline_id, streamline) = streamline_queue.pop_front().unwrap();
        for (p, &sign) in streamline.iter().zip([-1.0f32, 1.0f32].iter().cycle()) {
            let pixel = input_canvas.pixel_value(p.0, p.1).unwrap();
            let d_sep = streamline_d_sep_from_lightness(d_sep_min, d_sep_max, pixel.lightness);
            let new_seed = vec2::scale_and_add(
                p,
                &vec2::polar_angle_to_unit_vector(pixel.direction + 0.5 * PI),
                sign * d_sep,
            );
            let new_streamline = flow_field_streamline(
                input_canvas,
                &streamline_registry,
                streamline_id,
                &new_seed,
                d_sep_min,
                d_sep_max,
                d_test_factor,
                d_step,
                max_depth_step,
                max_accum_angle,
                max_steps,
                min_steps,
            );
            if new_streamline.is_some() {
                let sl = new_streamline.unwrap();
                let streamline_id = streamline_registry.add_streamline(&sl);
                let path = SkiaCanvas::linear_path(&sl);
                if path.is_some() {
                    output_canvas.stroke_path(&path.unwrap(), stroke_width, streamline_color);
                }
                streamline_queue.push_back((streamline_id, sl));
            }
        }
    }
}

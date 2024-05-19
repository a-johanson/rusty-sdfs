use std::collections::VecDeque;
use std::f32::consts::PI;

use rand::RngCore;

use crate::canvas::{Canvas, PixelPropertyCanvas, SkiaCanvas};
use crate::grid::on_jittered_grid;
use crate::streamline::{StreamlineRegistry, flow_field_streamline, streamline_d_sep_from_lightness};
use crate::vector::{vec2, Vec2};
use crate::{LinearGradient, VecFloat};


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

pub struct DomainRegion {
    pub near_a: Vec2,
    pub near_b: Vec2,
    pub far_a: Vec2,
    pub far_b: Vec2,
}

impl DomainRegion {
    pub fn new(camera: &Vec2, look_at: &Vec2, fov_degrees: VecFloat, near: VecFloat, far: VecFloat) -> Self {
        let dir = vec2::normalize_inplace(vec2::sub(look_at, camera));
        let tan_fov = (0.5 * PI / 180.0 * fov_degrees).tan();
        let d_near = near * tan_fov;
        let d_far = far * tan_fov;
        println!("length near: {}, length far: {}", 2.0 * d_near, 2.0 * d_far);
        let dir_ortho_ccw = vec2::from_values(-dir.1, dir.0);
        let m_near = vec2::scale_and_add(camera, &dir, near);
        let m_far = vec2::scale_and_add(camera, &dir, far);
        Self {
            near_a: vec2::scale_and_add(&m_near, &dir_ortho_ccw, d_near),
            near_b: vec2::scale_and_add(&m_near, &dir_ortho_ccw, -d_near),
            far_a: vec2::scale_and_add(&m_far, &dir_ortho_ccw, d_far),
            far_b: vec2::scale_and_add(&m_far, &dir_ortho_ccw, -d_far)
        }
    }

    pub fn lerp(&self, t_ab: VecFloat, t_nearfar: VecFloat) -> Vec2 {
        let nf_a = vec2::lerp(&self.near_a, &self.far_a, t_nearfar);
        let nf_b = vec2::lerp(&self.near_b, &self.far_b, t_nearfar);
        vec2::lerp(&nf_a, &nf_b, t_ab)
    }
}

pub fn render_heightmap_streamlines<F>(
    output_canvas: &mut SkiaCanvas,
    domain_region: &DomainRegion,
    line_count: u32,
    buffer_count_near: u32,
    buffer_count_far: u32,
    segment_count: u32,
    line_width: f32,
    line_rgb: &[u8; 3],
    fill_gradient: &LinearGradient,
    heightmap: F,
)
where
    F: Fn(&Vec2, &Vec2, &Vec2) -> f32, // args: uv_domain, t_domain, t_screen
{
    let width = output_canvas.width() as VecFloat;
    let height = output_canvas.height() as VecFloat;
    let margin = 2.0 * line_width + 1.0;

    let line_idx_from = -(buffer_count_near as i32);
    let line_idx_to = (line_count + buffer_count_far) as i32;
    for line_idx in (line_idx_from..line_idx_to).rev() {
        let t_nearfar = line_idx as VecFloat / ((line_count - 1) as VecFloat);
        let points: Vec<_> = (0..=segment_count).map(|seg_idx| {
                let t_ab = seg_idx as f32 / (segment_count as f32);
                let uv_domain = domain_region.lerp(t_ab, t_nearfar);
                let t_domain = vec2::from_values(t_ab, t_nearfar);
                const LN_BASE: VecFloat = 0.7;
                const EXP_MINUS_LN_BASE: VecFloat = 0.4965853037914095147;
                let t_screen = vec2::from_values(
                    t_ab,
                    // f32::exp(-t_nearfar * LN_BASE)
                    f32::exp(-t_nearfar * LN_BASE)
                );
                let h = heightmap(&uv_domain, &t_domain, &t_screen);
                vec2::from_values(
                    width * t_screen.0,
                    height * (t_screen.1 - h)
                )
            })
            .collect();

        let first_point_y = points[0].1;
        let last_point_y = points.last().unwrap().1;

        let points_prepend = [
            vec2::from_values(-margin, height + margin),
            vec2::from_values(-margin, first_point_y),
        ];
        let points_append = [
            vec2::from_values(width + margin, last_point_y),
            vec2::from_values(width + margin, height + margin),
        ];
        let points: Vec<_> = points_prepend.iter().copied()
            .chain(points.iter().copied())
            .chain(points_append)
            .collect();
        let path = SkiaCanvas::closed_linear_path(&points).unwrap();
        output_canvas.fill_path(&path, &fill_gradient.rgb(1.0 - 0.5 * (first_point_y + last_point_y) / height));
        output_canvas.stroke_path(&path, line_width, line_rgb);
    }
}

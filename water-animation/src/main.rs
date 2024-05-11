use std::f32::consts::PI;
use std::time::Duration;
use std::vec;

use minifb::WindowOptions;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;
use rusty_sdfs_lib::noise_2d;
use rusty_sdfs_lib::vec2;
use rusty_sdfs_lib::Animation;
use rusty_sdfs_lib::SkiaCanvas;
use rusty_sdfs_lib::Vec2;
use rusty_sdfs_lib::VecFloat;

fn main() {
    let mut wave_animation = WaveAnimation::new();
    wave_animation.play("waves", WindowOptions::default());
}

struct WaveAnimation {
    centroids: Vec<Vec2>,
    v: Vec<Vec2>,
    _rng: Xoshiro256StarStar,
    noise_x: Vec<f32>,
    noise_y: Vec<f32>,
}

impl WaveAnimation {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    const FPS: f32 =  60.0;
    fn new() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(0x9C61_EA21_046B_F751);
        const CENTROID_COUNT: usize = 50;
        const MAX_FLOW_SPEED: f32 = 1.0;
        let centroids: Vec<_> = (0..CENTROID_COUNT).map(|_| vec2::from_values(rng.gen_range(0.0..1.0) * Self::WIDTH as f32, rng.gen_range(0.0..1.0) * Self::HEIGHT as f32)).collect();
        let v: Vec<_> = (0..CENTROID_COUNT).map(|_| vec2::from_values(rng.gen_range(0.1..MAX_FLOW_SPEED), rng.gen_range(0.1..((Self::HEIGHT as f32 / Self::WIDTH as f32) * MAX_FLOW_SPEED)))).collect();
        let mut noise_x= vec![0.0f32; Self::WIDTH as usize * Self::HEIGHT as usize];
        let mut noise_y = vec![0.0f32; Self::WIDTH as usize * Self::HEIGHT as usize];
        for iy in 0..(Self::HEIGHT as usize) {
            let yf = iy as f32;
            for ix in 0..(Self::WIDTH as usize) {
                let idx = iy * (Self::WIDTH as usize) + ix;
                let xf = ix as f32;
                const NOISE_INPUT_SCALE: VecFloat = 0.025;
                const NOISE_SCALE: VecFloat = 10.0;
                const NOISE_OCTAVES: u32 = 4;
                const YX_OFFSET: VecFloat = 1000.0;
                const YY_OFFSET: VecFloat = 889.0;
                noise_x[idx] = NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * xf, NOISE_INPUT_SCALE * yf, NOISE_OCTAVES);
                noise_y[idx] = NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * xf + YX_OFFSET, NOISE_INPUT_SCALE * yf + YY_OFFSET, NOISE_OCTAVES);
            }
        }

        Self {
            centroids,
            v,
            _rng: rng,
            noise_x,
            noise_y,
        }
    }
}

impl Animation for WaveAnimation {
    fn width(&self) -> u32 {
        Self::WIDTH
    }

    fn height(&self) -> u32 {
        Self::HEIGHT
    }

    fn frame_duration(&self) -> Duration {
        Duration::from_micros(1_000_000 / Self::FPS as u64)
    }

    fn render_frame(&mut self) -> Vec<u32> {
        for (ic, c) in self.centroids.iter_mut().enumerate() {
            c.0 = (c.0 + self.v[ic].0) % Self::WIDTH as f32;
            c.1 = (c.1 + self.v[ic].1) % Self::HEIGHT as f32;
        }

        let mut canvas = SkiaCanvas::new(Self::WIDTH, Self::HEIGHT);
        canvas.fill(&[0, 230, 255]);

        for (ic, c) in self.centroids.iter().enumerate() {
            const RAY_COUNT: usize = 25;
            const RAY_ANGLE: VecFloat = 2.0 * PI / (RAY_COUNT as VecFloat);
            const RAY_MAX_ITER: u32 = 300;
            const RAY_INCR: VecFloat = 1.0;
            const BORDER_THICKNESS: VecFloat = 6.0;

            let ray_endpoints: Vec<_> = (0..RAY_COUNT).map(|ir| {
                let angle = RAY_ANGLE * ir as VecFloat;
                let dir = vec2::polar_angle_to_unit_vector(angle);
                let mut len = 0.0;
                for _ in 0..RAY_MAX_ITER {
                    len += RAY_INCR;
                    let p = vec2::scale_and_add(c, &dir, len);
                    let len_squared = len * len;
                    let mut other_centroids = self.centroids.iter()
                        .enumerate()
                        .filter(|(jc, _)| *jc != ic);
                    let is_no_other_centroid_closer = other_centroids.all(|(_, c_other)| {
                        let dist_squared = vec2::len_squared(&vec2::sub(c_other, &p));
                        dist_squared > len_squared
                    });
                    if !is_no_other_centroid_closer {
                        break;
                    }
                }
                len = (len - BORDER_THICKNESS).max(RAY_INCR);
                vec2::scale_and_add(c, &dir, len)
            }).collect();

            let (ray_left_ctrl, ray_right_ctrl): (Vec<_>, Vec<_>) = ray_endpoints.iter()
                .zip(ray_endpoints.iter().cycle().skip(ray_endpoints.len() - 1))
                .zip(ray_endpoints.iter().cycle().skip(1))
                .map(|((p, prev), next)| {
                    let dir = vec2::normalize_inplace(vec2::sub(next, prev));
                    let len = vec2::len(&vec2::sub(p, c));
                    let dist = PI * len / (RAY_COUNT as VecFloat);
                    let left_ctrl_point = vec2::scale_and_add(p, &dir, -dist);
                    let right_ctrl_point = vec2::scale_and_add(p, &dir, dist);
                    (left_ctrl_point, right_ctrl_point)
                })
                .unzip();

            let path = SkiaCanvas::closed_cubic_curve_path(&ray_endpoints, &ray_left_ctrl, &ray_right_ctrl).unwrap();
            canvas.fill_path(&path, &[10, 140, 255]);
            canvas.stroke_path(&path, 3.0, &[50, 175, 255]);
        }

        let mut noisy_canvas = SkiaCanvas::new(Self::WIDTH, Self::HEIGHT);
        noisy_canvas.iter_mut_rgba_with_coordinates(|x, y, rgba| {
            let xf = x as f32;
            let yf = y as f32;
            let idx = y as usize * Self::WIDTH as usize + x as usize;
            let x_shift = self.noise_x[idx];
            let y_shift = self.noise_y[idx];
            let color = canvas.sample_bilinear(xf + x_shift, yf + y_shift);
            rgba[0] = color.red();
            rgba[1] = color.green();
            rgba[2] = color.blue();
            rgba[3] = color.alpha();
        });
        // noisy_canvas.fill_points(&self.centroids, 5.0, &[0, 100, 255]);

        noisy_canvas.to_u32_rgb()
    }
}

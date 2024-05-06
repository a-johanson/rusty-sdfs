use std::time::Duration;

use minifb::WindowOptions;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;
use rusty_sdfs_lib::vec2;
use rusty_sdfs_lib::Animation;
use rusty_sdfs_lib::SkiaCanvas;
use rusty_sdfs_lib::Vec2;

fn main() {
    let mut wave_animation = WaveAnimation::new();
    wave_animation.play("waves", WindowOptions::default());
}

struct WaveAnimation {
    centroids: Vec<Vec2>,
    rng: Xoshiro256StarStar,
    canvas: SkiaCanvas,
}

impl WaveAnimation {
    const WIDTH: u32 = 280;
    const HEIGHT: u32 = 240;
    const FPS: f32 =  60.0;
    fn new() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(0x9C63_EA21_046B_F751);
        let centroids: Vec<_> = (0..10).map(|_| vec2::from_values(rng.gen_range(0.0..1.0) * Self::WIDTH as f32, rng.gen_range(0.0..1.0) * Self::HEIGHT as f32)).collect();
        let canvas = SkiaCanvas::new(Self::WIDTH, Self::HEIGHT);
        Self {
            centroids,
            rng,
            canvas,
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
        for c in self.centroids.iter_mut() {
            c.0 = (c.0 + 0.5) % Self::WIDTH as f32;
            c.1 = (c.1 + 0.25) % Self::HEIGHT as f32;
        }

        self.canvas.fill(&[255, 255, 255]);
        self.canvas.fill_points(&self.centroids, 5.0, &[255, 0, 0]);
        self.canvas.to_u32_rgb()
    }
}

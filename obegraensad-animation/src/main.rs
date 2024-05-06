use std::time::Duration;

use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128StarStar;
use minifb::{Scale, WindowOptions};
use rusty_sdfs_lib::Animation;
use rusty_sdfs_lib::SkiaCanvas;

fn main() {
    let mut falling_leaves = FallingLeaves::new();
    falling_leaves.play("OBEGRAENSAD", WindowOptions {
        scale: Scale::X16,
        ..WindowOptions::default()
    });
}

#[derive(Clone, Copy)]
struct Leaf {
    x: u8,
    y: u8,
}

impl Leaf {
    const fn new() -> Self {
        Self { x: 0, y: 0xFF }
    }

    fn is_active(&self) -> bool {
        self.y < FallingLeaves::DISPLAY_SIZE as u8
    }

    fn init(&mut self, r: u32) {
        self.x = (r & 0xF) as u8;
        self.y = 0;
    }

    fn step(&mut self, r: u32) {
        let t = (r & 0b111) as u8;
        match t {
            0..=4 => self.x += 1, // 5/8 chance to go right
            5 => self.x -= 1,     // 1/8 chance to go left
            _ => (),              // 2/8 chance to not move horizontally
        }
        self.x &= 0x0F;
        self.y += 1;
    }
}

const MAX_LEAVES: usize = 10;

pub struct FallingLeaves {
    rng: Xoshiro128StarStar,
    leaves: [Leaf; MAX_LEAVES],
    x_prev_frame: Option<u8>,
    canvas: SkiaCanvas,
}

impl FallingLeaves {
    const DISPLAY_SIZE: u32 = 16;
    const X_INCR: u8 = 5; // must not share factors with DISPLAY_SIZE

    pub fn new() -> Self {
        Self {
            rng: Xoshiro128StarStar::seed_from_u64(0x63AF_2BA8_046C_E751),
            leaves: [Leaf::new(); MAX_LEAVES],
            x_prev_frame: Some(Self::DISPLAY_SIZE as u8 - Self::X_INCR),
            canvas: SkiaCanvas::new(Self::DISPLAY_SIZE, Self::DISPLAY_SIZE),
        }
    }

    fn set_canvas_pixel(canvas: &mut SkiaCanvas, x: u8, y: u8) {
        canvas.fill_rect(x as f32, y as f32, 1.0, 1.0, &[255, 255, 255]);
    }
}

impl Animation for FallingLeaves {
    fn width(&self) -> u32 {
        Self::DISPLAY_SIZE
    }

    fn height(&self) -> u32 {
        Self::DISPLAY_SIZE
    }

    fn frame_duration(&self) -> Duration {
        Duration::from_millis(400)
    }

    fn render_frame(&mut self) -> Vec<u32> {
        self.canvas.fill(&[0, 0, 0]);

        // Move and draw all existing leaves
        for leaf in self.leaves.iter_mut() {
            if leaf.is_active() {
                leaf.step(self.rng.next_u32());
                Self::set_canvas_pixel(&mut self.canvas, leaf.x, leaf.y);
            }
        }

        // Spawn and draw new leaf in 1/2 of cases
        if (self.rng.next_u32() & 0b1) == 0 {
            for leaf in self.leaves.iter_mut() {
                if !leaf.is_active() {
                    let r = if self.x_prev_frame.is_none() {
                        self.rng.next_u32()
                    } else {
                        ((self.x_prev_frame.unwrap() + Self::X_INCR) & 0xF) as u32
                    };
                    leaf.init(r);
                    self.x_prev_frame = Some(leaf.x);
                    Self::set_canvas_pixel(&mut self.canvas, leaf.x, leaf.y);
                    break;
                }
            }
        } else {
            self.x_prev_frame = None;
        }

        self.canvas.to_u32_rgb()
    }
}

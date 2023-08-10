use std::path::Path;

use crate::vector::{Vec2, vec2};

use tiny_skia::{Pixmap, Transform, PathBuilder, Paint, Stroke, Color, LineCap, LineJoin, Rect};


pub trait Canvas {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn aspect_ratio(&self) -> f32 {
        (self.width() as f32) / (self.height() as f32)
    }

    fn to_screen_coordinates(&self, x: f32, y: f32) -> Vec2 {
        vec2::from_values(
             2.0 * (x / (self.width() as f32)  - 0.5),
            -2.0 * (y / (self.height() as f32) - 0.5),
        )
    }

    fn to_canvas_coordinates(&self, screen_coordinates: &Vec2) -> Vec2 {
        vec2::from_values(
            0.5 * ( screen_coordinates.0 + 1.0) * (self.width() as f32),
            0.5 * (-screen_coordinates.1 + 1.0) * (self.height() as f32)
        )
     }
}

pub struct SkiaCanvas {
    pixmap: Pixmap,
}

impl Canvas for SkiaCanvas {
    fn width(&self) -> u32 {
        self.pixmap.width()
    }

    fn height(&self) -> u32 {
        self.pixmap.height()
    }
}

impl SkiaCanvas {
    pub fn new(width: u32, height: u32) -> SkiaCanvas {
        let mut pixmap = Pixmap::new(width, height).unwrap();
        pixmap.fill(Color::from_rgba8(255, 255, 255, 255));
        SkiaCanvas {
            pixmap,
        }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, rgb: [u8; 3], a: u8) {
        let rect = Rect::from_xywh(x, y, w, h).unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], a);
        paint.anti_alias = true;

        let transform = Transform::identity();
        self.pixmap.fill_rect(rect, &paint, transform, None);
    }

    pub fn stroke_line_segments(&mut self, points: &[Vec2], width: f32, rgb: [u8; 3]) {
        if points.len() <= 1 {
            return;
        }

        let mut pb = PathBuilder::new();
        let head = points[0];
        let tail = &points[1..];
        pb.move_to(head.0, head.1);
        for p in tail {
            pb.line_to(p.0, p.1);
        }
        let path = pb.finish().unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], 255);
        paint.anti_alias = true;

        let mut stroke = Stroke::default();
        stroke.width = width;
        stroke.line_cap = LineCap::Round;
        stroke.line_join = LineJoin::Round;

        let transform = Transform::identity();
        self.pixmap.stroke_path(&path, &paint, &stroke, transform, None);
    }

    pub fn save_png(&self, path: &Path) {
        self.pixmap.save_png(path).unwrap();
    }
}

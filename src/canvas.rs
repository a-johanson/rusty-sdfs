use std::path::Path;

use gl_matrix::common::Vec2;
use tiny_skia::{Pixmap, Transform, PathBuilder, Paint, Stroke, Color, LineCap, LineJoin, Rect};


pub struct Canvas {
    pixmap: Pixmap,
}

impl Canvas {
    pub fn new(dim_x: u32, dim_y: u32) -> Canvas {
        let mut pixmap = Pixmap::new(dim_x, dim_y).unwrap();
        pixmap.fill(Color::from_rgba8(255, 255, 255, 255));
        Canvas {
            pixmap,
        }
    }

    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    pub fn aspect_ratio(&self) -> f32 {
        (self.width() as f32) / (self.height() as f32)
    }

    pub fn to_screen_coordinates(&self, x: f32, y: f32) -> Vec2 {
        [ 2.0 * (x / (self.width() as f32)  - 0.5),
         -2.0 * (y / (self.height() as f32) - 0.5),]
     }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, l: u8) {
        let rect = Rect::from_xywh(x, y, w, h).unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(l, l, l, 255);
        paint.anti_alias = true;

        let transform = Transform::identity();
        self.pixmap.fill_rect(rect, &paint, transform, None);
    }

    pub fn stroke_line_segments(&mut self, points: &[Vec2]) {
        if points.len() <= 1 {
            return;
        }

        let mut pb = PathBuilder::new();
        let head = points[0];
        let tail = &points[1..];
        pb.move_to(head[0], head[1]);
        for p in tail {
            pb.line_to(p[0], p[1]);
        }
        let path = pb.finish().unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = true;

        let mut stroke = Stroke::default();
        stroke.width = 3.0;
        stroke.line_cap = LineCap::Round;
        stroke.line_join = LineJoin::Round;

        let transform = Transform::identity();
        self.pixmap.stroke_path(&path, &paint, &stroke, transform, None);
    }

    pub fn save_png(&self, path: &Path) {
        self.pixmap.save_png(path).unwrap();
    }
}

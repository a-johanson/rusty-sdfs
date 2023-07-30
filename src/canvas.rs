use std::path::Path;

use tiny_skia::{Pixmap, Transform, PathBuilder, Paint, Stroke, Color};


pub struct Canvas {
    pub dim_x: u32,
    pub dim_y: u32,
    pixmap: Pixmap,
}

impl Canvas {
    pub fn new(dim_x: u32, dim_y: u32) -> Canvas {
        let mut pixmap = Pixmap::new(dim_x, dim_y).unwrap();
        pixmap.fill(Color::from_rgba8(255, 255, 255, 255));
        Canvas {
            dim_x,
            dim_y,
            pixmap,
        }
    }

    pub fn stroke_line_segments(&mut self) {
        let mut pb = PathBuilder::new();
        pb.move_to(50.0, 100.0);
        pb.line_to(130.0, 20.0);
        let path = pb.finish().unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = true;

        let stroke = Stroke::default();

        let transform = Transform::identity();
        self.pixmap.stroke_path(&path, &paint, &stroke, transform, None);
    }

    pub fn save_png(&self, path: &Path) {
        self.pixmap.save_png(path).unwrap();
    }
}

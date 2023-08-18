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
pub struct LightDirectionDistanceCanvas {
    data: Vec<[f32; 3]>,
    width: u32,
    height: u32,
}

impl Canvas for LightDirectionDistanceCanvas {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl LightDirectionDistanceCanvas {
    pub fn new(width: u32, height: u32) -> LightDirectionDistanceCanvas {
        let data_length = (width as usize) * (height as usize);
        let data = vec![[f32::NAN; 3]; data_length];
        LightDirectionDistanceCanvas {
            data,
            width,
            height,
        }
    }

    fn pixel_index(&self, x: u32, y: u32) -> usize {
        (self.width as usize) * (y as usize) + (x as usize)
    }

    pub fn fill(&mut self, lightness: f32, direction: f32, distance: f32) {
        for v in self.data.iter_mut() {
            *v = [lightness, direction, distance];
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, lightness: f32, direction: f32, distance: f32) {
        let idx = self.pixel_index(x, y);
        let v = self.data.get_mut(idx).unwrap();
        *v = [lightness, direction, distance];
    }

    pub fn pixel_value(&self, x: u32, y: u32) -> (f32, f32, f32) {
        let idx = self.pixel_index(x, y);
        let v = self.data.get(idx).unwrap();
        (v[0], v[1], v[2])
    }

    pub fn sample_pixel_value(&self, x: f32, y: f32) -> (f32, f32, f32) {
        // find up to four relevant pixels, take the weighted average of their values ignoring NANs
        (0.0, 0.0, 0.0)
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        vec![]
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

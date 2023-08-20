use std::path::Path;

use crate::ray_marcher::RayMarcher;
use crate::sdf::Sdf;
use crate::vector::{Vec2, Vec3, vec2, vec3, VecFloat};

use tiny_skia::{Pixmap, Transform, PathBuilder, Paint, Stroke, Color, LineCap, LineJoin, Rect, IntSize};


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
    const NAN_RGBA_VALUE: [u8; 4] = [255, 0, 255, 255];

    pub fn new(width: u32, height: u32) -> LightDirectionDistanceCanvas {
        let data_length = (width as usize) * (height as usize);
        let data = vec![[f32::NAN; 3]; data_length];
        LightDirectionDistanceCanvas {
            data,
            width,
            height,
        }
    }

    pub fn from_sdf_scene(ray_marcher: &RayMarcher, sdf: Sdf, width: u32, height: u32, light_point_source: &Vec3) -> LightDirectionDistanceCanvas {
        let mut canvas = Self::new(width, height);
        for i_y in 0..height {
            for i_x in 0..width {
                let screen_coordinates = canvas.to_screen_coordinates(i_x as f32 + 0.5, i_y as f32 + 0.5);
                let intersection = ray_marcher.intersection_with_scene(sdf, &screen_coordinates);
                if intersection.is_some() {
                    let (p, distance) = intersection.unwrap();
                    let normal = RayMarcher::scene_normal(sdf, &p);
                    let lightness = RayMarcher::light_intensity(sdf, &p, &normal, &light_point_source);
                    let tangent_plane_basis = vec3::orthonormal_basis_of_plane(&normal, &vec3::sub(&light_point_source, &p));
                    let direction = match tangent_plane_basis {
                        Some((_, v)) => {
                            // Project p +/- h * v onto the canvas, take the polar angle of their difference as the direction
                            const H: VecFloat = 0.01;
                            let p_plus_v = vec3::scale_and_add(&p, &v, H);
                            let p_plus_v = ray_marcher.to_screen_coordinates(&p_plus_v);
                            let p_plus_v = canvas.to_canvas_coordinates(&p_plus_v);
                            let p_minus_v = vec3::scale_and_add(&p, &v, -H);
                            let p_minus_v = ray_marcher.to_screen_coordinates(&p_minus_v);
                            let p_minus_v = canvas.to_canvas_coordinates(&p_minus_v);

                            let dir_vec = vec2::sub(&p_plus_v, &p_minus_v);
                            vec2::polar_angle(&dir_vec)
                        },
                        None => f32::NAN
                    };
                    canvas.set_pixel(i_x, i_y, lightness, direction, distance);
                }
            }
        }
        canvas
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

    pub fn lightness_to_skia_canvas(&self) -> SkiaCanvas {
        let rgba_data = self.data.iter().map(|ldd| {
                let lightness = ldd[0];
                if lightness.is_nan() {
                    Self::NAN_RGBA_VALUE
                } else {
                    let l = (lightness.max(0.0).min(1.0) * 255.0) as u8;
                    [l, l, l, 255]
                }
            }).flatten().collect();
        SkiaCanvas::from_rgba(rgba_data, self.width, self.height)
    }

    pub fn direction_to_skia_canvas(&self) -> SkiaCanvas {
        let rgba_data = self.data.iter().map(|ldd| {
                let direction = ldd[1];
                if direction.is_nan() {
                    Self::NAN_RGBA_VALUE
                } else {
                    const PI2: f32 = 2.0 * std::f32::consts::PI;
                    let mut normalized_dir = direction % PI2;
                    if normalized_dir < 0.0 {
                        normalized_dir += PI2;
                    }
                    let d = (normalized_dir / PI2 * 255.0) as u8;
                    [d, d, d, 255]
                }
            }).flatten().collect();
        SkiaCanvas::from_rgba(rgba_data, self.width, self.height)
    }

    pub fn distance_to_skia_canvas(&self) -> SkiaCanvas {
        let (min_dist, max_dist) = self.data.iter().fold(
            (std::f32::INFINITY, std::f32::NEG_INFINITY),
            |(min_acc, max_acc), ldd| {
                let distance = ldd[2];
                if distance.is_nan() {
                    (min_acc, max_acc)
                } else {
                    (min_acc.min(distance), max_acc.max(distance))
                }
            }
        );
        let rgba_data = self.data.iter().map(|ldd| {
                let distance = ldd[2];
                if distance.is_nan() {
                    Self::NAN_RGBA_VALUE
                } else {
                    let normalized_dist = (distance - min_dist) / (max_dist - min_dist);
                    let d = ((1.0 - normalized_dist) * 255.0) as u8;
                    [d, d, d, 255]
                }
            }).flatten().collect();
        SkiaCanvas::from_rgba(rgba_data, self.width, self.height)
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

    pub fn from_rgba(rgba_data: Vec<u8>, width: u32, height: u32) -> SkiaCanvas {
        let mut pixmap = Pixmap::from_vec(rgba_data, IntSize::from_wh(width, height).unwrap()).unwrap();
        SkiaCanvas {
            pixmap
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

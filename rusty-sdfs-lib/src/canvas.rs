use std::path::Path;

use rayon::prelude::*;

use crate::ray_marcher::RayMarcher;
use crate::scene::Scene;
use crate::vector::{vec2, vec3, Vec2, Vec3, VecFloat};

use tiny_skia::{
    Color, IntSize, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

pub trait Canvas {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn aspect_ratio(&self) -> f32 {
        (self.width() as f32) / (self.height() as f32)
    }

    fn to_screen_coordinates_wh(width: u32, height: u32, x: f32, y: f32) -> Vec2 {
        vec2::from_values(
            2.0 * (x / (width as f32) - 0.5),
            -2.0 * (y / (height as f32) - 0.5),
        )
    }

    fn to_screen_coordinates(&self, x: f32, y: f32) -> Vec2 {
        Self::to_screen_coordinates_wh(self.width(), self.height(), x, y)
    }

    fn to_canvas_coordinates_wh(width: u32, height: u32, screen_coordinates: &Vec2) -> Vec2 {
        vec2::from_values(
            0.5 * (screen_coordinates.0 + 1.0) * (width as f32),
            0.5 * (-screen_coordinates.1 + 1.0) * (height as f32),
        )
    }

    fn to_canvas_coordinates(&self, screen_coordinates: &Vec2) -> Vec2 {
        Self::to_canvas_coordinates_wh(self.width(), self.height(), screen_coordinates)
    }
}

#[derive(Clone, Copy)]
pub struct PixelProperties {
    pub lightness: f32,
    pub direction: f32,
    pub depth: f32,
    pub bg_hsl: Vec3,
    pub is_shaded: bool,
    pub is_hatched: bool,
}

impl PixelProperties {
    fn default() -> PixelProperties {
        PixelProperties {
            lightness: f32::NAN,
            direction: f32::NAN,
            depth: f32::NAN,
            bg_hsl: vec3::from_values(0.0, 0.0, 1.0),
            is_shaded: false,
            is_hatched: false,
        }
    }
}

pub struct PixelPropertyCanvas {
    data: Vec<PixelProperties>,
    width: u32,
    height: u32,
}

impl Canvas for PixelPropertyCanvas {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

impl PixelPropertyCanvas {
    const NAN_RGBA_VALUE: [u8; 4] = [255, 0, 255, 255];

    pub fn new(width: u32, height: u32) -> PixelPropertyCanvas {
        let data_length = (width as usize) * (height as usize);
        let data = vec![PixelProperties::default(); data_length];
        PixelPropertyCanvas {
            data,
            width,
            height,
        }
    }

    pub fn from_scene<S>(
        ray_marcher: &RayMarcher,
        scene: &S,
        width: u32,
        height: u32,
        angle_in_tangent_plane: VecFloat,
    ) -> PixelPropertyCanvas
    where
        S: Scene + Sync,
    {
        let mut canvas = Self::new(width, height);
        let angle_cos = angle_in_tangent_plane.cos();
        let angle_sin = angle_in_tangent_plane.sin();
        canvas
            .pixels_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, pixel)| {
                let (i_x, i_y) = Self::pixel_coordinates_wh(width, index);
                let screen_coordinates = Self::to_screen_coordinates_wh(
                    width,
                    height,
                    i_x as f32 + 0.5,
                    i_y as f32 + 0.5,
                );
                let intersection = ray_marcher.intersection_with_scene(scene, &screen_coordinates);
                if intersection.is_some() {
                    let (p, depth, material) = intersection.unwrap();
                    let normal = ray_marcher.scene_normal(scene, &p);
                    let lightness = ray_marcher.light_intensity(
                        scene,
                        &material.reflective_properties,
                        &p,
                        &normal,
                        &material.light_source,
                    );
                    let tangent_plane_basis = vec3::orthonormal_basis_of_plane(
                        &normal,
                        &vec3::sub(&material.light_source, &p),
                    );
                    let direction = match tangent_plane_basis {
                        Some((u, v)) => {
                            let dir_in_plane = vec3::scale_and_add_inplace(
                                vec3::scale(&v, angle_cos),
                                &u,
                                angle_sin,
                            );

                            // Project p +/- h * dir_in_plane onto the canvas, take the polar angle of their difference as the direction
                            const H: VecFloat = 0.01;
                            let p_plus_dir = vec3::scale_and_add(&p, &dir_in_plane, H);
                            let p_plus_dir = ray_marcher.to_screen_coordinates(&p_plus_dir);
                            let p_plus_dir =
                                Self::to_canvas_coordinates_wh(width, height, &p_plus_dir);
                            let p_minus_dir = vec3::scale_and_add(&p, &dir_in_plane, -H);
                            let p_minus_dir = ray_marcher.to_screen_coordinates(&p_minus_dir);
                            let p_minus_dir =
                                Self::to_canvas_coordinates_wh(width, height, &p_minus_dir);

                            let dir_vec = vec2::sub(&p_plus_dir, &p_minus_dir);
                            vec2::polar_angle(&dir_vec)
                        }
                        None => f32::NAN,
                    };
                    pixel.lightness = lightness;
                    pixel.direction = direction;
                    pixel.depth = depth;
                    pixel.bg_hsl = material.bg_hsl;
                    pixel.is_shaded = material.is_shaded;
                    pixel.is_hatched = material.is_hatched;
                }
            });
        canvas
    }

    fn pixel_index(&self, x: u32, y: u32) -> usize {
        (self.width as usize) * (y as usize) + (x as usize)
    }

    fn pixel_coordinates_wh(width: u32, index: usize) -> (u32, u32) {
        (
            (index % (width as usize)) as u32,
            (index / (width as usize)) as u32,
        )
    }

    fn pixel_coordinates(&self, index: usize) -> (u32, u32) {
        Self::pixel_coordinates_wh(self.width, index)
    }

    pub fn pixel_value(&self, x: f32, y: f32) -> Option<PixelProperties> {
        if x < 0.0 || y < 0.0 || x >= self.width as f32 || y >= self.height as f32 {
            return None;
        }
        let idx = self.pixel_index(x as u32, y as u32);
        let pixel = self.data.get(idx).unwrap();
        if pixel.lightness.is_nan() || pixel.direction.is_nan() || pixel.depth.is_nan() {
            None
        } else {
            Some(*pixel)
        }
    }

    pub fn pixels_mut(&mut self) -> &mut Vec<PixelProperties> {
        &mut self.data
    }

    pub fn bg_to_skia_canvas(&self) -> SkiaCanvas {
        let rgba_data = self
            .data
            .iter()
            .map(|pixel| {
                let hsl = if pixel.is_shaded && !pixel.lightness.is_nan() {
                    vec3::from_values(
                        pixel.bg_hsl.0,
                        pixel.bg_hsl.1,
                        (pixel.bg_hsl.2 * pixel.lightness).clamp(0.0, 1.0),
                    )
                } else {
                    pixel.bg_hsl
                };
                vec3::hsl_to_rgba_u8(&hsl)
            })
            .flatten()
            .collect();
        SkiaCanvas::from_rgba(rgba_data, self.width, self.height)
    }

    pub fn lightness_to_skia_canvas(&self) -> SkiaCanvas {
        let rgba_data = self
            .data
            .iter()
            .map(|pixel| {
                if pixel.lightness.is_nan() {
                    Self::NAN_RGBA_VALUE
                } else {
                    let l = (pixel.lightness.clamp(0.0, 1.0) * 255.0) as u8;
                    [l, l, l, 255]
                }
            })
            .flatten()
            .collect();
        SkiaCanvas::from_rgba(rgba_data, self.width, self.height)
    }

    pub fn direction_to_skia_canvas(&self) -> SkiaCanvas {
        let rgba_data = self
            .data
            .iter()
            .map(|pixel| {
                if pixel.direction.is_nan() {
                    Self::NAN_RGBA_VALUE
                } else {
                    const PI2: f32 = 2.0 * std::f32::consts::PI;
                    let mut normalized_dir = pixel.direction % PI2;
                    if normalized_dir < 0.0 {
                        normalized_dir += PI2;
                    }
                    let d = (normalized_dir / PI2 * 255.0) as u8;
                    [d, d, d, 255]
                }
            })
            .flatten()
            .collect();
        SkiaCanvas::from_rgba(rgba_data, self.width, self.height)
    }

    pub fn depth_to_skia_canvas(&self) -> SkiaCanvas {
        let (min_depth, max_depth) = self.data.iter().fold(
            (std::f32::INFINITY, std::f32::NEG_INFINITY),
            |(min_acc, max_acc), pixel| {
                if pixel.depth.is_nan() {
                    (min_acc, max_acc)
                } else {
                    (min_acc.min(pixel.depth), max_acc.max(pixel.depth))
                }
            },
        );
        let rgba_data = self
            .data
            .iter()
            .map(|pixel| {
                if pixel.depth.is_nan() {
                    Self::NAN_RGBA_VALUE
                } else {
                    let normalized_depth = (pixel.depth - min_depth) / (max_depth - min_depth);
                    let d = ((1.0 - normalized_depth) * 255.0) as u8;
                    [d, d, d, 255]
                }
            })
            .flatten()
            .collect();
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
        SkiaCanvas { pixmap }
    }

    pub fn from_rgba(rgba_data: Vec<u8>, width: u32, height: u32) -> SkiaCanvas {
        let pixmap = Pixmap::from_vec(rgba_data, IntSize::from_wh(width, height).unwrap()).unwrap();
        SkiaCanvas { pixmap }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, rgb: [u8; 3], a: u8) {
        let rect = Rect::from_xywh(x, y, w, h).unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], a);
        paint.anti_alias = true;

        let transform = Transform::identity();
        self.pixmap.fill_rect(rect, &paint, transform, None);
    }

    pub fn stroke_line_segments(&mut self, points: &[Vec2], width: f32, rgb: &[u8; 3]) {
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
        self.pixmap
            .stroke_path(&path, &paint, &stroke, transform, None);
    }

    pub fn save_png(&self, path: &Path) {
        self.pixmap.save_png(path).unwrap();
    }
}

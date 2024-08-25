use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

use crate::ray_marcher::RayMarcher;
use crate::scene::Scene;
use crate::vector::{vec2, vec3, Vec2, Vec3, VecFloat};
use crate::Material;

use bincode;
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};
use tiny_skia::{
    Color, FillRule, IntSize, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, PremultipliedColorU8, Rect, Stroke, Transform
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

#[derive(Debug)]
pub enum CanvasError {
    Io(io::Error),
    Serialization(bincode::Error),
}

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanvasError::Io(err) => write!(f, "I/O error: {}", err),
            CanvasError::Serialization(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl From<io::Error> for CanvasError {
    fn from(err: io::Error) -> CanvasError {
        CanvasError::Io(err)
    }
}

impl From<bincode::Error> for CanvasError {
    fn from(err: bincode::Error) -> CanvasError {
        CanvasError::Serialization(err)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

    pub fn to_file(&self, filename: &str) -> Result<(), CanvasError> {
        let file = File::create(filename)?;
        let writer = BufWriter::new(file);
        Ok(bincode::serialize_into(writer, self)?)
    }

    pub fn from_file(filename: &str) -> Result<Self, CanvasError> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Ok(bincode::deserialize_from(reader)?)
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
        let offset_angle_vector = vec2::from_values(
            angle_in_tangent_plane.cos(),
            angle_in_tangent_plane.sin()
        );
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
                    let direction = Self::world_to_canvas_direction(
                        ray_marcher,
                        width,
                        height,
                        &p,
                        &normal,
                        &material.light_source,
                        &offset_angle_vector
                    );
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

    pub fn from_heightmap<F>(
        ray_marcher: &RayMarcher,
        heightmap: &F,
        material: &Material,
        width: u32,
        height: u32,
        angle_in_tangent_plane: VecFloat,
    ) -> PixelPropertyCanvas
    where
        F: Fn(f32, f32) -> f32 + Sync,
    {
        let mut canvas = Self::new(width, height);
        let offset_angle_vector = vec2::from_values(
            angle_in_tangent_plane.cos(),
            angle_in_tangent_plane.sin()
        );
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
                let intersection = ray_marcher.intersection_with_heightmap(heightmap, &screen_coordinates);
                if intersection.is_some() {
                    let (p, depth) = intersection.unwrap();
                    let normal = ray_marcher.heightmap_normal(heightmap, &p);
                    let lightness = ray_marcher.heightmap_light_intensity(
                        heightmap,
                        &material.reflective_properties,
                        &p,
                        &normal,
                        &material.light_source,
                    );
                    let direction = Self::world_to_canvas_direction(
                        ray_marcher,
                        width,
                        height,
                        &p,
                        &normal,
                        &material.light_source,
                        &offset_angle_vector
                    );
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

    fn world_to_canvas_direction(
        ray_marcher: &RayMarcher,
        canvas_width: u32,
        canvas_height: u32,
        p: &Vec3,
        normal: &Vec3,
        light_source: &Vec3,
        offset_angle_vector: &Vec2
    ) -> f32 {
        let tangent_plane_basis = vec3::orthonormal_basis_of_plane(
            normal,
            &vec3::sub(light_source, p),
        );
        match tangent_plane_basis {
            Some((u, v)) => {
                let dir_in_plane = vec3::scale_and_add_inplace(
                    vec3::scale(&v, offset_angle_vector.0),
                    &u,
                    offset_angle_vector.1,
                );

                // Project p +/- h * dir_in_plane onto the canvas, take the polar angle of their difference as the direction
                const H: VecFloat = 0.01;
                let p_plus_dir = vec3::scale_and_add(p, &dir_in_plane, H);
                let p_plus_dir = ray_marcher.to_screen_coordinates(&p_plus_dir);
                let p_plus_dir =
                    Self::to_canvas_coordinates_wh(canvas_width, canvas_height, &p_plus_dir);
                let p_minus_dir = vec3::scale_and_add(p, &dir_in_plane, -H);
                let p_minus_dir = ray_marcher.to_screen_coordinates(&p_minus_dir);
                let p_minus_dir =
                    Self::to_canvas_coordinates_wh(canvas_width, canvas_height, &p_minus_dir);

                let dir_vec = vec2::sub(&p_plus_dir, &p_minus_dir);
                vec2::polar_angle(&dir_vec)
            }
            None => f32::NAN,
        }
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
        let pixmap = Pixmap::new(width, height).unwrap();
        let mut canvas = SkiaCanvas { pixmap };
        canvas.fill(&[255, 255, 255]);
        canvas
    }

    pub fn from_rgba(rgba_data: Vec<u8>, width: u32, height: u32) -> SkiaCanvas {
        let pixmap = Pixmap::from_vec(rgba_data, IntSize::from_wh(width, height).unwrap()).unwrap();
        SkiaCanvas { pixmap }
    }

    pub fn to_u32_rgb(&self) -> Vec<u32> {
        self.pixmap.data().chunks_exact(4).map(|rgba| {
            let r = rgba[0] as u32;
            let g = rgba[1] as u32;
            let b = rgba[2] as u32;
            (r << 16) | (g << 8) | b
        }).collect()
    }

    pub fn sample_bilinear(&self, x: f32, y: f32) -> PremultipliedColorU8 {
        const EPSILON: f32  = 1.0 / 256.0;
        let x_clamp = x.clamp(0.0, (self.width() - 1) as f32 - EPSILON);
        let y_clamp = y.clamp(0.0, (self.height() - 1) as f32 - EPSILON);
        let xi = x_clamp as u32;
        let yi = y_clamp as u32;
        let xf = x_clamp.fract();
        let yf = y_clamp.fract();

        let p00 = self.pixmap.pixel(xi, yi).unwrap();
        let p01 = self.pixmap.pixel(xi+1, yi).unwrap();
        let p10 = self.pixmap.pixel(xi, yi+1).unwrap();
        let p11 = self.pixmap.pixel(xi+1, yi+1).unwrap();

        let w00 = (1.0 - xf) * (1.0 - yf);
        let w01 = xf * (1.0 - yf);
        let w10 = (1.0 - xf) * yf;
        let w11 = xf * yf;

        let r = (w00 * p00.red() as f32 + w01 * p01.red() as f32 + w10 * p10.red() as f32 + w11 * p11.red() as f32) as u8;
        let g = (w00 * p00.green() as f32 + w01 * p01.green() as f32 + w10 * p10.green() as f32 + w11 * p11.green() as f32) as u8;
        let b = (w00 * p00.blue() as f32 + w01 * p01.blue() as f32 + w10 * p10.blue() as f32 + w11 * p11.blue() as f32) as u8;
        let a = (w00 * p00.alpha() as f32 + w01 * p01.alpha() as f32 + w10 * p10.alpha() as f32 + w11 * p11.alpha() as f32) as u8;

        PremultipliedColorU8::from_rgba(r, g, b, a).unwrap()
    }

    pub fn iter_mut_rgba_with_coordinates<F>(&mut self, f: F)
    where
        F: Fn(u32, u32, &mut [u8]) -> ()
    {
        let w = self.width() as usize;
        let h = self.height() as usize;
        for iy in 0..h {
            for ix in 0..w {
                let base_index = 4 * (iy * w + ix);
                let rgba = &mut self.pixmap.data_mut()[base_index..base_index + 4];
                f(ix as u32, iy as u32, rgba);
            }
        }
    }

    pub fn fill(&mut self, rgb: &[u8; 3]) {
        self.pixmap.fill(Color::from_rgba8(rgb[0], rgb[1], rgb[2], 255));
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, rgb: &[u8; 3]) {
        let rect = Rect::from_xywh(x, y, w, h).unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], 255);
        paint.anti_alias = true;

        let transform = Transform::identity();
        self.pixmap.fill_rect(rect, &paint, transform, None);
    }

    pub fn fill_points(&mut self, points: &[Vec2], radius: f32, rgb: &[u8; 3]) {
        if points.len() < 1 {
            return;
        }

        let mut pb = PathBuilder::new();
        for p in points {
            pb.push_circle(p.0, p.1, radius);
        }
        let path = pb.finish().unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], 255);
        paint.anti_alias = true;

        let transform = Transform::identity();
        self.pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
    }

    pub fn linear_path(points: &[Vec2]) -> Option<Path> {
        if points.len() < 2 {
            return None;
        }
        let mut pb = PathBuilder::new();
        let head = points[0];
        let tail = &points[1..];
        pb.move_to(head.0, head.1);
        for p in tail {
            pb.line_to(p.0, p.1);
        }
        pb.finish()
    }

    pub fn closed_linear_path(points: &[Vec2]) -> Option<Path> {
        if points.len() < 2 {
            return None;
        }
        let mut pb = PathBuilder::new();
        let head = points[0];
        let tail = &points[1..];
        pb.move_to(head.0, head.1);
        for p in tail {
            pb.line_to(p.0, p.1);
        }
        pb.close();
        pb.finish()
    }

    pub fn closed_cubic_curve_path(curve_points: &[Vec2], ctrl_points_left: &[Vec2], ctrl_points_right: &[Vec2]) -> Option<Path> {
        if curve_points.len() < 2 || ctrl_points_left.len() != curve_points.len() || ctrl_points_right.len() != curve_points.len() {
            return None;
        }
        let mut pb = PathBuilder::new();
        let p0 = curve_points[0];
        pb.move_to(p0.0, p0.1);
        curve_points.iter()
            .skip(1)
            .zip(ctrl_points_right.iter())
            .zip(ctrl_points_left.iter().skip(1))
            .for_each(|((p, c1), c2)| {
                pb.cubic_to(c1.0, c1.1, c2.0, c2.1, p.0, p.1);
            });
        let c1 = ctrl_points_right.last().unwrap();
        let c2 = ctrl_points_left[0];
        pb.cubic_to(c1.0, c1.1, c2.0, c2.1, p0.0, p0.1);
        pb.finish()
    }

    pub fn stroke_path(&mut self, path: &Path, width: f32, rgb: &[u8; 3]) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], 255);
        paint.anti_alias = true;

        let mut stroke = Stroke::default();
        stroke.width = width;
        stroke.line_cap = LineCap::Round;
        stroke.line_join = LineJoin::Round;

        let transform = Transform::identity();
        self.pixmap.stroke_path(path, &paint, &stroke, transform, None);
    }

    pub fn fill_path(&mut self, path: &Path, rgb: &[u8; 3]) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], 255);
        paint.anti_alias = true;

        let transform = Transform::identity();
        self.pixmap.fill_path(path, &paint, FillRule::Winding, transform, None);
    }

    pub fn stroke_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, rgb: &[u8; 3]) {
        let mut pb = PathBuilder::new();
        pb.move_to(x0, y0);
        pb.line_to(x1, y1);
        let path = pb.finish().unwrap();
        self.stroke_path(&path, width, rgb);
    }

    pub fn save_png(&self, path: &std::path::Path) {
        self.pixmap.save_png(path).unwrap();
    }

    pub fn display_in_window(&self, title: &str) {
        let mut window = Window::new(
            title,
            self.width() as usize,
            self.height() as usize,
            WindowOptions::default()
        )
        .unwrap();
        window.update(); // Ensure that the window is initialized before setting its contents
        let buffer = self.to_u32_rgb();
        window.update_with_buffer(&buffer, self.width() as usize, self.height() as usize).unwrap();
        window.limit_update_rate(Some(std::time::Duration::from_millis(100)));
        while window.is_open() && !window.is_key_down(Key::Escape) {
            window.update();
        }
    }
}

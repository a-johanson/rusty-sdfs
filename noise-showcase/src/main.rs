mod wave;

use std::path::Path;

use rusty_sdfs_lib::vec3;
use rusty_sdfs_lib::Material;
use rusty_sdfs_lib::PixelPropertyCanvas;
use rusty_sdfs_lib::RayMarcher;
use rusty_sdfs_lib::ReflectiveProperties;
use rusty_sdfs_lib::SkiaCanvas;
use rusty_sdfs_lib::noise_2d;

use crate::wave::{blob_heightmap, noisy_waves};

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;

    let camera = vec3::from_values(0.0, 2.5, 5.0);
    let look_at = vec3::from_values(0.0, 0.0, 0.0);
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let fov = 55.0;
    const MAX_CHANGE_RATE: f32 = 2.0;
    let ray_marcher = RayMarcher::new(
        1.0 * 1.0 / (MAX_CHANGE_RATE * MAX_CHANGE_RATE + 1.0).sqrt(),
        &camera,
        &look_at,
        &up,
        fov,
        (width as f32) / (height as f32),
    );

    let rp = ReflectiveProperties::new(
        0.1,
        0.0,
        0.0,
        0.9,
        0.0,
        None,
        None,
        None,
        None
    );
    let material = Material::new(
        &vec3::from_values(1.0e5, 2.0e5, 5.0e5),
        Some(&rp),
        None,
        true,
        false
    );
    let pp_canvas = PixelPropertyCanvas::from_heightmap(&ray_marcher, &blob_heightmap, &material, width, height, 0.0);
    let lightness_canvas = pp_canvas.lightness_to_skia_canvas();
    lightness_canvas.save_png(Path::new("heightmap.png"));

    let scale: f32 = 0.01;
    let octaves = 4;
    let mut noise_values: Vec<f32> = vec![0.0; width as usize * height as usize];

    for iy in 0..height {
        let y = scale * (iy as f32 - 0.5 * height as f32);
        for ix in 0..width {
            let x = scale * (ix as f32 - 0.5 * width as f32);
            noise_values[iy as usize * width as usize + ix as usize] = noisy_waves(x, y, octaves);
        }
    }

    let min_value = noise_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_value = noise_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    println!("min_value = {min_value}, max_value = {max_value}");
    let rgba_values: Vec<u8> = noise_values.iter().map(|&v| {
        let v_normalized = (v - min_value) / (max_value - min_value);
        let lightness = (255.0 * v_normalized) as u8;
        [lightness, lightness, lightness, 255]
    }).flatten().collect();

    let output_canvas = SkiaCanvas::from_rgba(rgba_values, width, height);
    output_canvas.save_png(Path::new("noise.png"));
}

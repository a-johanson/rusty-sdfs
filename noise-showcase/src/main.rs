
use std::path::Path;

use rusty_sdfs_lib::SkiaCanvas;
use rusty_sdfs_lib::noise_2d;

fn main() {
    let width: u32 = 600;
    let height: u32 = 600;
    let scale: f32 = 0.01;
    let octaves = 4;
    let mut noise_values: Vec<f32> = vec![0.0; width as usize * height as usize];

    for iy in 0..height {
        let y = scale * (iy as f32 - 0.5 * height as f32);
        for ix in 0..width {
            let x = scale * (ix as f32 - 0.5 * width as f32);
            noise_values[iy as usize * width as usize + ix as usize] = noise_2d(x, y, octaves);
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

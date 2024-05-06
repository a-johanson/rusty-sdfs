#![allow(dead_code)]

mod wave;

// use std::path::Path;

use rusty_sdfs_lib::vec3;
use rusty_sdfs_lib::Material;
use rusty_sdfs_lib::PixelPropertyCanvas;
use rusty_sdfs_lib::RayMarcher;
use rusty_sdfs_lib::ReflectiveProperties;

use crate::wave::blob_heightmap;

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
    // lightness_canvas.save_png(Path::new("heightmap.png"));
    lightness_canvas.display_in_window("heightmap");
}

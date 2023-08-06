mod canvas;
mod ray_marcher;
mod sdf;

use std::path::Path;
use std::time::Instant;

use gl_matrix::common::Vec3;
use gl_matrix::vec3;

use canvas::Canvas;
use ray_marcher::RayMarcher;
use sdf::{sd_plane, sd_sphere};


fn scene(p: &Vec3) -> f32 {
    sd_sphere(p, 1.0).min(sd_plane(p, &vec3::from_values(0.0, 1.0, 0.0), -1.0))
}

fn on_grid<F>(width: f32, height: f32, cells_x: u32, cells_y: u32, mut f: F)
where
    F: FnMut(f32, f32, f32, f32) -> ()
{
    let cell_width = width / (cells_x as f32);
    let cell_height = height / (cells_y as f32);
    let mut i_y: u32 = 0;
    while i_y < cells_y {
        let mut i_x: u32 = 0;
        while i_x < cells_x {
            let x = cell_width * (i_x as f32);
            let y = cell_height * (i_y as f32);
            f(x, y, cell_width, cell_height);

            i_x += 1;
        }
        i_y += 1;
    }
}

fn main() {
    let mut canvas = Canvas::new(1200, 1600);

    let camera = vec3::from_values(0.0, 2.0, 5.0);
    let look_at = vec3::from_values(0.0, 0.0, 0.0);
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let ray_marcher = RayMarcher::new(
        &camera,
        &look_at,
        &up,
        55.0,
        canvas.aspect_ratio()
    );
    let light_point_source = vec3::from_values(2.0, 2.0, 1.0);

    println!("Rendering...");
    let start_instant = Instant::now();
    on_grid(canvas.width() as f32, canvas.height() as f32, canvas.width(), canvas.height(), |x, y, w, h| {
        let screen_coordinates = canvas.to_screen_coordinates(x + 0.5 * w, y + 0.5 * h);
        let intersection = ray_marcher.intersection_with_scene(scene, &screen_coordinates);
        let l = match intersection {
            Some(p) => {
                let normal = RayMarcher::scene_normal(scene, &p);
                (255.0 * RayMarcher::light_intensity(scene, &p, &normal, &light_point_source)) as u8
            }
            None => 0
        };
        canvas.fill_rect(x, y, w, h, l);
    });

    // canvas.stroke_line_segments(&[
    //     [100.0, 100.0],
    //     [500.0, 200.0],
    //     [200.0, 400.0],
    // ]);

    let duration = start_instant.elapsed();
    println!("Finished rendering after {} seconds", duration.as_secs_f32());
    println!("Saving image to disk...");
    canvas.save_png(Path::new("out.png"));
    println!("Done");
}

mod canvas;
mod grid;
mod ray_marcher;
mod scene;
mod sdf;

use std::path::Path;
use std::time::Instant;

use gl_matrix::vec3;

use canvas::Canvas;
use grid::on_grid;
use ray_marcher::RayMarcher;
use scene::scene;


fn main() {
    const WIDTH_IN_CM: f32  = 21.0;
    const HEIGHT_IN_CM: f32 = 29.7;
    const DPI: f32          = 75.0;
    const INCH_PER_CM: f32  = 1.0 / 2.54;
    let width  = (WIDTH_IN_CM  * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let mut canvas = Canvas::new(width, height);

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

    println!("Rendering on canvas of size {} px x {} px...", width, height);
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

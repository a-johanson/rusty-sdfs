mod canvas;
mod grid;
mod ray_marcher;
mod scene;
mod sdf;
mod streamline;
mod vector;

use std::path::Path;
use std::time::Instant;

use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

use canvas::{Canvas, SkiaCanvas, LightDirectionDistanceCanvas};
use grid::{on_grid, on_jittered_grid};
use ray_marcher::RayMarcher;
use scene::scene;
use sdf::Sdf;
use streamline::gradient_streamline_segments;
use vector::{Vec2, Vec3, vec2, vec3, to_radian, VecFloat};


fn main() {
    const RNG_SEED: u64           = 6280954363;
    const WIDTH_IN_CM: f32        = 21.0;
    const HEIGHT_IN_CM: f32       = 29.7;
    const STROKE_WIDTH_IN_MM: f32 = 0.5;
    const DPI: f32                = 75.0;

    const INCH_PER_CM: f32  = 1.0 / 2.54;
    const STROKE_WIDTH: f32 = 0.1 * STROKE_WIDTH_IN_MM * INCH_PER_CM * DPI;
    let width  = (WIDTH_IN_CM  * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let mut canvas = SkiaCanvas::new(width, height);

    let camera = vec3::from_values(0.0, 2.5, 10.0);
    let look_at = vec3::from_values(0.1, 2.0, 0.0);
    let up = vec3::from_values(0.0, 1.0, 0.0);
    let ray_marcher = RayMarcher::new(
        &camera,
        &look_at,
        &up,
        45.0,
        canvas.aspect_ratio()
    );
    let light_point_source = vec3::scale(&vec3::from_values(-5.0, 4.0, 3.0), 7.0);

    let mut rng = Xoshiro256PlusPlus::seed_from_u64(RNG_SEED);

    println!("Rendering on canvas of size {} px x {} px using a stroke width of {} px...", width, height, STROKE_WIDTH);
    let start_instant = Instant::now();
    on_grid(canvas.width() as f32, canvas.height() as f32, canvas.width(), canvas.height(), |x, y, w, h| {
        let screen_coordinates = canvas.to_screen_coordinates(x + 0.5 * w, y + 0.5 * h);
        let intersection = ray_marcher.intersection_with_scene(scene, &screen_coordinates);
        let l = match intersection {
            Some((p, _)) => {
                let normal = RayMarcher::scene_normal(scene, &p);
                (255.0 * RayMarcher::light_intensity(scene, &p, &normal, &light_point_source)) as u8
            }
            None => 0
        };
        canvas.fill_rect(x, y, w, h, [l, l, l], 255);
    });

    // canvas.fill_rect(0.0, 0.0, width as f32, height as f32, [255, 255, 255], 127);

    on_jittered_grid(canvas.width() as f32, canvas.height() as f32, canvas.width() / 25, canvas.height() / 25, &mut rng, |x, y| {
        let screen_coordinates = canvas.to_screen_coordinates(x, y);
        let intersection = ray_marcher.intersection_with_scene(scene, &screen_coordinates);
        if intersection.is_some() {
            let (p, _) = intersection.unwrap();
            let hatch_line_segments_right = gradient_streamline_segments(&ray_marcher, scene, &p, &light_point_source, 70, 0.01, to_radian(90.0));
            let hatch_line_segments_left  = gradient_streamline_segments(&ray_marcher, scene, &p, &light_point_source, 70, 0.01, to_radian(-90.0));

            for seg in hatch_line_segments_right.iter().chain(hatch_line_segments_left.iter()) {
                let canvas_points: Vec<Vec2> = seg.iter().map(|sc| canvas.to_canvas_coordinates(sc)).collect();
                canvas.stroke_line_segments(&canvas_points, STROKE_WIDTH, [217, 2, 125]);
            }
        }
    });

    let mut ldd_canvas = LightDirectionDistanceCanvas::new(width, height);
    for i_y in 0..height {
        for i_x in 0..width {
            let screen_coordinates = ldd_canvas.to_screen_coordinates(i_x as f32 + 0.5, i_y as f32 + 0.5);
            let intersection = ray_marcher.intersection_with_scene(scene, &screen_coordinates);
            if intersection.is_some() {
                let (p, distance) = intersection.unwrap();
                let normal = RayMarcher::scene_normal(scene, &p);
                let lightness = RayMarcher::light_intensity(scene, &p, &normal, &light_point_source);
                let tangent_plane_basis = vec3::orthonormal_basis_of_plane(&normal, &vec3::sub(&light_point_source, &p));
                let direction = match tangent_plane_basis {
                    Some((u, v)) => {
                        // Project p +/- h * v onto the canvas, take the polar angle of their difference as the direction
                        const H: VecFloat = 0.01;
                        let p_plus_v = vec3::scale_and_add(&p, &v, H);
                        let p_plus_v = ray_marcher.to_screen_coordinates(&p_plus_v);
                        let p_plus_v = ldd_canvas.to_canvas_coordinates(&p_plus_v);
                        let p_minus_v = vec3::scale_and_add(&p, &v, -H);
                        let p_minus_v = ray_marcher.to_screen_coordinates(&p_minus_v);
                        let p_minus_v = ldd_canvas.to_canvas_coordinates(&p_minus_v);

                        let dir_vec = vec2::sub(&p_plus_v, &p_minus_v);
                        vec2::polar_angle(&dir_vec)

                    },
                    None => f32::NAN
                };

                ldd_canvas.set_pixel(i_x, i_y, lightness, direction, distance);
            }
        }
    }

    let duration = start_instant.elapsed();
    println!("Finished rendering after {} seconds", duration.as_secs_f32());
    println!("Saving image to disk...");
    canvas.save_png(Path::new("out.png"));
    let lightness_canvas = ldd_canvas.lightness_to_skia_canvas();
    lightness_canvas.save_png(Path::new("out_lightness.png"));
    let direction_canvas = ldd_canvas.direction_to_skia_canvas();
    direction_canvas.save_png(Path::new("out_direction.png"));
    let distance_canvas = ldd_canvas.distance_to_skia_canvas();
    distance_canvas.save_png(Path::new("out_distance.png"));
    println!("Done");
}

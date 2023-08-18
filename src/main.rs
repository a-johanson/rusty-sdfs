mod canvas;
mod grid;
mod ray_marcher;
mod scene;
mod sdf;
mod vector;

use std::path::Path;
use std::time::Instant;

use canvas::{Canvas, SkiaCanvas};
use grid::on_grid;
use ray_marcher::RayMarcher;
use scene::scene;
use sdf::Sdf;
use vector::{Vec2, Vec3, vec3, to_radian};


fn hatch_line_segments(
    ray_marcher: &RayMarcher,
    sdf: Sdf,
    p_scene: &Vec3,
    light_point_source: &Vec3,
    step_count: u32,
    step_scale: f32,
    hatch_angle: f32,
) -> Vec<Vec<Vec2>> {
    let mut segments: Vec<Vec<Vec2>> = vec![vec![ray_marcher.to_screen_coordinates(p_scene)]];
    let cos_hatch_angle = hatch_angle.cos();
    let sin_hatch_angle = hatch_angle.sin();
    let mut p_prev = *p_scene;
    let mut n_prev = RayMarcher::scene_normal(sdf, &p_prev);
    let mut i: u32 = 0;
    while i < step_count {
        // Construct an orthonormal basis (u, v) of the plane defined by the normal at p_prev
        let plane_basis = vec3::orthonormal_basis_of_plane(&n_prev, &vec3::sub(light_point_source, &p_prev));
        if plane_basis.is_none() {
            println!("WARNING: cannot construct orthonormal basis of tangent plane");
            break;
        }
        let (u, v) = plane_basis.unwrap();

        let surface_dir = vec3::scale_and_add_inplace(
            vec3::scale(&u, cos_hatch_angle),
            &v,
            sin_hatch_angle
        );

        let p_next = vec3::scale_and_add(&p_prev, &surface_dir, step_scale);
        let n_next = RayMarcher::scene_normal(sdf, &p_next);
        let visibility = RayMarcher::visibility_factor(sdf, &ray_marcher.camera, &p_next, Some(&n_next));

        if visibility > 0.0 {
            segments.last_mut().unwrap().push(ray_marcher.to_screen_coordinates(&p_next));
        }
        else if !segments.last().unwrap().is_empty() {
            segments.push(Vec::<Vec2>::new());
        }

        p_prev = p_next;
        n_prev = n_next;

        i += 1;
    }
    segments
}

fn main() {
    const WIDTH_IN_CM: f32        = 21.0;
    const HEIGHT_IN_CM: f32       = 29.7;
    const STROKE_WIDTH_IN_MM: f32 = 0.5;
    const DPI: f32                = 75.0;

    const INCH_PER_CM: f32  = 1.0 / 2.54;
    const STROKE_WIDTH: f32 = 0.1 * STROKE_WIDTH_IN_MM * INCH_PER_CM * DPI;
    let width  = (WIDTH_IN_CM  * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let mut canvas = SkiaCanvas::new(width, height);

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

    println!("Rendering on canvas of size {} px x {} px using a stroke width of {} px...", width, height, STROKE_WIDTH);
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
        canvas.fill_rect(x, y, w, h, [l, l, l], 255);
    });

    // canvas.fill_rect(0.0, 0.0, width as f32, height as f32, [255, 255, 255], 127);

    on_grid(canvas.width() as f32, canvas.height() as f32, canvas.width() / 25, canvas.height() / 25, |x, y, w, h| {
        let screen_coordinates = canvas.to_screen_coordinates(x + 0.5 * w, y + 0.5 * h);
        let intersection = ray_marcher.intersection_with_scene(scene, &screen_coordinates);
        if intersection.is_some() {
            let p = intersection.unwrap();
            let hatch_line_segments_right = hatch_line_segments(&ray_marcher, scene, &p, &light_point_source, 70, 0.01, to_radian(90.0));
            let hatch_line_segments_left  = hatch_line_segments(&ray_marcher, scene, &p, &light_point_source, 70, 0.01, to_radian(-90.0));

            for seg in hatch_line_segments_right.iter().chain(hatch_line_segments_left.iter()) {
                let canvas_points: Vec<Vec2> = seg.iter().map(|sc| canvas.to_canvas_coordinates(sc)).collect();
                canvas.stroke_line_segments(&canvas_points, STROKE_WIDTH, [217, 2, 125]);
            }
        }
    });

    let duration = start_instant.elapsed();
    println!("Finished rendering after {} seconds", duration.as_secs_f32());
    println!("Saving image to disk...");
    canvas.save_png(Path::new("out.png"));
    println!("Done");
}

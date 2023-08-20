use crate::ray_marcher::RayMarcher;
use crate::sdf::Sdf;
use crate::vector::{Vec2, Vec3, vec3};


// Idea from Jobard & Lefer (1997) "Creating Evenly-Spaced Streamlines of Arbitrary Density":

// 1. Drawing streamlines with evenly spaced separation
// We want to ensure no pair of streamlines is closer than d_sep
// When drawing a streamline, though, only stop when we are closer to a streamline than d_test <= d_sep
// When doing distance tests, we only compute distance to the equidistantly placed sample points on a streamline
// We further accelerate distance testing by subdividing space into cells of side length d_sep
// In our case, we want to make d_sep (and, d_test) depend on the local lightness of the scene
// (Ignore this for now) In our case, we might want to consider also the z coordinate of sample points when testing for separation
// (Ignore this for now) Otherwise, e.g., dark areas in the background might repel streamlines in lighter region on the foreground

// Try:
// d_sep_min = 0.1 mm
// d_sep_max = 9 mm
// d_sep (in mm) = (d_sep_max - d_sep_min) * lightness + d_sep_min
// d_test = 0.8 * d_sep
// d_step << d_test <= d_sep
// E.g., d_step = 0.1 * d_test

// 2. Seed point selection
// Start with a random streamline
// Generate a series of candidate seed points d_sep away from the streamline
// For each of them, generate a new streamline if allowed
// Add those new streamlines to a queue to be processed like the first one
// In our case, this might create problems as d_sep depends on lightness
// Thus, we might want to sub-divide space into smaller cells to find a good tradeoff between points to compare to and cells to visit

// 3. Equidistant streamline integration
// To measure consistent distance to a streamline by just calculating the distance to sample points, the sample points must be evenly spaced
// In 3D, this might be a problem since we select the sample points evenly spaced in 3D but their 2D projections, in general, won't be evenly spaced
// If we want to go 3D, we might need to re-sample in 2D

// Let's try this on a 2D LightDirectionDistanceCanvas first


pub fn gradient_streamline_segments(
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

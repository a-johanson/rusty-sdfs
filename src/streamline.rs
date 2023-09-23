use crate::canvas::PixelPropertyCanvas;
use crate::ray_marcher::RayMarcher;
use crate::scene::Scene;
use crate::vector::{vec2, vec3, Vec2, Vec3};


// *** Screen Space Streamlines

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

#[derive(Clone)]
pub struct StreamlineRegistryEntry {
    pub streamline_id: u32,
    pub point: Vec2,
}
pub struct StreamlineRegistry {
    width: f32,
    height: f32,
    cell_size: f32,
    cells_x: u32,
    cells_y: u32,
    next_streamline_id: u32,
    cell_content: Vec<Vec<StreamlineRegistryEntry>>,
}

impl StreamlineRegistry {
    pub fn new(width: u32, height: u32, cell_size: f32) -> StreamlineRegistry {
        let width = width as f32;
        let height = height as f32;
        let cells_x = (width / cell_size).ceil() as u32;
        let cells_y = (height / cell_size).ceil() as u32;
        let cell_content = vec![vec![]; (cells_x * cells_y) as usize];
        StreamlineRegistry {
            width,
            height,
            cell_size,
            cells_x,
            cells_y,
            next_streamline_id: 1,
            cell_content,
        }
    }

    fn cell_coordinates(&self, p: &Vec2) -> (u32, u32) {
        ((p.0 / self.cell_size) as u32, (p.1 / self.cell_size) as u32)
    }

    fn cell(&self, i_x: u32, i_y: u32) -> &Vec<StreamlineRegistryEntry> {
        let cell_idx = i_y * self.cells_x + i_x;
        self.cell_content.get(cell_idx as usize).unwrap()
    }

    fn cell_mut(&mut self, p: &Vec2) -> &mut Vec<StreamlineRegistryEntry> {
        let cell_idx = self.cell_index(p);
        self.cell_content.get_mut(cell_idx as usize).unwrap()
    }

    fn cell_index(&self, p: &Vec2) -> u32 {
        let (i_x, i_y) = self.cell_coordinates(p);
        i_y * self.cells_x + i_x
    }

    pub fn add_streamline(&mut self, streamline: &[Vec2]) -> u32 {
        let streamline_id = self.next_streamline_id;
        self.next_streamline_id += 1;
        for p in streamline {
            let cc = self.cell_mut(p);
            cc.push(StreamlineRegistryEntry {
                streamline_id,
                point: *p,
            });
        }
        streamline_id
    }

    pub fn is_point_allowed(
        &self,
        p: &Vec2,
        d_sep: f32,
        d_sep_relaxed: f32,
        relaxed_streamline_id: u32,
    ) -> bool {
        let cell_radius = (d_sep / self.cell_size).ceil() as u32;
        let (i_x_cell, i_y_cell) = self.cell_coordinates(p);
        let i_x_min = i_x_cell.saturating_sub(cell_radius);
        let i_x_max = (i_x_cell + cell_radius).min(self.cells_x - 1);
        let i_y_min = i_y_cell.saturating_sub(cell_radius);
        let i_y_max = (i_y_cell + cell_radius).min(self.cells_y - 1);

        for i_y in i_y_min..=i_y_max {
            for i_x in i_x_min..=i_x_max {
                let cell = self.cell(i_x, i_y);
                for candidate in cell {
                    let min_dist = if candidate.streamline_id == relaxed_streamline_id {
                        d_sep_relaxed
                    } else {
                        d_sep
                    };
                    if vec2::dist(p, &candidate.point) < min_dist {
                        return false;
                    }
                }
            }
        }
        true
    }
}

pub fn streamline_d_sep_from_lightness(d_sep_min: f32, d_sep_max: f32, lightness: f32) -> f32 {
    (d_sep_max - d_sep_min) * lightness * lightness * lightness + d_sep_min
}

pub fn flow_field_streamline(
    canvas: &PixelPropertyCanvas,
    streamline_registry: &StreamlineRegistry,
    start_from_streamline_id: u32,
    p_start: &Vec2,
    d_sep_min: f32,
    d_sep_max: f32,
    d_test_factor: f32,
    d_step: f32,
    max_depth_step: f32,
    max_accum_angle: f32,
    max_steps: u32,
    min_steps: u32,
) -> Option<Vec<Vec2>> {
    let pv_start = canvas.pixel_value(p_start.0, p_start.1);
    if pv_start.is_none() {
        return None;
    }

    let pv_start = pv_start.unwrap();
    let d_sep = streamline_d_sep_from_lightness(d_sep_min, d_sep_max, pv_start.lightness);
    if !streamline_registry.is_point_allowed(
        p_start,
        d_sep,
        d_test_factor * d_sep,
        start_from_streamline_id,
    ) {
        return None;
    }

    fn continue_line(
        canvas: &PixelPropertyCanvas,
        streamline_registry: &StreamlineRegistry,
        p_start: &Vec2,
        direction_start: f32,
        depth_start: f32,
        d_sep_min: f32,
        d_sep_max: f32,
        d_test_factor: f32,
        d_step: f32,
        max_depth_step: f32,
        max_accum_angle: f32,
        max_steps: u32,
    ) -> Vec<Vec2> {
        let mut line: Vec<Vec2> = Vec::new();
        let mut p_last = *p_start;
        let mut next_direction = direction_start;
        let mut last_depth = depth_start;
        let mut accum_angle = 0.0f32;

        for _ in 0..max_steps {
            let next_dir_uv = vec2::polar_angle_to_unit_vector(next_direction);
            let p_new = vec2::scale_and_add(&p_last, &next_dir_uv, d_step);
            let pv_new = canvas.pixel_value(p_new.0, p_new.1);
            if pv_new.is_none() {
                break;
            }

            let pv_new = pv_new.unwrap();
            let new_dir_uv = vec2::polar_angle_to_unit_vector(pv_new.direction);
            accum_angle += vec2::dot(&next_dir_uv, &new_dir_uv).clamp(-1.0, 1.0).acos();
            let d_sep = d_test_factor
                * streamline_d_sep_from_lightness(d_sep_min, d_sep_max, pv_new.lightness);
            if accum_angle > max_accum_angle
                || (pv_new.depth - last_depth).abs() > max_depth_step
                || !streamline_registry.is_point_allowed(&p_new, d_sep, d_sep, 0)
            {
                break;
            }

            line.push(p_new);
            p_last = p_new;
            next_direction = pv_new.direction;
            last_depth = pv_new.depth;
        }
        line
    }

    let line_with_direction = continue_line(
        canvas,
        streamline_registry,
        p_start,
        pv_start.direction,
        pv_start.depth,
        d_sep_min,
        d_sep_max,
        d_test_factor,
        d_step,
        max_depth_step,
        0.5 * max_accum_angle,
        max_steps / 2,
    );
    let line_against_direction = continue_line(
        canvas,
        streamline_registry,
        p_start,
        pv_start.direction,
        pv_start.depth,
        d_sep_min,
        d_sep_max,
        d_test_factor,
        -d_step,
        max_depth_step,
        0.5 * max_accum_angle,
        max_steps / 2,
    );
    let line_midpoint = [*p_start];

    let line: Vec<Vec2> = line_against_direction
        .iter()
        .rev()
        .chain(line_midpoint.iter())
        .chain(line_with_direction.iter())
        .cloned()
        .collect();

    if line.len() > (min_steps + 1) as usize {
        Some(line)
    } else {
        None
    }
}



// *** World Space Streamlines

pub fn gradient_streamline_segments(
    ray_marcher: &RayMarcher,
    scene: &impl Scene,
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
    let mut n_prev = RayMarcher::scene_normal(scene, &p_prev);
    let mut i: u32 = 0;
    while i < step_count {
        // Construct an orthonormal basis (u, v) of the plane defined by the normal at p_prev
        let plane_basis =
            vec3::orthonormal_basis_of_plane(&n_prev, &vec3::sub(light_point_source, &p_prev));
        if plane_basis.is_none() {
            println!("WARNING: cannot construct orthonormal basis of tangent plane");
            break;
        }
        let (u, v) = plane_basis.unwrap();

        let surface_dir =
            vec3::scale_and_add_inplace(vec3::scale(&u, cos_hatch_angle), &v, sin_hatch_angle);

        let p_next = vec3::scale_and_add(&p_prev, &surface_dir, step_scale);
        let n_next = RayMarcher::scene_normal(scene, &p_next);
        let visibility = 1.0; //RayMarcher::visibility_factor(sdf, &ray_marcher.camera, &p_next, Some(&n_next));

        if visibility > 0.0 {
            segments
                .last_mut()
                .unwrap()
                .push(ray_marcher.to_screen_coordinates(&p_next));
        } else if !segments.last().unwrap().is_empty() {
            segments.push(Vec::<Vec2>::new());
        }

        p_prev = p_next;
        n_prev = n_next;

        i += 1;
    }
    segments
}

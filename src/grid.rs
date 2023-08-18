use rand::{Rng, RngCore};


pub fn on_grid<F>(width: f32, height: f32, cell_count_x: u32, cell_count_y: u32, mut f: F)
where
    F: FnMut(f32, f32, f32, f32) -> ()
{
    let cell_width = width / (cell_count_x as f32);
    let cell_height = height / (cell_count_y as f32);
    for i_y in 0..cell_count_y {
        for i_x in 0..cell_count_x {
            let x = cell_width * (i_x as f32);
            let y = cell_height * (i_y as f32);
            f(x, y, cell_width, cell_height);
        }
    }
}

pub fn on_jittered_grid<F>(width: f32, height: f32, cell_count_x: u32, cell_count_y: u32, rng: &mut dyn RngCore, mut f: F)
where
    F: FnMut(f32, f32) -> ()
{
    let cell_width = width / (cell_count_x as f32);
    let cell_height = height / (cell_count_y as f32);
    for i_y in 0..cell_count_y {
        for i_x in 0..cell_count_x {
            let x = cell_width * ((i_x as f32) + rng.gen::<f32>());
            let y = cell_height * ((i_y as f32) + rng.gen::<f32>());
            f(x, y);
        }
    }
}

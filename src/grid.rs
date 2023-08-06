
pub fn on_grid<F>(width: f32, height: f32, cells_x: u32, cells_y: u32, mut f: F)
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

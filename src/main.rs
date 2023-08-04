mod canvas;
mod ray_marcher;

use std::path::Path;

use canvas::Canvas;

fn main() {
    let mut canvas = Canvas::new(600, 800);

    canvas.stroke_line_segments(&[
        [100.0, 100.0],
        [500.0, 200.0],
        [200.0, 400.0],
    ]);

    canvas.save_png(Path::new("out.png"));
}

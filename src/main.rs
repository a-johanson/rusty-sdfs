mod canvas;

use std::path::Path;

use canvas::Canvas;

fn main() {
    let mut canvas = Canvas::new(600, 800);

    canvas.stroke_line_segments();

    canvas.save_png(Path::new("out.png"));
}

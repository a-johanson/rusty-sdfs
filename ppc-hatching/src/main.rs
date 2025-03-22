#![allow(dead_code)]

use std::f32::consts::PI;
use std::path::Path;
use std::time::Instant;

use rusty_sdfs_lib::render_edges;
use rusty_sdfs_lib::render_hatch_lines;
use rusty_sdfs_lib::Canvas;
use rusty_sdfs_lib::PixelPropertyCanvas;
use rusty_sdfs_lib::SkiaCanvas;

fn main() {
    const STROKE_WIDTH_IN_MM: f32 = 0.15;
    const DPI: f32 = 200.0;

    const INCH_PER_CM: f32 = 1.0 / 2.54;
    const INCH_PER_MM: f32 = 0.1 / 2.54;
    const STROKE_WIDTH: f32 = STROKE_WIDTH_IN_MM * INCH_PER_MM * DPI;

    let pp_canvas = PixelPropertyCanvas::from_file("trees.ppc").unwrap();

    println!(
        "Hatching on a canvas of size {} px x {} px using a stroke width of {} px...",
        pp_canvas.width(), pp_canvas.height(), STROKE_WIDTH
    );
    let start_instant = Instant::now();
    let mut output_canvas = SkiaCanvas::new(pp_canvas.width(), pp_canvas.height());//pp_canvas.direction_to_skia_canvas();
    let step_size = 0.5;
    let separation = 5.0;
    let line_color = [0, 0, 0];
    let line_width = 1.0;
    render_hatch_lines(&pp_canvas, &mut output_canvas, 0.85, step_size, &line_color, line_width, 0.2*PI, separation);
    render_hatch_lines(&pp_canvas, &mut output_canvas, 0.5, step_size, &line_color, line_width, 0.55*PI, 0.75 * separation);
    render_hatch_lines(&pp_canvas, &mut output_canvas, 0.25, step_size, &line_color, line_width, 0.85*PI, 0.3 * separation);

    render_edges(&pp_canvas, &mut output_canvas, &[0, 0, 0], line_width);

    let duraction_hatching = start_instant.elapsed();
    println!(
        "Finished hatching after {} seconds",
        duraction_hatching.as_secs_f32()
    );

    println!("Outputting image(s) to disk/display...");
    // output_canvas.save_png(Path::new("output_trees.png"));
    output_canvas.display_in_window("ppc hatching");
    println!("Done");
}

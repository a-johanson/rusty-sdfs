#![allow(dead_code)]

use rusty_sdfs_lib::noisy_waves_heightmap;
use rusty_sdfs_lib::render_heightmap_streamlines;
use rusty_sdfs_lib::vec2;
use rusty_sdfs_lib::DomainRegion;
use rusty_sdfs_lib::LinearGradient;
use rusty_sdfs_lib::SkiaCanvas;


fn main() {
    const WIDTH_IN_CM: f32 = 11.0;
    const HEIGHT_IN_CM: f32 = 16.0;
    const STROKE_WIDTH_IN_MM: f32 = 0.15;
    const LINE_SEP_IN_MM: f32 = 0.9;
    const SEGMENT_LENGTH_IN_DOTS: f32 = 2.0;
    const DPI: f32 = 300.0;

    const INCH_PER_CM: f32 = 1.0 / 2.54;
    const INCH_PER_MM: f32 = 0.1 / 2.54;

    let width = (WIDTH_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let height = (HEIGHT_IN_CM * INCH_PER_CM * DPI).round() as u32;
    let line_count = (10.0 * HEIGHT_IN_CM / LINE_SEP_IN_MM).round() as u32;
    let buffer_count_near = line_count / 2;
    let buffer_count_far = 10 * line_count;
    let segment_count = (width as f32 / SEGMENT_LENGTH_IN_DOTS).round() as u32;
    let line_width = STROKE_WIDTH_IN_MM * INCH_PER_MM * DPI;

    println!("Draw on {} px x {} px canvas with line width {} px, {} lines, {} segments per line", width, height, line_width, line_count, segment_count);

    let mut canvas = SkiaCanvas::new(width, height);

    let domain = DomainRegion::new(
        &vec2::from_values(1.5, 2.5),
        &vec2::from_values(0.0, 10.0),
        18.0,
        4.1,
        14.0
    );
    // let domain = DomainRegion {
    //     near_a: vec2::from_values(-1.0, 1.0),
    //     near_b: vec2::from_values(1.0, 1.0),
    //     far_a: vec2::from_values(-3.0, 17.0),
    //     far_b: vec2::from_values(3.0, 17.0)
    // };

    // let black = [0x00, 0x00, 0x00];
    let white = [0xFF, 0xFF, 0xFF];
    let rich_black = [0x0D, 0x16, 0x1B];
    let gunmetal = [0x14, 0x26, 0x34];
    let paynes_gray = [0x21, 0x59, 0x6D];
    let platinum = [0xDD, 0xDE, 0xD8];
    // let gradient = LinearGradient::new(&white, &white);
    let mut gradient = LinearGradient::new(&rich_black, &platinum);
    gradient.add_stop(0.1, &gunmetal);
    gradient.add_stop(0.5, &paynes_gray);
    gradient.add_stop(0.9, &platinum);

    render_heightmap_streamlines(
        &mut canvas,
        &domain,
        line_count,
        buffer_count_near,
        buffer_count_far,
        segment_count,
        line_width,
        &white,
        &gradient,
        |uv_domain, t_domain, t_screen| {
            // let exp_decay = f32::exp(-t_domain.1);
            // let noise_scale = 0.2 * exp_decay.max(0.0) * exp_decay;
            let noise_scale = 0.15 * t_screen.1.max(0.0) * t_screen.1;
            let noise = noise_scale * noisy_waves_heightmap(uv_domain.0, uv_domain.1);
            let low_freq_scale = 0.5;
            // let low_freq = 0.0;
            let low_freq = low_freq_scale * 0.35 * (t_screen.1 + 0.3).min(1.0) * (3.0 * (t_screen.0 - 1.0 + 0.1 * t_domain.1)).cos();
            // let low_freq = low_freq_scale * 0.75 * t_screen.0;
            low_freq + noise
        }
    );

    // canvas.display_in_window("waves");
    canvas.save_png(&std::path::Path::new("waves.png"));

}

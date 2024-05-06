use std::time::Duration;

use minifb::{Key, Window, WindowOptions};


pub trait Animation {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn frame_duration(&self) -> Duration;
    fn render_frame(&mut self) -> Vec<u32>;

    fn play(&mut self, title: &str, window_options: WindowOptions) {
        let mut window = Window::new(
            title,
            self.width() as usize,
            self.height() as usize,
            window_options
        )
        .unwrap();
        window.update(); // Ensure that the window is initialized before starting the animation
        window.limit_update_rate(Some(self.frame_duration()));
        while window.is_open() && !window.is_key_down(Key::Escape) {
            window.update_with_buffer(
                &self.render_frame(),
                self.width() as usize,
                self.height() as usize
            )
            .unwrap();
        }
    }
}

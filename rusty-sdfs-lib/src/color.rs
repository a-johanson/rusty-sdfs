use crate::{vec3, VecFloat};

pub struct LinearGradient {
    stops: Vec<(f32, [u8; 3])>
}

impl LinearGradient {
    pub fn new(start_rgb: &[u8; 3], end_rgb: &[u8; 3]) -> Self {
        Self {
            stops: vec![
                (0.0, *start_rgb),
                (1.0, *end_rgb),
            ],
        }
    }

    pub fn add_stop(&mut self, t: f32, rgb: &[u8; 3]) {
        if t <= 0.0 || t >= 1.0 {
            return;
        }
        self.stops.push((t, *rgb));
        self.stops.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    pub fn rgb(&self, t: f32) -> [u8; 3] {
        if t <= 0.0 {
            return self.stops[0].1;
        }

        for (prev, curr) in self.stops.iter().zip(self.stops.iter().skip(1)) {
            if t <= curr.0 {
                let t_a = prev.0;
                let t_b = curr.0;
                let diff = t_b - t_a;
                if diff.abs() < 1.0e-7 {
                    return prev.1;
                }
                let t_relative = (t - t_a) / diff;
                let c_a = vec3::from_values(prev.1[0] as VecFloat, prev.1[1] as VecFloat, prev.1[2] as VecFloat);
                let c_b = vec3::from_values(curr.1[0] as VecFloat, curr.1[1] as VecFloat, curr.1[2] as VecFloat);
                let c = vec3::lerp(&c_a, &c_b, t_relative);
                return [
                    c.0 as u8,
                    c.1 as u8,
                    c.2 as u8,
                ];
            }
        }

        self.stops.last().unwrap().1
    }
}

use rusty_sdfs_lib::{noise_2d, vec2, VecFloat};


pub fn waves_1d(x: VecFloat) -> VecFloat {
    1.0 - x.sin().abs()
}

pub fn waves_2d(x: VecFloat, y: VecFloat, pointiness: VecFloat) -> VecFloat {
    (waves_1d(x) * waves_1d(y)).powf(pointiness)
}

pub fn blob_heightmap(x: VecFloat, z: VecFloat) -> VecFloat {
    waves_1d(x) * waves_1d(z)
}

pub fn noise_heightmap(x: VecFloat, z: VecFloat) -> VecFloat {
    const NOISE_INPUT_SCALE: VecFloat = 0.35;
    noise_2d(NOISE_INPUT_SCALE * x, NOISE_INPUT_SCALE * z, 4)
}

pub fn noisy_waves_octave(x: VecFloat, y: VecFloat, pointiness: VecFloat) -> VecFloat {
    const NOISE_INPUT_SCALE: VecFloat = 0.45;
    const NOISE_SCALE: VecFloat = 1.75;
    const NOISE_OCTAVES: u32 = 4;
    const OFFSET1: VecFloat = 1000.5;
    const OFFSET2: VecFloat = 889.1;
    let x_shift = NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * x, NOISE_INPUT_SCALE * y, NOISE_OCTAVES);
    let y_shift = NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * x + OFFSET1, NOISE_INPUT_SCALE * y + OFFSET2, NOISE_OCTAVES);
    const ADDED_NOISE_SCALE: VecFloat = 0.15;
    waves_2d(x + x_shift, y + y_shift, pointiness) + ADDED_NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * x - OFFSET2, NOISE_INPUT_SCALE * y - OFFSET1, NOISE_OCTAVES)
}

pub fn noisy_waves(x: VecFloat, y: VecFloat) -> VecFloat {
    const POINTINESS: VecFloat = 0.9;
    const OCTAVES: u32 = 3;
    let mut accum = noisy_waves_octave(x, y, POINTINESS);
    let mut scale: VecFloat = 1.0;
    let mut p = vec2::from_values(x, y);
    for _ in 1..OCTAVES {
        p = vec2::rotate_trig_inplace(p, 1.7 * (12.0/13.0), 1.7 * (5.0/13.0));
        scale *= 0.5;
        accum += scale * noisy_waves_octave(p.0, p.1, POINTINESS);
    }
    0.35 * accum
}

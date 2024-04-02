use rusty_sdfs_lib::{noise_2d, VecFloat};


pub fn waves_1d(x: VecFloat) -> VecFloat {
    1.0 - x.sin().abs()
}

pub fn waves_2d(x: VecFloat, y: VecFloat, pointiness: VecFloat) -> VecFloat {
    (waves_1d(x) * waves_1d(y)).powf(pointiness)
}

pub fn noisy_waves_octave(x: VecFloat, y: VecFloat, pointiness: VecFloat) -> VecFloat {
    const NOISE_INPUT_SCALE: VecFloat = 0.55;
    const NOISE_SCALE: VecFloat = 0.5;
    const NOISE_OCTAVES: u32 = 4;
    const YX_OFFSET: VecFloat = 1000.0;
    const YY_OFFSET: VecFloat = 889.0;
    let x_shift = NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * x, NOISE_INPUT_SCALE * y, NOISE_OCTAVES);
    let y_shift = NOISE_SCALE * noise_2d(NOISE_INPUT_SCALE * x + YX_OFFSET, NOISE_INPUT_SCALE * y + YY_OFFSET, NOISE_OCTAVES);
    waves_2d(x + x_shift, y + y_shift, pointiness)
}

pub fn noisy_waves(x: VecFloat, y: VecFloat, octaves: u32) -> VecFloat {
    noisy_waves_octave(x, y, 0.8)
}

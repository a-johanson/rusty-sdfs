use wyhash::wyhash;

use crate::vector::{vec2, VecFloat};

const WYHASH_DEFAULT_SEED1: u64 = 14678021983192906369;
const WYHASH_DEFAULT_SEED2: u64 = 601104623970451784;
const WYHASH_DEFAULT_SEED3: u64 = 82545205824138771;

pub fn smoothstep(t: VecFloat) -> VecFloat {
    t * t * (3.0 - 2.0 * t)
}

pub fn waves_1d(x: VecFloat) -> VecFloat {
    1.0 - x.sin().abs()
}

pub fn waves_2d(x: VecFloat, y: VecFloat, pointiness: VecFloat) -> VecFloat {
    (waves_1d(x) * waves_1d(y)).powf(pointiness)
}

pub fn rand_1d(x: VecFloat, seed: u64) -> VecFloat {
    let bytes = x.to_le_bytes();
    2.0 * ((wyhash(&bytes, seed) as VecFloat) / (u64::MAX as VecFloat)) - 1.0
}

pub fn rand_2d(x: f32, y: f32, seed: u64) -> VecFloat {
    let x_bytes = x.to_le_bytes();
    let y_bytes = y.to_le_bytes();
    let bytes = [
        x_bytes[0], x_bytes[1], x_bytes[2], x_bytes[3],
        y_bytes[0], y_bytes[1], y_bytes[2], y_bytes[3],
    ];
    2.0 * ((wyhash(&bytes, seed) as VecFloat) / (u64::MAX as VecFloat)) - 1.0
}

pub fn rand_3d(x: f32, y: f32, z: f32, seed: u64) -> VecFloat {
    let x_bytes = x.to_le_bytes();
    let y_bytes = y.to_le_bytes();
    let z_bytes = z.to_le_bytes();
    let bytes = [
        x_bytes[0], x_bytes[1], x_bytes[2], x_bytes[3],
        y_bytes[0], y_bytes[1], y_bytes[2], y_bytes[3],
        z_bytes[0], z_bytes[1], z_bytes[2], z_bytes[3],
    ];
    2.0 * ((wyhash(&bytes, seed) as VecFloat) / (u64::MAX as VecFloat)) - 1.0
}

fn noise_1d_octave(x: VecFloat) -> VecFloat {
    let idx = x.floor();
    let t = x - idx;
    

    let v0 = 0.5 * rand_1d(idx, WYHASH_DEFAULT_SEED1);
    let v1 = 0.5 * rand_1d(idx + 1.0, WYHASH_DEFAULT_SEED1);
    let g0 = rand_1d(idx, WYHASH_DEFAULT_SEED2);
    let g1 = rand_1d(idx + 1.0, WYHASH_DEFAULT_SEED2);

    let f0 = g0 * t + v0;
    let f1 = g1 * (t - 1.0) + v1;

    let u = smoothstep(t);
    f0 * (1.0 - u) + f1 * u
}


fn noise_2d_octave(x: VecFloat, y: VecFloat) -> VecFloat {
    let ix = x.floor();
    let tx = x - ix;
    let iy = y.floor();
    let ty = y - iy;

    let ix0 = ix;
    let ix1 = ix + 1.0;
    let iy0 = iy;
    let iy1 = iy + 1.0;

    // Function values at each corner
    let v00 = 0.5 * rand_2d(ix0, iy0, WYHASH_DEFAULT_SEED1);
    let v01 = 0.5 * rand_2d(ix1, iy0, WYHASH_DEFAULT_SEED1);
    let v10 = 0.5 * rand_2d(ix0, iy1, WYHASH_DEFAULT_SEED1);
    let v11 = 0.5 * rand_2d(ix1, iy1, WYHASH_DEFAULT_SEED1);

    // Gradients at each corner
    let g00 = vec2::from_values(rand_2d(ix0, iy0, WYHASH_DEFAULT_SEED2), rand_2d(ix0, iy0, WYHASH_DEFAULT_SEED3));
    let g01 = vec2::from_values(rand_2d(ix1, iy0, WYHASH_DEFAULT_SEED2), rand_2d(ix1, iy0, WYHASH_DEFAULT_SEED3));
    let g10 = vec2::from_values(rand_2d(ix0, iy1, WYHASH_DEFAULT_SEED2), rand_2d(ix0, iy1, WYHASH_DEFAULT_SEED3));
    let g11 = vec2::from_values(rand_2d(ix1, iy1, WYHASH_DEFAULT_SEED2), rand_2d(ix1, iy1, WYHASH_DEFAULT_SEED3));

    // The respective function values at (tx, ty) assuming each corner was associated
    // with an affine function with value v__ at the corner and the gradient g__
    let f00 = vec2::dot(&g00, &vec2::from_values(tx, ty)) + v00;
    let f01 = vec2::dot(&g01, &vec2::from_values(tx - 1.0, ty)) + v01;
    let f10 = vec2::dot(&g10, &vec2::from_values(tx, ty - 1.0)) + v10;
    let f11 = vec2::dot(&g11, &vec2::from_values(tx - 1.0, ty - 1.0)) + v11;

    // Bilinear interpolation
    let ux = smoothstep(tx);
    let f0 = f00 * (1.0 - ux) + f01 * ux;
    let f1 = f10 * (1.0 - ux) + f11 * ux;

    let uy = smoothstep(ty);
    f0 * (1.0 - uy) + f1 * uy
}

pub fn noise_2d(x: VecFloat, y: VecFloat, octaves: u32) -> VecFloat {
    let mut accum = noise_2d_octave(x, y);
    let mut scale: VecFloat = 1.0;
    let mut p = vec2::from_values(x, y);
    for _ in 1..octaves {
        p = vec2::rotate_trig_inplace(p, 2.0 * (12.0/13.0), 2.0 * (5.0/13.0));
        scale *= 0.5;
        accum += scale * noise_2d_octave(p.0, p.1);
    }
    accum
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

pub fn noisy_waves_heightmap(x: VecFloat, y: VecFloat) -> VecFloat {
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
    accum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_1d() {
        const N: i64 = 1000000;
        const MAX_MEAN: f64 = 1.0e-3;
        const MAX_COLLISION_SHARE: f64 = 1.0e-5;
        let mut samples: u64 = 0;
        let mut accum: f64 = 0.0;
        let mut collisions_value: u64 = 0;
        let mut collisions_seed: u64 = 0;
        for ix in -N..N {
            let x = ix as VecFloat;
            let r1 = rand_1d(x, WYHASH_DEFAULT_SEED1);
            let r11 = rand_1d(x + 1.0, WYHASH_DEFAULT_SEED1);
            let r2 = rand_1d(x, WYHASH_DEFAULT_SEED2);
            samples += 1;
            accum += r1 as f64;
            if r1 == r11 {
                collisions_value += 1;
                println!("Warning for rand_1d: same output for different inputs (x={x})");
            }
            if r1 == r2 {
                collisions_seed += 1;
                println!("Warning for rand_1d: same output for different seeds (x={x})");
            }
            assert!(r1 >= -1.0 && r1 <= 1.0);
            assert!(r11 >= -1.0 && r11 <= 1.0);
            assert!(r2 >= -1.0 && r2 <= 1.0);
        }
        let mean = accum / samples as f64;
        println!("Info for rand_1d: mean = {mean}");
        assert!(mean.abs() <= MAX_MEAN);
        let collision_share_value = collisions_value as f64 / samples as f64;
        println!("Info for rand_1d: collision share for different inputs = {collision_share_value}");
        assert!(collision_share_value <= MAX_COLLISION_SHARE);
        let collision_share_seed = collisions_seed as f64 / samples as f64;
        println!("Info for rand_1d: collision share for different seeds = {collision_share_seed}");
        assert!(collision_share_seed <= MAX_COLLISION_SHARE);

    }

    #[test]
    fn test_rand_2d() {
        const N: i64 = 1000;
        const MAX_MEAN: f64 = 1.0e-3;
        const MAX_COLLISION_SHARE: f64 = 1.0e-5;
        let mut samples: u64 = 0;
        let mut accum: f64 = 0.0;
        let mut collisions_value: u64 = 0;
        let mut collisions_seed: u64 = 0;
        for iy in -N..N {
            let y = iy as f32;
            for ix in -N..N {
                let x = ix as f32;
                let r1 = rand_2d(x, y, WYHASH_DEFAULT_SEED1);
                let r11 = rand_2d(x + 1.0, y, WYHASH_DEFAULT_SEED1);
                let r2 = rand_2d(x, y, WYHASH_DEFAULT_SEED2);
                samples += 1;
                accum += r1 as f64;
                if r1 == r11 {
                    collisions_value += 1;
                    println!("Warning for rand_2d: same output for different inputs (x={x},y={y})");
                }
                if r1 == r2 {
                    collisions_seed += 1;
                    println!("Warning for rand_2d: same output for different seeds (x={x},y={y})");
                }
                assert!(r1 >= -1.0 && r1 <= 1.0);
                assert!(r11 >= -1.0 && r11 <= 1.0);
                assert!(r2 >= -1.0 && r2 <= 1.0);
            }
        }
        let mean = accum / samples as f64;
        println!("Info for rand_2d: mean = {mean}");
        assert!(mean.abs() <= MAX_MEAN);
        let collision_share_value = collisions_value as f64 / samples as f64;
        println!("Info for rand_2d: collision share for different inputs = {collision_share_value}");
        assert!(collision_share_value <= MAX_COLLISION_SHARE);
        let collision_share_seed = collisions_seed as f64 / samples as f64;
        println!("Info for rand_2d: collision share for different seeds = {collision_share_seed}");
        assert!(collision_share_seed <= MAX_COLLISION_SHARE);
    }

    #[test]
    fn test_rand_3d() {
        const N: i64 = 100;
        const MAX_MEAN: f64 = 1.0e-3;
        const MAX_COLLISION_SHARE: f64 = 1.0e-5;
        let mut samples: u64 = 0;
        let mut accum: f64 = 0.0;
        let mut collisions_value: u64 = 0;
        let mut collisions_seed: u64 = 0;
        for iz in -N..N {
            let z = iz as f32;
            for iy in -N..N {
                let y = iy as f32;
                for ix in -N..N {
                    let x = ix as f32;
                    let r1 = rand_3d(x, y, z, WYHASH_DEFAULT_SEED1);
                    let r11 = rand_3d(x + 1.0, y, z, WYHASH_DEFAULT_SEED1);
                    let r2 = rand_3d(x, y, z, WYHASH_DEFAULT_SEED2);
                    samples += 1;
                    accum += r1 as f64;
                    if r1 == r11 {
                        collisions_value += 1;
                        println!("Warning for rand_3d: same output for different inputs (x={x},y={y},z={z})");
                    }
                    if r1 == r2 {
                        collisions_seed += 1;
                        println!("Warning for rand_3d: same output for different seeds (x={x},y={y},z={z})");
                    }
                    assert!(r1 >= -1.0 && r1 <= 1.0);
                    assert!(r11 >= -1.0 && r11 <= 1.0);
                    assert!(r2 >= -1.0 && r2 <= 1.0);
                }
            }
        }
        let mean = accum / samples as f64;
        println!("Info for rand_3d: mean = {mean}");
        assert!(mean.abs() <= MAX_MEAN);
        let collision_share_value = collisions_value as f64 / samples as f64;
        println!("Info for rand_3d: collision share for different inputs = {collision_share_value}");
        assert!(collision_share_value <= MAX_COLLISION_SHARE);
        let collision_share_seed = collisions_seed as f64 / samples as f64;
        println!("Info for rand_3d: collision share for different seeds = {collision_share_seed}");
        assert!(collision_share_seed <= MAX_COLLISION_SHARE);
    }
}

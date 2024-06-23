
use fastnoise_lite::{FastNoiseLite, NoiseType};
use lazy_static::lazy_static;

fn init_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise
}

pub fn get_noise_3d(x: f64, y: f64, z: f64) -> f64 {
    let raw_noise = NOISE_GENERATOR.get_noise_3d(x, y, z) as f64;
    raw_noise
    // let min_noise = -0.9999828338623047;
    // let max_noise = -0.11478698253631592;
    // let normalized_noise = 2.0 * ((raw_noise - min_noise) / (max_noise - min_noise)) - 1.0;
    // normalized_noise
}

pub fn octave_perlin(x: f64, y: f64, z: f64, octaves: usize, persistence: f64) -> f64 {
    let mut total: f64 = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0; // Used for normalizing result to 0.0 - 1.0
    for _ in 0..octaves {
        total += get_noise_3d(x * frequency, y * frequency, z * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    total / max_value
}

lazy_static! {
    pub static ref NOISE_GENERATOR: FastNoiseLite = init_noise();
}


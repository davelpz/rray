
use fastnoise_lite::{FastNoiseLite, NoiseType};
use lazy_static::lazy_static;

fn init_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise
}

/// Generates a 3D noise value using a pre-initialized noise generator.
///
/// This function computes a 3D Perlin noise value for the given coordinates. The noise value
/// is generated using a globally available `NOISE_GENERATOR` which is initialized with Perlin
/// noise settings. The function returns a raw noise value as a `f64`.
///
/// # Arguments
///
/// * `x` - The x-coordinate in 3D space.
/// * `y` - The y-coordinate in 3D space.
/// * `z` - The z-coordinate in 3D space.
///
/// # Returns
///
/// Returns a `f64` representing the raw noise value at the given 3D coordinates.
pub fn get_noise_3d(x: f64, y: f64, z: f64) -> f64 {
    let raw_noise = NOISE_GENERATOR.get_noise_3d(x, y, z) as f64;
    raw_noise
}

/// Generates a fractal noise value using the Perlin noise algorithm with octaves.
///
/// This function computes a fractal noise value at the specified 3D coordinates using an
/// octave-based approach. It aggregates multiple layers of noise, each with increasing frequency
/// and decreasing amplitude, to create a more complex noise pattern. The result is normalized
/// to a range of 0.0 to 1.0. This technique is often used to generate natural-looking textures
/// and terrain features.
///
/// # Arguments
///
/// * `x` - The x-coordinate in 3D space.
/// * `y` - The y-coordinate in 3D space.
/// * `z` - The z-coordinate in 3D space.
/// * `octaves` - The number of layers of noise to combine.
/// * `persistence` - The rate at which the amplitude of each successive layer decreases.
///
/// # Returns
///
/// Returns a `f64` representing the normalized fractal noise value at the given 3D coordinates.
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


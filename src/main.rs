extern crate lazy_static;

use crate::raytracer::scene_builder_yaml::render_scene_from_file;
use clap::Parser;

mod tuple;
mod color;
mod matrix;
mod raytracer;
pub const EPSILON: f64 = 0.00001; // Small value used for floating-point comparisons

/// Validates that the provided value is less than or equal to the max allowed value.
///
/// # Arguments
///
/// * `val` - A string slice that holds the value to validate.
/// * `max_value` - The maximum allowed value.
///
/// # Returns
///
/// * `Ok(())` if the value is valid,
/// * `Err(String)` if the value is not a positive number or exceeds `max_value`.
fn validate_max_value(val: &str, max_value: usize) -> Result<(), String> {
    let parsed: usize = val.parse().map_err(|_| format!("must be a positive number"))?;
    if parsed > max_value {
        Err(format!("value must be less than or equal to {}", max_value))
    } else {
        Ok(())
    }
}

/// Validates the anti-aliasing (AA) level provided by the user.
///
/// # Arguments
///
/// * `s` - A string slice that holds the AA level to validate.
///
/// # Returns
///
/// * `Ok(usize)` if the AA level is valid,
/// * `Err(String)` if the AA level is not a positive number, not a number, or exceeds the max allowed value (5).
fn validate_aa(s: &str) -> Result<usize, String> {
    validate_max_value(s, 5).and_then(|_| s.parse().map_err(|_| "expected a number".to_string()))
}

/// Simple raytracer application.
///
/// Parses command line arguments to configure and render a scene described in a YAML file.
#[derive(Parser, Debug)]
#[command(version = "1.0", about = "A simple raytracer", long_about = None)]
struct Args {
    /// Width of the generated image, default is 800
    #[arg(short = 'W', long, default_value_t = 800)]
    width: usize,

    /// Height of the generated image, default is 600
    #[arg(short = 'H', long, default_value_t = 600)]
    height: usize,

    /// Scene file in YAML format
    #[arg(short, long)]
    scene: String,

    /// Name of the output file, default is output.png
    #[arg(short, long, default_value = "output.png")]
    output: String,

    /// Anti-aliasing level (default 1) (max 5)
    #[arg(short, long, default_value_t = 1, value_parser = validate_aa)]
    aa: usize,
}

fn main() {
   let args = Args::parse();

    // Render the scene based on the provided command line arguments
    render_scene_from_file(&args.scene, args.width, args.height, &args.output, args.aa);
}
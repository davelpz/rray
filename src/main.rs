extern crate lazy_static;

use crate::raytracer::scene_builder_yaml::render_scene_from_file;
use clap::Parser;

mod tuple;
mod color;
mod canvas;
mod matrix;
mod raytracer;
pub const EPSILON: f64 = 0.00001;

/// Validates that the provided value is less than or equal to the max allowed value.
fn validate_max_value(val: &str, max_value: usize) -> Result<(), String> {
    let parsed: usize = val.parse().map_err(|_| format!("must be a positive number"))?;
    if parsed > max_value {
        Err(format!("value must be less than or equal to {}", max_value))
    } else {
        Ok(())
    }
}

fn validate_aa(s: &str) -> Result<usize, String> {
    validate_max_value(s, 5).and_then(|_| s.parse().map_err(|_| "expected a number".to_string()))
}

/// Simple raytracer
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

    render_scene_from_file(&args.scene, args.width, args.height, &args.output, args.aa);
}
extern crate lazy_static;

use crate::raytracer::scene_builder_yaml::render_scene_from_file;

mod tuple;
mod color;
mod canvas;
mod matrix;
mod raytracer;
pub const EPSILON: f64 = 0.00001;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 4 {
        let width: usize = args[1].parse().expect("Failed to parse width");
        let height: usize = args[2].parse().expect("Failed to parse height");
        render_scene_from_file(&args[3], width, height, &args[4]);
    } else {
        println!("Usage: cargo run -- <width> <height> <scene.yaml> <output.png>");
    }
}
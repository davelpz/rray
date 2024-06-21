extern crate lazy_static;

use crate::scene::create_scene_from_file;
use crate::world_builder::render_scene;

mod tuple;
mod color;
mod canvas;
mod matrix;
mod ray;
mod shape;
mod light;
mod material;
mod world;
mod camera;
mod scene;
mod pattern;
mod world_builder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 4 {
        let width: usize = args[1].parse().expect("Failed to parse width");
        let height: usize = args[2].parse().expect("Failed to parse height");
        let scene = create_scene_from_file(&args[3]);
        match scene {
            Some(s) => {
                render_scene(s,  width, height, &args[4])
            },
            None => println!("Failed to create scene from file"),
        }
    } else {
        println!("Usage: cargo run -- <width> <height> <scene.json> <output.png>");
    }
}
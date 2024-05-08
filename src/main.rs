mod tuple;
mod color;
mod canvas;
mod matrix;
mod ray;
mod shape;

use crate::canvas::canvas::Canvas;
use crate::color::color::Color;

#[allow(unused_variables)]
fn main() {
    println!("Hello, world!");
    let mut canvas = Canvas::new(2, 2);
    canvas.write_pixel(0, 0, Color::new(1.0, 0.0, 0.0));
    canvas.write_pixel(1, 0, Color::new(0.0, 0.0, 1.0));
    canvas.write_pixel(0, 1, Color::new(0.0, 1.0, 0.0));
    canvas.write_pixel(1, 1, Color::new(0.0, 1.0, 1.0));
    canvas.write_to_file("canvas.png");
}

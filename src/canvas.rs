#![allow(dead_code)]

use std::io::BufWriter;
use png::Encoder;
use std::fs::File;
use std::path::Path;
use crate::color::Color;

#[derive(Debug, PartialEq)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let pixels = vec![Color::new(0.0, 0.0, 0.0); width * height];
        Canvas { width, height, pixels }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = y * self.width + x;
        self.pixels[index] = color;
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> Color {
        let index = y * self.width + x;
        self.pixels[index].clone()
    }

    pub fn write_to_file(&self, filename: &str) {
        let path = Path::new(filename);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = Encoder::new(w, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let mut data = Vec::new();
        for pixel in &self.pixels {
            data.push((pixel.r * 255.0) as u8);
            data.push((pixel.g * 255.0) as u8);
            data.push((pixel.b * 255.0) as u8);
            data.push(255u8);
        }
        writer.write_image_data(&data).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::Canvas;
    use crate::color::Color;

    #[test]
    fn test_canvas() {
        let c = Canvas::new(10, 20);
        assert_eq!(c.width, 10);
        assert_eq!(c.height, 20);
        assert_eq!(c.pixels.len(), 10 * 20);
        for pixel in c.pixels {
            assert_eq!(pixel, Color::new(0.0, 0.0, 0.0));
        }
    }

    #[test]
    fn test_write_pixel() {
        let mut c = Canvas::new(10, 20);
        let red = Color::new(1.0, 0.0, 0.0);
        c.write_pixel(2, 3, red);
        let red = Color::new(1.0, 0.0, 0.0);
        assert_eq!(c.pixel_at(2, 3), red);
    }
}
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
        self.pixels[index]
    }

    fn get_u8_colors(&self, aa: usize) -> Vec<u8> {
        let mut data = Vec::new();
        let total_pixels = (aa * aa) as f64;
        for y in (0..self.height).step_by(aa) {
            for x in (0..self.width).step_by(aa) {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;

                for dy in 0..aa {
                    for dx in 0..aa {
                        let pixel = self.pixel_at(x + dx, y + dy);
                        r += pixel.r;
                        g += pixel.g;
                        b += pixel.b;
                    }
                }

                r /= total_pixels;
                g /= total_pixels;
                b /= total_pixels;

                data.push((r * 255.0) as u8);
                data.push((g * 255.0) as u8);
                data.push((b * 255.0) as u8);
                data.push(255u8);
            }
        }
        data
    }


    pub fn write_to_file(&self, filename: &str, aa: usize) {
        let path = Path::new(filename);
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        let width = (self.width / aa) as u32;
        let height = (self.height / aa) as u32;
        let mut encoder = Encoder::new(w, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let data = self.get_u8_colors(aa);
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
#![allow(dead_code)]

use std::io::BufWriter;
use png::Encoder;
use std::fs::File;
use std::path::Path;
use crate::color::Color;

/// Represents a canvas for drawing in a ray tracing application.
///
/// This struct holds the dimensions of the canvas and a vector of pixels, where each pixel
/// is represented by a `Color`. The canvas acts as the drawing surface for the ray tracing
/// renderer, storing the color of each pixel as determined by the rendering process.
///
/// # Fields
///
/// * `width` - The width of the canvas in pixels.
/// * `height` - The height of the canvas in pixels.
/// * `pixels` - A vector of `Color` values representing the color of each pixel on the canvas.
#[derive(Debug, PartialEq)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Canvas {
    /// Creates a new `Canvas` instance with a specified width and height.
    ///
    /// Initializes the canvas with a given width and height, and fills the pixel buffer
    /// with black pixels (color value of 0.0 for red, green, and blue).
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the canvas in pixels.
    /// * `height` - The height of the canvas in pixels.
    ///
    /// # Returns
    ///
    /// Returns a new `Canvas` instance.
    pub fn new(width: usize, height: usize) -> Canvas {
        let pixels = vec![Color::new(0.0, 0.0, 0.0); width * height];
        Canvas { width, height, pixels }
    }

    /// Writes a pixel with a specified color at the given coordinates.
    ///
    /// This method modifies the color of a single pixel in the canvas's pixel buffer.
    /// The coordinates are zero-indexed, where (0, 0) is the top-left corner.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `color` - The `Color` to set the pixel to.
    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = y * self.width + x;
        self.pixels[index] = color;
    }

    /// Retrieves the color of a pixel at the given coordinates.
    ///
    /// This method returns the color of the pixel located at the specified coordinates.
    /// The coordinates are zero-indexed, where (0, 0) is the top-left corner.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    ///
    /// # Returns
    ///
    /// Returns the `Color` of the specified pixel.
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


    /// Writes the canvas content to a PNG file with anti-aliasing.
    ///
    /// This method saves the current state of the canvas to a PNG file, applying anti-aliasing
    /// based on the `aa` (anti-aliasing factor) provided. The anti-aliasing process averages
    /// the colors of `aa` x `aa` blocks of pixels to smooth out the transitions between colors.
    /// The resulting image is saved to the specified filename.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path and name of the file where the canvas should be saved.
    /// * `aa` - The anti-aliasing factor, specifying the size of the pixel blocks to average
    ///   for anti-aliasing. A higher value results in more smoothing but can lead to loss of detail.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be created, or if there is an error writing the PNG data to the file.
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
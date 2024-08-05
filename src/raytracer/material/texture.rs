
use image::ImageReader;
use image::RgbaImage;
use crate::color::Color;

/// Represents a texture for use in texturing 3D objects.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub image: RgbaImage
}

impl Texture {
    /// Creates a new `Texture` instance from an image file.
    pub fn new(path: &str) -> Texture {
        let image = ImageReader::open(path).unwrap().decode().unwrap().to_rgba8();
        let (width, height) = image.dimensions();
        Texture { width, height, image }
    }

    /// Returns the color of the texture at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `u` - The u-coordinate of the texture.
    /// * `v` - The v-coordinate of the texture.
    ///
    /// # Returns
    ///
    /// Returns the color of the texture at the specified coordinates.
    pub fn get_color(&self, u: f64, v: f64) -> [u8; 4] {
        // Clamp input texture coordinates to [0, 1]
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        // Convert u,v to pixel coordinates
        let x = ((u * self.width as f64) as u32).min(self.width - 1);
        let y = ((v * self.height as f64) as u32).min(self.height - 1);

        // Flip y for correct orientation, since v=0 is at the bottom
        let y = self.height - y - 1;

        // Get pixel color
        let pixel = self.image.get_pixel(x, y);
        [pixel[0], pixel[1], pixel[2], pixel[3]]
    }

    pub fn sample_texture(&self, u: f64, v: f64) -> Color {
        let color = self.get_color(u, v);
        Color::new(color[0] as f64 / 255.0,
                     color[1] as f64 / 255.0,
                     color[2] as f64 / 255.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture() {
        let texture = Texture::new("examples/test_texture.png");
        let color = texture.get_color(0.0, 0.0);
        assert_eq!(color, [0, 0, 0, 255]);
        let color = texture.get_color(1.0, 1.0);
        assert_eq!(color, [0, 0, 0, 255]);
        let color = texture.get_color(0.0, 1.0);
        assert_eq!(color, [19, 73, 151, 255]);
        let color = texture.get_color(1.0, 0.0);
        assert_eq!(color, [19, 73, 151, 255]);
        let color = texture.get_color(0.5, 0.5);
        assert_eq!(color, [19, 73, 151, 255]);
        let color = texture.get_color(0.2, 0.8 - crate::EPSILON);
        assert_eq!(color, [19, 73, 151, 255]);
        let color = texture.get_color(0.4, 0.6 - crate::EPSILON);
        assert_eq!(color, [19, 73, 151, 255]);
        let color = texture.get_color(0.6, 0.4 - crate::EPSILON);
        assert_eq!(color, [19, 73, 151, 255]);
        let color = texture.get_color(0.8, 0.2 - crate::EPSILON);
        assert_eq!(color, [19, 73, 151, 255]);
    }
}
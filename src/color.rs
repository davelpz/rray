#![allow(dead_code)]

use std::ops::Mul;

const EPSILON: f64 = 0.00001;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        (self.r - other.r).abs() < EPSILON
            && (self.g - other.g).abs() < EPSILON
            && (self.b - other.b).abs() < EPSILON
    }
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Color {
        Color { r, g, b }
    }

    pub fn add(&self, other: &Color) -> Color {
        Color::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }

    pub fn subtract(&self, other: &Color) -> Color {
        Color::new(self.r - other.r, self.g - other.g, self.b - other.b)
    }

    pub fn multiply(&self, scalar: f64) -> Color {
        Color::new(self.r * scalar, self.g * scalar, self.b * scalar)
    }

    pub fn product(&self, other: &Color) -> Color {
        Color::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}


#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn test_color() {
        let c1 = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(c1.r, -0.5);
        assert_eq!(c1.g, 0.4);
        assert_eq!(c1.b, 1.7);
        let c2 = Color::new(0.9, 1.0, 0.2);
        assert_eq!(c1.add(&c2), Color::new(0.4, 1.4, 1.9));
        assert_eq!(c1.subtract(&c2), Color::new(-1.4, -0.6, 1.5));
        assert_eq!(c1.multiply(2.0), Color::new(-1.0, 0.8, 3.4));
        assert_eq!(c1.product(&c2), Color::new(-0.45, 0.4, 0.34));
    }

    #[test]
    fn test_color_add() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(c1.add(&c2), Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn test_color_subtract() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        assert_eq!(c1.subtract(&c2), Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn test_color_multiply() {
        let c1 = Color::new(0.2, 0.3, 0.4);
        assert_eq!(c1.multiply(2.0), Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn test_color_product() {
        let c1 = Color::new(1.0, 0.2, 0.4);
        let c2 = Color::new(0.9, 1.0, 0.1);
        assert_eq!(c1.product(&c2), Color::new(0.9, 0.2, 0.04));
    }
}
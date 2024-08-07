#![allow(dead_code)]

// This module defines the `Tuple` struct and its associated operations. Tuples are used to represent points and vectors in 3D space.

use std::ops::{Sub, Add, Mul, Div};
use crate::EPSILON;

/// Represents a tuple in 3D space, which can be a point or a vector based on `w`.
/// A `w` of 1.0 indicates a point, and 0.0 indicates a vector.
#[derive(Debug, Clone, Copy)]
pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64, // Determines whether the tuple is a point (w=1.0) or a vector (w=0.0)
}

impl PartialEq for Tuple {
    /// Checks for equality between two `Tuple` instances, considering the floating-point precision defined by `EPSILON`.
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < EPSILON
            && (self.y - other.y).abs() < EPSILON
            && (self.z - other.z).abs() < EPSILON
            && (self.w - other.w).abs() < EPSILON
    }
}

impl Tuple {
    /// Creates a new `Tuple`.
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Tuple {
        Tuple { x, y, z, w }
    }

    /// Determines if the tuple is a point.
    pub fn is_point(&self) -> bool {
        self.w == 1.0
    }

    /// Determines if the tuple is a vector.
    pub fn is_vector(&self) -> bool {
        self.w == 0.0
    }

    /// Creates a new point `Tuple`.
    pub fn point(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 1.0)
    }

    /// Creates a new vector `Tuple`.
    pub fn vector(x: f64, y: f64, z: f64) -> Tuple {
        Tuple::new(x, y, z, 0.0)
    }

    /// Adds two tuples together, returning a new `Tuple`.
    pub fn add(&self, other: &Tuple) -> Tuple {
        Tuple::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
    }

    /// Adds a scalar to each component of the tuple, returning a new `Tuple`.
    pub fn add_float(&self, other: f64) -> Tuple {
        Tuple::new(self.x + other, self.y + other, self.z + other, self.w + other)
    }

    /// Subtracts one tuple from another, returning a new `Tuple`.
    pub fn subtract(&self, other: &Tuple) -> Tuple {
        Tuple::new(self.x - other.x, self.y - other.y, self.z - other.z, self.w - other.w)
    }

    /// Negates the tuple, returning a new `Tuple`.
    pub fn negate(&self) -> Tuple {
        Tuple::new(-self.x, -self.y, -self.z, -self.w)
    }

    /// Multiplies each component of the tuple by a scalar, returning a new `Tuple`.
    pub fn multiply(&self, scalar: f64) -> Tuple {
        Tuple::new(self.x * scalar, self.y * scalar, self.z * scalar, self.w * scalar)
    }

    /// Divides each component of the tuple by a scalar, returning a new `Tuple`.
    pub fn divide(&self, scalar: f64) -> Tuple {
        Tuple::new(self.x / scalar, self.y / scalar, self.z / scalar, self.w / scalar)
    }

    /// Calculates the magnitude of the vector.
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    /// Calculates the squared length of a point
    pub fn length_squared(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    /// Normalizes the vector, returning a unit vector in the same direction.
    pub fn normalize(&self) -> Tuple {
        let magnitude = self.magnitude();
        Tuple::new(self.x / magnitude, self.y / magnitude, self.z / magnitude, self.w / magnitude)
    }

    /// Calculates the dot product of two vectors.
    pub fn dot(&self, other: &Tuple) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    /// Calculates the cross product of two vectors.
    pub fn cross(&self, other: &Tuple) -> Tuple {
        Tuple::vector(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        )
    }

    /// Reflects a vector off a surface, given the surface's normal vector.
    pub fn reflect(&self, normal: &Tuple) -> Tuple {
        self.subtract(&normal.multiply(2.0 * self.dot(normal)))
    }
}

/// Operator overloads for `Tuple`.
impl Sub for Tuple {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.subtract(&other)
    }
}

impl Add for Tuple {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Tuple::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
    }
}

impl Add<f64> for Tuple {
    type Output = Self;

    fn add(self, scalar: f64) -> Self {
        Tuple::new(self.x + scalar, self.y + scalar, self.z + scalar, self.w + scalar)
    }
}

impl Mul<f64> for Tuple {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        self.multiply(scalar)
    }
}

impl Div<f64> for Tuple {
    type Output = Self;

    fn div(self, scalar: f64) -> Self {
        self.divide(scalar)
    }
}

//unit tests
#[cfg(test)]
mod tests {
    use super::Tuple;
    use super::EPSILON;

    #[test]
    fn test_tuple_is_point() {
        let t = Tuple::new(4.3, -4.2, 3.1, 1.0);
        assert_eq!(t.x, 4.3);
        assert_eq!(t.y, -4.2);
        assert_eq!(t.z, 3.1);
        assert_eq!(t.w, 1.0);
        assert_eq!(t.is_point(), true);
    }

    #[test]
    fn test_tuple_is_vector() {
        let t = Tuple::new(4.3, -4.2, 3.1, 0.0);
        assert_eq!(t.is_vector(), true);
    }

    #[test]
    fn test_tuple_point() {
        let t = Tuple::point(4.0, -4.0, 3.0);
        assert_eq!(t.is_point(), true);
    }

    #[test]
    fn test_tuple_vector() {
        let t = Tuple::vector(4.0, -4.0, 3.0);
        assert_eq!(t.is_vector(), true);
    }

    #[test]
    fn test_tuple_add() {
        let a1 = Tuple::new(3.0, -2.0, 5.0, 1.0);
        let a2 = Tuple::new(-2.0, 3.0, 1.0, 0.0);
        let result = a1.add(&a2);
        assert_eq!(result.x, 1.0);
        assert_eq!(result.y, 1.0);
        assert_eq!(result.z, 6.0);
        assert_eq!(result.w, 1.0);
    }

    #[test]
    fn test_tuple_subtract() {
        let a1 = Tuple::point(3.0, 2.0, 1.0);
        let a2 = Tuple::point(5.0, 6.0, 7.0);
        let result = a1.subtract(&a2);
        assert_eq!(result.x, -2.0);
        assert_eq!(result.y, -4.0);
        assert_eq!(result.z, -6.0);
        assert_eq!(result.w, 0.0);
        assert_eq!(result.is_vector(), true);

        let a1 = Tuple::point(3.0, 2.0, 1.0);
        let a2 = Tuple::vector(5.0, 6.0, 7.0);
        let result = a1.subtract(&a2);
        assert_eq!(result.x, -2.0);
        assert_eq!(result.y, -4.0);
        assert_eq!(result.z, -6.0);
        assert_eq!(result.w, 1.0);
        assert_eq!(result.is_point(), true);

        let a1 = Tuple::vector(3.0, 2.0, 1.0);
        let a2 = Tuple::vector(5.0, 6.0, 7.0);
        let result = a1.subtract(&a2);
        assert_eq!(result.x, -2.0);
        assert_eq!(result.y, -4.0);
        assert_eq!(result.z, -6.0);
        assert_eq!(result.w, 0.0);
        assert_eq!(result.is_vector(), true);
    }

    #[test]
    fn test_tuple_negate() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = a.negate();
        assert_eq!(result.x, -1.0);
        assert_eq!(result.y, 2.0);
        assert_eq!(result.z, -3.0);
        assert_eq!(result.w, 4.0);
    }

    #[test]
    fn test_tuple_multiply() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = a.multiply(3.5);
        assert_eq!(result.x, 3.5);
        assert_eq!(result.y, -7.0);
        assert_eq!(result.z, 10.5);
        assert_eq!(result.w, -14.0);

        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = a.multiply(0.5);
        assert_eq!(result.x, 0.5);
        assert_eq!(result.y, -1.0);
        assert_eq!(result.z, 1.5);
        assert_eq!(result.w, -2.0);
    }

    #[test]
    fn test_tuple_divide() {
        let a = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let result = a.divide(2.0);
        assert_eq!(result.x, 0.5);
        assert_eq!(result.y, -1.0);
        assert_eq!(result.z, 1.5);
        assert_eq!(result.w, -2.0);
    }

    #[test]
    fn test_tuple_magnitude() {
        let a = Tuple::vector(1.0, 0.0, 0.0);
        assert_eq!(a.magnitude(), 1.0);

        let a = Tuple::vector(0.0, 1.0, 0.0);
        assert_eq!(a.magnitude(), 1.0);

        let a = Tuple::vector(0.0, 0.0, 1.0);
        assert_eq!(a.magnitude(), 1.0);

        let a = Tuple::vector(1.0, 2.0, 3.0);
        assert_eq!(a.magnitude(), 14.0_f64.sqrt());

        let a = Tuple::vector(-1.0, -2.0, -3.0);
        assert_eq!(a.magnitude(), 14.0_f64.sqrt());
    }

    #[test]
    fn test_tuple_normalize() {
        let a = Tuple::vector(4.0, 0.0, 0.0);
        let result = a.normalize();
        assert_eq!(result.x, 1.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
        assert_eq!(result.w, 0.0);

        let a = Tuple::vector(1.0, 2.0, 3.0);
        let result = a.normalize();
        assert_eq!(result.x, 1.0 / 14.0_f64.sqrt());
        assert_eq!(result.y, 2.0 / 14.0_f64.sqrt());
        assert_eq!(result.z, 3.0 / 14.0_f64.sqrt());
        assert_eq!(result.w, 0.0);

        let a = Tuple::vector(1.0, 2.0, 3.0);
        let result = a.normalize();
        assert_eq!(result.magnitude(), 1.0);
    }

    #[test]
    fn test_tuple_dot() {
        let a = Tuple::vector(1.0, 2.0, 3.0);
        let b = Tuple::vector(2.0, 3.0, 4.0);
        assert_eq!(a.dot(&b), 20.0);
    }

    #[test]
    fn test_tuple_cross() {
        let a = Tuple::vector(1.0, 2.0, 3.0);
        let b = Tuple::vector(2.0, 3.0, 4.0);
        let result = a.cross(&b);
        assert_eq!(result.x, -1.0);
        assert_eq!(result.y, 2.0);
        assert_eq!(result.z, -1.0);

        let result = b.cross(&a);
        assert_eq!(result.x, 1.0);
        assert_eq!(result.y, -2.0);
        assert_eq!(result.z, 1.0);
    }

    #[test]
    fn test_tuple_reflect() {
        let v = Tuple::vector(1.0, -1.0, 0.0);
        let n = Tuple::vector(0.0, 1.0, 0.0);
        let r = v.reflect(&n);
        assert_eq!(true, (r.x - 1.0).abs() < EPSILON);
        assert_eq!(true, (r.y - 1.0).abs() < EPSILON);
        assert_eq!(true, (r.z - 0.0).abs() < EPSILON);

        let v = Tuple::vector(0.0, -1.0, 0.0);
        let n = Tuple::vector(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let r = v.reflect(&n);
        assert_eq!(true, (r.x - 1.0).abs() < EPSILON);
        assert_eq!(true, (r.y - 0.0).abs() < EPSILON);
        assert_eq!(true, (r.z - 0.0).abs() < EPSILON);
    }
}
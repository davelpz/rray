use crate::color::Color;
use crate::matrix::Matrix;
use crate::raytracer::material::noise;
use crate::raytracer::material::texture::Texture;
use crate::tuple::Tuple;

/// Represents the type of pattern to be applied to a surface in a ray tracing context.
///
/// This enum defines various types of patterns that can be used to texture objects within a ray tracing scene.
/// Patterns can range from simple solid colors to more complex types like stripes, gradients, and procedural noise.
/// Patterns can be combined or modified for more complex effects, such as blending two patterns or applying a perturbation.
///
/// # Variants
///
/// * `Test` - A test pattern, typically used for debugging.
/// * `Solid(Color)` - A solid color pattern.
/// * `Stripe(Box<Pattern>, Box<Pattern>)` - A stripe pattern alternating between two other patterns.
/// * `Gradient(Box<Pattern>, Box<Pattern>)` - A gradient pattern smoothly transitioning between two patterns.
/// * `Ring(Box<Pattern>, Box<Pattern>)` - A ring pattern alternating between two patterns in a radial fashion.
/// * `Checker(Box<Pattern>, Box<Pattern>)` - A checkerboard pattern alternating between two patterns.
/// * `Blend(Box<Pattern>, Box<Pattern>, f64)` - A blend of two patterns, with the blend ratio specified by a floating point value.
/// * `Perturbed(Box<Pattern>, f64, usize, f64)` - A pattern perturbed by noise, with parameters for scale, octaves, and persistence.
/// * `Noise(Box<Pattern>, Box<Pattern>, f64, usize, f64)` - A noise-based pattern, with parameters for scale, octaves, and persistence.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PatternType {
    Test,
    Solid(Color),
    Stripe(Box<Pattern>, Box<Pattern>),
    Gradient(Box<Pattern>, Box<Pattern>),
    Ring(Box<Pattern>, Box<Pattern>),
    Checker(Box<Pattern>, Box<Pattern>),
    Blend(Box<Pattern>, Box<Pattern>, f64),
    Perturbed(Box<Pattern>, f64, usize, f64),
    Noise(Box<Pattern>, Box<Pattern>, f64, usize, f64),
    Texture(Texture)
}

/// Represents a pattern with a specific type and transformation.
///
/// This struct encapsulates a pattern type, defined by the `PatternType` enum, and a transformation
/// matrix. The pattern type determines the visual appearance of the pattern, such as solid color,
/// stripes, gradients, etc. The transformation matrix allows the pattern to be scaled, rotated,
/// or translated, enabling more complex and varied pattern applications on surfaces.
///
/// # Fields
///
/// * `pattern_type` - The specific type of the pattern, determining its visual appearance.
/// * `transform` - A transformation matrix applied to the pattern for scaling, rotation, or translation.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub transform: Matrix,
}

impl Pattern {
    #[allow(dead_code)]
    pub fn test() -> Pattern {
        Pattern {
            pattern_type: PatternType::Test,
            transform: Matrix::identity(4),
        }
    }

    pub fn solid(color: Color, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Solid(color),
            transform,
        }
    }

    pub fn stripe(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Stripe(Box::new(a), Box::new(b)),
            transform,
        }
    }

    pub fn gradient(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Gradient(Box::new(a), Box::new(b)),
            transform,
        }
    }

    pub fn ring(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Ring(Box::new(a), Box::new(b)),
            transform,
        }
    }

    pub fn checker(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Checker(Box::new(a), Box::new(b)),
            transform,
        }
    }

    pub fn blend(a: Pattern, b: Pattern, scale: f64, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Blend(Box::new(a), Box::new(b), scale),
            transform,
        }
    }

    pub fn perturbed(a: Pattern, scale: f64, octaves: usize, persistence: f64, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Perturbed(Box::new(a), scale, octaves, persistence),
            transform,
        }
    }

    pub fn noise(a: Pattern, b: Pattern, scale: f64, octaves: usize, persistence: f64, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Noise(Box::new(a), Box::new(b), scale, octaves, persistence),
            transform,
        }
    }

    pub fn texture(file_name: &str, transform: Matrix) -> Pattern {
        Pattern {
            pattern_type: PatternType::Texture(Texture::new(file_name)),
            transform,
        }
    }

    /// Calculates the color of the pattern at a given point on an object.
    ///
    /// This method computes the color of the pattern at a specific point on an object, taking into
    /// account the pattern's type and its transformation. The method applies the pattern's transformation
    /// to the given object point to determine the pattern's local point, and then calculates the color
    /// based on the pattern's type (e.g., solid, stripe, gradient, etc.). For patterns involving noise
    /// or perturbation, additional calculations are performed to adjust the point's coordinates before
    /// determining the color.
    ///
    /// # Arguments
    ///
    /// * `object_point` - A reference to a `Tuple` representing the point on the object in world space
    ///   where the color is to be calculated.
    ///
    /// # Returns
    ///
    /// Returns a `Color` representing the color of the pattern at the given point on the object.
    pub fn pattern_at(&self, object_point: &Tuple, shape: usize) -> Color {
        let pattern_point = self.transform.inverse().multiply_tuple(object_point);
        match &self.pattern_type {
            PatternType::Test => {
                Color::new(pattern_point.x, pattern_point.y, pattern_point.z)
            },
            PatternType::Solid(color) => {
                color.clone()
            },
            PatternType::Stripe(a, b) => {
                if (pattern_point.x.floor() as i32) % 2 == 0 {
                    a.pattern_at(&pattern_point, shape)
                } else {
                    b.pattern_at(&pattern_point, shape)
                }
            },
            PatternType::Gradient(a, b) => {
                let a = a.pattern_at(&pattern_point, shape);
                let b = b.pattern_at(&pattern_point, shape);
                let distance = b.subtract(&a);
                let fraction = pattern_point.x - pattern_point.x.floor();
                a.add(&distance.multiply(fraction))
            },
            PatternType::Ring(a, b) => {
                if (pattern_point.x.powi(2) + pattern_point.z.powi(2)).sqrt().floor() as i32 % 2 == 0 {
                    a.pattern_at(&pattern_point, shape)
                } else {
                    b.pattern_at(&pattern_point, shape)
                }
            },
            PatternType::Checker(a, b) => {
                if (pattern_point.x.floor() + pattern_point.y.floor() + pattern_point.z.floor()) as i32 % 2 == 0 {
                    a.pattern_at(&pattern_point, shape)
                } else {
                    b.pattern_at(&pattern_point, shape)
                }
            },
            PatternType::Blend(a, b, scale) => {
                let a = a.pattern_at(&pattern_point, shape);
                let b = b.pattern_at(&pattern_point, shape);
                a.multiply(1.0-scale).add(&b.multiply(*scale))
            },
            PatternType::Perturbed(a, scale, octaves, persistence) => {
                let x = pattern_point.x;
                let y = pattern_point.y;
                let z = pattern_point.z;
                let noise_x = noise::octave_perlin(x, y, z, *octaves, *persistence) * scale;
                let noise_y = noise::octave_perlin(x, y, z + 1.0, *octaves, *persistence) * scale;
                let noise_z = noise::octave_perlin(x, y, z + 2.0, *octaves, *persistence) * scale;
                let new_x = pattern_point.x + noise_x;
                let new_y = pattern_point.y + noise_y;
                let new_z = pattern_point.z + noise_z;
                let new_point = Tuple::new(new_x, new_y, new_z, pattern_point.w);
                a.pattern_at(&new_point, shape)
            },
            PatternType::Noise(a, b, scale, octaves, persistence) => {
                let noise = noise::octave_perlin(pattern_point.x, pattern_point.y, pattern_point.z, *octaves, *persistence);
                let noise = noise * scale;
                if noise <= 0.0 {
                    a.pattern_at(&pattern_point, shape).multiply(-noise)
                } else {
                    b.pattern_at(&pattern_point, shape).multiply(noise)
                }
            },
            PatternType::Texture(texture) => {
                let object = crate::raytracer::object::db::get_object(shape);
                let (u,v) = object.uv_mapping(&pattern_point);
                texture.sample_texture(u, v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::raytracer::intersection::Intersection;
    use crate::raytracer::light::Light;
    use crate::tuple::Tuple;
    use crate::raytracer::material::noise::get_noise_3d;
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::ray::Ray;
    use crate::raytracer::scene::Scene;

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let p = Pattern::stripe(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 1.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 2.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let p = Pattern::stripe(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 1.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 2.0), 0), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let p = Pattern::stripe(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.9, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(1.0, 0.0, 0.0), 0), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(-0.1, 0.0, 0.0), 0), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(-1.0, 0.0, 0.0), 0), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(-1.1, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn gradient_pattern_linearly_interpolates_between_colors() {
        let p = Pattern::gradient(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                  Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                  Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.25, 0.0, 0.0), 0), Color::new(0.75, 0.75, 0.75));
        assert_eq!(p.pattern_at(&Tuple::point(0.5, 0.0, 0.0), 0), Color::new(0.5, 0.5, 0.5));
        assert_eq!(p.pattern_at(&Tuple::point(0.75, 0.0, 0.0), 0), Color::new(0.25, 0.25, 0.25));
    }

    #[test]
    fn ring_should_extend_in_both_x_and_z() {
        let p = Pattern::ring(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                              Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                              Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(1.0, 0.0, 0.0), 0), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 1.0), 0), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.708, 0.0, 0.708), 0), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn checkers_repeat_in_x() {
        let p = Pattern::checker(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                 Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                 Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.99, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(1.01, 0.0, 0.0), 0), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn checkers_repeat_in_y() {
        let p = Pattern::checker(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                 Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                 Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.99, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 1.01, 0.0), 0), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn checkers_repeat_in_z() {
        let p = Pattern::checker(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                 Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                 Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.99), 0), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 1.01), 0), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    #[ignore]
    fn test_fastnoise() {
        // Create and configure the FastNoise object
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for _i in 0..1000000 {
            let random_x = (1000.0) * rand::random::<f64>();
            let random_y = (1000.0) * rand::random::<f64>();
            let random_z = (1000.0) * rand::random::<f64>();
            let value = get_noise_3d(random_x, random_y, random_z);
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
        }
        println!("min: {}, max {}", min, max);
    }

    #[test]
    fn schlick_reflectance_under_total_internal_reflection() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::white()));
        let shape = Sphere::glass_sphere();
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let r = Ray::new(Tuple::point(0.0, 0.0, 2_f64.sqrt()/2.0), Tuple::vector(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection::new(-2.0_f64.sqrt() / 2.0, id, 0.0, 0.0),
            Intersection::new(2.0_f64.sqrt() / 2.0, id, 0.0, 0.0)
        ];
        let comps = xs[1].prepare_computations(&r, &xs);
        let reflectance = comps.schlick();
        assert_eq!(reflectance, 1.0);
    }

    #[test]
    fn schlick_reflectance_with_perpendicular_viewing_angle() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::white()));
        let shape = Sphere::glass_sphere();
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection::new(-1.0, id, 0.0, 0.0),
            Intersection::new(1.0, id, 0.0, 0.0)
        ];
        let comps = xs[1].prepare_computations(&r, &xs);
        let reflectance = comps.schlick();
        assert!((reflectance - 0.04).abs() < crate::EPSILON);
    }

    #[test]
    fn schlick_approximation_with_small_angle_and_n2_greater_than_n1() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::white()));
        let shape = Sphere::glass_sphere();
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let r = Ray::new(Tuple::point(0.0, 0.99, -2.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![
            Intersection::new(1.8589, id, 0.0, 0.0)
        ];
        let comps = xs[0].prepare_computations(&r, &xs);
        let reflectance = comps.schlick();
        assert!((reflectance - 0.48873).abs() < crate::EPSILON);
    }
}
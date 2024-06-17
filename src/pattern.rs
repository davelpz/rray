pub mod pattern {
    use crate::pattern::noise::NOISE_GENERATOR;
    use crate::color::color::Color;
    use crate::matrix::matrix::Matrix;
    use crate::tuple::tuple::Tuple;

    #[derive(Debug, Clone, PartialEq)]
    pub enum PatternType {
        Solid(Color),
        Stripe(Box<Pattern>, Box<Pattern>),
        Gradient(Box<Pattern>, Box<Pattern>),
        Ring(Box<Pattern>, Box<Pattern>),
        Checker(Box<Pattern>, Box<Pattern>),
        Blend(Box<Pattern>, Box<Pattern>),
        Perturbed(Box<Pattern>, f64),
        Noise(Box<Pattern>,Box<Pattern>, f64)
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Pattern {
        pub pattern_type: PatternType,
        pub transform: Matrix,
    }

    impl Pattern {
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

        pub fn blend(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Blend(Box::new(a), Box::new(b)),
                transform,
            }
        }

        pub fn perturbed(a: Pattern, scale: f64, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Perturbed(Box::new(a), scale),
                transform,
            }
        }

        pub fn noise(a: Pattern, b: Pattern, scale: f64, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Noise(Box::new(a), Box::new(b), scale),
                transform,
            }
        }

        pub fn pattern_at(&self, object_point: &Tuple) -> Color {
            let pattern_point = self.transform.inverse().multiply_tuple(object_point);
            match &self.pattern_type {
                PatternType::Solid(color) => {
                    color.clone()
                },
                PatternType::Stripe(a,b) => {
                    if (pattern_point.x.floor() as i32) % 2 == 0 {
                        a.pattern_at(&pattern_point)
                    } else {
                        b.pattern_at(&pattern_point)
                    }
                },
                PatternType::Gradient(a,b) => {
                    let a = a.pattern_at(&pattern_point);
                    let b = b.pattern_at(&pattern_point);
                    let distance = b.subtract(&a);
                    let fraction = pattern_point.x - pattern_point.x.floor();
                    a.add(&distance.multiply(fraction))
                },
                PatternType::Ring(a,b) => {
                    if (pattern_point.x.powi(2) + pattern_point.z.powi(2)).sqrt().floor() as i32 % 2 == 0 {
                        a.pattern_at(&pattern_point)
                    } else {
                        b.pattern_at(&pattern_point)
                    }
                },
                PatternType::Checker(a,b) => {
                    if (pattern_point.x.floor() + pattern_point.y.floor() + pattern_point.z.floor()) as i32 % 2 == 0 {
                        a.pattern_at(&pattern_point)
                    } else {
                        b.pattern_at(&pattern_point)
                    }
                },
                PatternType::Blend(a,b) => {
                    let a = a.pattern_at(&pattern_point);
                    let b = b.pattern_at(&pattern_point);
                    a.add(&b).multiply(0.5)
                },
                PatternType::Perturbed(a, scale) => {
                    let pscale = 1.0;
                    let x = pattern_point.x * pscale;
                    let y = pattern_point.y * pscale;
                    let z = pattern_point.z * pscale;
                    let noise_x = (NOISE_GENERATOR.get_noise_3d(x, y, z) as f64) * scale;
                    let noise_y = (NOISE_GENERATOR.get_noise_3d(x, y, z + 1.0) as f64) * scale;
                    let noise_z = (NOISE_GENERATOR.get_noise_3d(x, y, z + 2.0) as f64) * scale;
                    println!("pattern_point: {:.10},{:.10},{:.10}", pattern_point.x,pattern_point.y,pattern_point.z);
                    println!("noise_x: {:.10}, noise_y: {:.10}, noise_z: {:.10}", noise_x, noise_y, noise_z);
                    let new_x = pattern_point.x + noise_x;
                    let new_y = pattern_point.y + noise_y;
                    let new_z = pattern_point.z + noise_z;
                    let new_point = Tuple::new(new_x, new_y, new_z, pattern_point.w);
                    let o = (pattern_point.x.floor() + pattern_point.y.floor() + pattern_point.z.floor()) as i32 % 2;
                    let n = (new_point.x.floor() + new_point.y.floor() + new_point.z.floor()) as i32 % 2;
                    println!("new_point: {:.10},{:.10},{:.10}\to: {}\tn: {}", new_point.x,new_point.y,new_point.z,o,n);
                    a.pattern_at(&new_point)
                },
                PatternType::Noise(a,b, scale) => {
                    let noise = NOISE_GENERATOR.get_noise_3d(pattern_point.x, pattern_point.y, pattern_point.z);
                    let noise = noise as f64 * scale;
                    if noise <= 0.0 {
                        let noise = -noise;
                        a.pattern_at(&pattern_point).multiply(noise)
                    } else {
                        b.pattern_at(&pattern_point).multiply(noise)
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    fn turb(p: Tuple, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;
        for _i in 0..depth {
            let n = NOISE_GENERATOR.get_noise_3d(temp_p.x, temp_p.y, temp_p.z);
            accum += weight * n;
            weight *= 0.5;
            temp_p = temp_p.multiply(2.0);
        }

        accum.abs() as f64
    }
}

pub mod noise {
    use fastnoise_lite::{FastNoiseLite, NoiseType};
    use lazy_static::lazy_static;

    fn init_noise() -> FastNoiseLite {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::Perlin));
        noise
    }

    lazy_static! {
        pub static ref NOISE_GENERATOR: FastNoiseLite = init_noise();
    }
}


#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::matrix::matrix::Matrix;
    use crate::tuple::tuple::Tuple;
    use crate::pattern::pattern::{Pattern};

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let p = Pattern::stripe(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 1.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 2.0, 0.0)), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let p = Pattern::stripe(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 1.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 2.0)), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let p = Pattern::stripe(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.9, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(1.0, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(-0.1, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(-1.0, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(-1.1, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn gradient_pattern_linearly_interpolates_between_colors() {
        let p = Pattern::gradient(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                  Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                  Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.25, 0.0, 0.0)), Color::new(0.75, 0.75, 0.75));
        assert_eq!(p.pattern_at(&Tuple::point(0.5, 0.0, 0.0)), Color::new(0.5, 0.5, 0.5));
        assert_eq!(p.pattern_at(&Tuple::point(0.75, 0.0, 0.0)), Color::new(0.25, 0.25, 0.25));
    }

    #[test]
    fn ring_should_extend_in_both_x_and_z() {
        let p = Pattern::ring(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                              Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                              Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(1.0, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 1.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.708, 0.0, 0.708)), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn checkers_repeat_in_x() {
        let p = Pattern::checker(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                 Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                 Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.99, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(1.01, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn checkers_repeat_in_y() {
        let p = Pattern::checker(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                 Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                 Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.99, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 1.01, 0.0)), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn checkers_repeat_in_z() {
        let p = Pattern::checker(Pattern::solid(Color::new(1.0,1.0,1.0), Matrix::identity(4)),
                                 Pattern::solid(Color::new(0.0,0.0,0.0), Matrix::identity(4)),
                                 Matrix::identity(4));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 0.99)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(p.pattern_at(&Tuple::point(0.0, 0.0, 1.01)), Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_fastnoise() {
        // Create and configure the FastNoise object
        let mut noise = &crate::pattern::noise::NOISE_GENERATOR;
        for _i in 0..100 {
            let random_x = 1.0 + (100.0 - 1.0) * rand::random::<f64>();
            let random_y = 1.0 + (100.0 - 1.0) * rand::random::<f64>();
            let random_z = 1.0 + (100.0 - 1.0) * rand::random::<f64>();
            let value = noise.get_noise_3d(random_x, random_y, random_z);
            println!("value: {}", value);
        }
    }
}
pub mod pattern {
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
                    // let mut perturbed_point = pattern_point.clone();
                    // let noise = perlin_noise(&pattern_point) * scale;
                    // perturbed_point = perturbed_point + noise;
                    // // perturbed_point.x += perlin_noise(&pattern_point) * scale;
                    // // perturbed_point.y += perlin_noise(&pattern_point) * scale;
                    // // perturbed_point.z += perlin_noise(&pattern_point) * scale;

                    let jitter = turb(pattern_point, 7) * scale;
                    a.pattern_at(&pattern_point.add_float( jitter))
                }
            }
        }
    }

    fn turb(p: Tuple, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;
        for _i in 0..depth {
            accum += weight * perlin_noise(&temp_p);
            weight *= 0.5;
            temp_p = temp_p.multiply(2.0);
        }

        accum.abs()
    }

    pub fn perlin_noise(point: &Tuple) -> f64 {
    let p = [
        151,160,137,91,90,15,
        131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
        190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
        88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
        77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
        102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
        135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
        5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
        223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
        129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
        251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
        49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
        138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180
    ];
    let repeat = p.len() / 2;
    let xi = (point.x as usize) % repeat;
    let yi = (point.y as usize) % repeat;
    let zi = (point.z as usize) % repeat;
    let xf = point.x.fract();
    let yf = point.y.fract();
    let zf = point.z.fract();
    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);
    let aaa = p[p[p[    xi ]+    yi ]+    zi ];
    let aba = p[p[p[    xi ]+inc(yi)]+    zi ];
    let aab = p[p[p[    xi ]+    yi ]+inc(zi)];
    let abb = p[p[p[    xi ]+inc(yi)]+inc(zi)];
    let baa = p[p[p[inc(xi)]+    yi ]+    zi ];
    let bba = p[p[p[inc(xi)]+inc(yi)]+    zi ];
    let bab = p[p[p[inc(xi)]+    yi ]+inc(zi)];
    let bbb = p[p[p[inc(xi)]+inc(yi)]+inc(zi)];

    lerp(w, lerp(v, lerp(u, grad(aaa, xf  , yf  , zf  ),
                               grad(baa, xf-1.0, yf  , zf  )),
                       lerp(u, grad(aba, xf  , yf-1.0, zf  ),
                               grad(bba, xf-1.0, yf-1.0, zf  ))),
           lerp(v, lerp(u, grad(aab, xf  , yf  , zf-1.0),
                               grad(bab, xf-1.0, yf  , zf-1.0)),
                       lerp(u, grad(abb, xf  , yf-1.0, zf-1.0),
                               grad(bbb, xf-1.0, yf-1.0, zf-1.0))))
}

fn inc(num: usize) -> usize {
    num + 1
}

fn grad(hash: usize, x: f64, y: f64, z: f64) -> f64 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
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
}
pub mod pattern {
    use crate::color::color::Color;
    use crate::matrix::matrix::Matrix;
    use crate::tuple::tuple::Tuple;

    #[derive(Debug, Clone, PartialEq)]
    pub enum PatternType {
        Solid,
        Stripe,
        Gradient,
        Ring,
        Checker,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PatternDetails {
        Color(Color),
        Composite(Box<Pattern>, Box<Pattern>),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Pattern {
        pub pattern_type: PatternType,
        pub details: PatternDetails,
        pub transform: Matrix,
    }

    impl Pattern {
        pub fn solid(color: Color, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Solid,
                details: PatternDetails::Color(color),
                transform,
            }
        }

        pub fn stripe(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Stripe,
                details: PatternDetails::Composite(Box::new(a), Box::new(b)),
                transform,
            }
        }

        pub fn gradient(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Gradient,
                details: PatternDetails::Composite(Box::new(a), Box::new(b)),
                transform,
            }
        }

        pub fn ring(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Ring,
                details: PatternDetails::Composite(Box::new(a), Box::new(b)),
                transform,
            }
        }

        pub fn checker(a: Pattern, b: Pattern, transform: Matrix) -> Pattern {
            Pattern {
                pattern_type: PatternType::Checker,
                details: PatternDetails::Composite(Box::new(a), Box::new(b)),
                transform,
            }
        }

        pub fn pattern_at(&self, point: &Tuple) -> Color {
            match &self.pattern_type {
                PatternType::Solid => {
                    match &self.details {
                        PatternDetails::Color(color) => color.clone(),
                        PatternDetails::Composite(a, _) => a.pattern_at(point),
                    }
                },
                PatternType::Stripe => {
                    if (point.x.floor() as i32) % 2 == 0 {
                        match &self.details {
                            PatternDetails::Color(color) => color.clone(),
                            PatternDetails::Composite(a, _) => a.pattern_at(point),
                        }
                    } else {
                        match &self.details {
                            PatternDetails::Color(color) => color.clone(),
                            PatternDetails::Composite(_, b) => b.pattern_at(point),
                        }
                    }
                },
                PatternType::Gradient => {
                    let a = match &self.details {
                        PatternDetails::Color(color) => color.clone(),
                        PatternDetails::Composite(a, _) => a.pattern_at(point),
                    };
                    let b = match &self.details {
                        PatternDetails::Color(color) => color.clone(),
                        PatternDetails::Composite(_, b) => b.pattern_at(point),
                    };
                    let distance = b.subtract(&a);
                    let fraction = point.x - point.x.floor();
                    a.add(&distance.multiply(fraction))
                },
                PatternType::Ring => {
                    if (point.x.powi(2) + point.z.powi(2)).sqrt().floor() as i32 % 2 == 0 {
                        match &self.details {
                            PatternDetails::Color(color) => color.clone(),
                            PatternDetails::Composite(a, _) => a.pattern_at(point),
                        }
                    } else {
                        match &self.details {
                            PatternDetails::Color(color) => color.clone(),
                            PatternDetails::Composite(_, b) => b.pattern_at(point),
                        }
                    }
                },
                PatternType::Checker => {
                    if (point.x.floor() + point.y.floor() + point.z.floor()) as i32 % 2 == 0 {
                        match &self.details {
                            PatternDetails::Color(color) => color.clone(),
                            PatternDetails::Composite(a, _) => a.pattern_at(point),
                        }
                    } else {
                        match &self.details {
                            PatternDetails::Color(color) => color.clone(),
                            PatternDetails::Composite(_, b) => b.pattern_at(point),
                        }
                    }
                },
            }
        }
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
}
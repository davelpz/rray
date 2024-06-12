#[allow(dead_code)]

pub mod material {
    use crate::color::color::Color;
    use crate::matrix::matrix::Matrix;
    use crate::shape::shape::Shape;
    use crate::tuple::tuple::Tuple;

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum Pattern {
        Stripe(Color, Color),
        Solid(Color),
        Gradient(Color, Color),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Material {
        pub pattern: Pattern,
        pub transform: Matrix,
        pub ambient: f64,
        pub diffuse: f64,
        pub specular: f64,
        pub shininess: f64,
    }

    impl Material {
        pub fn new(pattern: Pattern, transform: Matrix, ambient: f64, diffuse: f64, specular: f64, shininess: f64) -> Material {
            Material { pattern, transform, ambient, diffuse, specular, shininess }
        }

        pub fn default() -> Material {
            Material {
                pattern: Pattern::Solid(Color::new(1.0, 1.0, 1.0)),
                transform: Matrix::identity(4),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 200.0,
            }
        }

        pub fn pattern_at(&self, point: &Tuple) -> Color {
            match self.pattern {
                Pattern::Stripe(a, b) => {
                    if (point.x.floor() as i32) % 2 == 0 {
                        a.clone()
                    } else {
                        b.clone()
                    }
                },
                Pattern::Solid(color) => color.clone(),
                Pattern::Gradient(a, b) => {
                    let distance = b.subtract(&a);
                    let fraction = point.x - point.x.floor();
                    a.add(&distance.multiply(fraction))
                }
            }
        }

        pub fn pattern_at_object(&self, shape: &Shape, world_point: &Tuple) -> Color {
            let object_point = shape.transform.inverse().multiply_tuple(world_point);
            let pattern_point = self.transform.inverse().multiply_tuple(&object_point);
            shape.material.pattern_at(&pattern_point)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::tuple::tuple::Tuple;
    use crate::light::light::Light;
    use crate::light::light::lighting;
    use crate::material::material::{Material, Pattern};
    use crate::matrix::matrix::Matrix;
    use crate::shape::shape::Shape;

    #[test]
    fn surface_in_shadow() {
        let m = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let in_shadow = true;
        let result = lighting(&m, &Shape::sphere(),  &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn stripe_pattern_is_constant_in_y() {
        let m = Material::new(Pattern::Stripe(Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0)), Matrix::identity(4),1.0, 0.0, 0.0, 0.0);
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 1.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 2.0, 0.0)), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn stripe_pattern_is_constant_in_z() {
        let m = Material::new(Pattern::Stripe(Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0)), Matrix::identity(4), 1.0, 0.0, 0.0, 0.0);
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 0.0, 1.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 0.0, 2.0)), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn stripe_pattern_alternates_in_x() {
        let m = Material::new(Pattern::Stripe(Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0)), Matrix::identity(4), 1.0, 0.0, 0.0, 0.0);
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(0.9, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(1.0, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(m.pattern_at(&Tuple::point(-0.1, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(m.pattern_at(&Tuple::point(-1.0, 0.0, 0.0)), Color::new(0.0, 0.0, 0.0));
        assert_eq!(m.pattern_at(&Tuple::point(-1.1, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn gradient_pattern_linearly_interpolates_between_colors() {
        let m = Material::new(Pattern::Gradient(Color::new(1.0, 1.0, 1.0), Color::new(0.0, 0.0, 0.0)), Matrix::identity(4), 1.0, 0.0, 0.0, 0.0);
        assert_eq!(m.pattern_at(&Tuple::point(0.0, 0.0, 0.0)), Color::new(1.0, 1.0, 1.0));
        assert_eq!(m.pattern_at(&Tuple::point(0.25, 0.0, 0.0)), Color::new(0.75, 0.75, 0.75));
        assert_eq!(m.pattern_at(&Tuple::point(0.5, 0.0, 0.0)), Color::new(0.5, 0.5, 0.5));
        assert_eq!(m.pattern_at(&Tuple::point(0.75, 0.0, 0.0)), Color::new(0.25, 0.25, 0.25));
    }

}
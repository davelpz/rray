#[allow(dead_code)]

pub mod material {
    use crate::color::color::Color;
    use crate::matrix::matrix::Matrix;
    use crate::pattern::pattern::Pattern;
    use crate::shape::shape::Shape;
    use crate::tuple::tuple::Tuple;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Material {
        pub pattern: Pattern,
        pub ambient: f64,
        pub diffuse: f64,
        pub specular: f64,
        pub shininess: f64,
    }

    impl Material {
        pub fn new(pattern: Pattern, ambient: f64, diffuse: f64, specular: f64, shininess: f64) -> Material {
            Material { pattern, ambient, diffuse, specular, shininess }
        }

        pub fn default() -> Material {
            Material {
                pattern: Pattern::solid(Color::new(1.0, 1.0, 1.0), Matrix::identity(4)),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 200.0,
            }
        }

        pub fn pattern_at(&self, object_point: &Tuple) -> Color {
            self.pattern.pattern_at(object_point)
        }

        pub fn pattern_at_object(&self, shape: &Shape, world_point: &Tuple) -> Color {
            let object_point = shape.transform.inverse().multiply_tuple(world_point);
            shape.material.pattern_at(&object_point)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::tuple::tuple::Tuple;
    use crate::light::light::Light;
    use crate::light::light::lighting;
    use crate::material::material::{Material};
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
}
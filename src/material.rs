#[allow(dead_code)]

pub mod material {
    use crate::color::color::Color;
    use crate::matrix::matrix::Matrix;
    use crate::pattern::pattern::Pattern;
    use crate::shape::Shape;
    use crate::tuple::tuple::Tuple;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Material {
        pub pattern: Pattern,
        pub ambient: f64,
        pub diffuse: f64,
        pub specular: f64,
        pub shininess: f64,
        pub reflective: f64,
        pub transparency: f64,
        pub refractive_index: f64,
    }

    impl Material {
        pub fn new(pattern: Pattern, ambient: f64, diffuse: f64, specular: f64, shininess: f64, reflective: f64, transparency: f64, refractive_index: f64) -> Material {
            Material { pattern, ambient, diffuse, specular, shininess, reflective, transparency, refractive_index}
        }

        pub fn default() -> Material {
            Material {
                pattern: Pattern::solid(Color::new(1.0, 1.0, 1.0), Matrix::identity(4)),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 200.0,
                reflective: 0.0,
                transparency: 0.0,
                refractive_index: 1.0,
            }
        }
    }

    pub fn pattern_at_object(shape: &Shape, world_point: &Tuple) -> Color {
        let object_point = shape.transform.inverse().multiply_tuple(world_point);
        shape.material.pattern.pattern_at(&object_point)
    }

}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::tuple::tuple::Tuple;
    use crate::light::light::Light;
    use crate::light::light::lighting;
    use crate::material::material;
    use crate::material::material::{Material};
    use crate::matrix::matrix::Matrix;
    use crate::pattern::pattern::Pattern;
    use crate::shape::Shape;

    #[test]
    fn surface_in_shadow() {
        let m = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let in_shadow = true;
        let mut shape = Shape::sphere();
        shape.material = m;
        let result = lighting(&shape,  &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_pattern() {
        let mut shape = Shape::sphere();
        shape.transform = Matrix::scale(2.0, 2.0, 2.0);
        let mut m = Material::default();
        m.pattern = Pattern::test();
        m.pattern.transform = Matrix::translate(0.5, 1.0, 1.5);
        shape.material = m;
        let c = material::pattern_at_object(&shape, &Tuple::point(2.5, 3.0, 3.5));
        assert_eq!(c, Color::new(0.75, 0.5, 0.25));
    }
}
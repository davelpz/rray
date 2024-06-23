use crate::color::Color;
use crate::matrix::Matrix;
use crate::raytracer::material::pattern::Pattern;
use crate::tuple::Tuple;
use crate::raytracer::scene::get_object;

pub(crate) mod pattern;
mod noise;

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub pattern: pattern::Pattern,
    pub ambient: f64,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflective: f64,
    pub transparency: f64,
    pub refractive_index: f64,
}

impl Material {
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

pub fn pattern_at_object(shape: usize, world_point: &Tuple) -> Color {
    let shape = get_object(shape);
    let object_point = shape.get_transform().inverse().multiply_tuple(world_point);
    shape.get_material().pattern.pattern_at(&object_point)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::tuple::Tuple;
    use crate::matrix::Matrix;
    use crate::raytracer::light::{Light, lighting};
    use crate::raytracer::material::Material;
    use crate::raytracer::material::pattern_at_object;
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::scene::{add_object, number_of_objects};

    #[test]
    fn surface_in_shadow() {
        let m = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let in_shadow = true;
        let mut shape = Sphere::new();
        shape.material = m;
        add_object(Arc::new(shape));
        let id = number_of_objects() - 1;
        let result = lighting(id,  &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_pattern() {
        let mut shape = Sphere::new();
        shape.transform = Matrix::scale(2.0, 2.0, 2.0);
        let mut m = Material::default();
        m.pattern = Pattern::test();
        m.pattern.transform = Matrix::translate(0.5, 1.0, 1.5);
        shape.material = m;
        add_object(Arc::new(shape));
        let id = number_of_objects() - 1;
        let c = pattern_at_object(id, &Tuple::point(2.5, 3.0, 3.5));
        assert_eq!(c, Color::new(0.75, 0.5, 0.25));
    }
}
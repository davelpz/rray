use crate::color::Color;
use crate::matrix::Matrix;
use crate::raytracer::material::pattern::Pattern;
use crate::tuple::Tuple;
use crate::raytracer::object::db::get_object;
use crate::raytracer::object::world_to_object;

pub(crate) mod pattern;
mod noise;

/// Represents the material properties of a surface in a ray tracing scene.
///
/// This struct encapsulates various physical properties that define how a surface interacts with light.
/// These properties include the surface's pattern (texture), its ambient, diffuse, specular reflection
/// characteristics, shininess, reflectivity, transparency, and refractive index. These properties are
/// used in the lighting model to calculate the color of the surface under various lighting conditions.
///
/// # Fields
///
/// * `pattern` - The pattern applied to the surface, defining its texture.
/// * `ambient` - The ambient light reflection coefficient. Represents the constant color of the object
///   under ambient lighting.
/// * `diffuse` - The diffuse reflection coefficient. Determines how matte or bright the surface appears
///   under direct lighting.
/// * `specular` - The specular reflection coefficient. Controls the strength of the specular highlight.
/// * `shininess` - Affects the size of the specular highlight. Higher values result in smaller, sharper highlights.
/// * `reflective` - The reflectivity of the surface. A value of 0 means the surface is not reflective,
///   while 1 means it reflects light perfectly.
/// * `transparency` - The transparency of the material. A value of 0 means the material is opaque,
///   while 1 means it is completely transparent.
/// * `refractive_index` - The refractive index of the material, used in calculating refraction through
///   transparent materials.
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

/// Calculates the color of a pattern at a given point in world space for a specific object.
///
/// This function converts a point in world space to object space, then uses the object's material
/// pattern to determine the color at that point. It's a crucial part of the rendering process,
/// allowing patterns to be accurately applied to objects based on their world position and the
/// object's transformation.
///
/// # Arguments
///
/// * `shape` - The unique identifier (usize) of the object within the scene.
/// * `world_point` - A reference to a `Tuple` representing the point in world space where the color
///   is to be calculated.
///
/// # Returns
///
/// Returns a `Color` representing the color of the pattern at the specified point on the object.
pub fn pattern_at_object(shape: usize, world_point: &Tuple) -> Color {
    let object_point = world_to_object(shape, world_point);
    get_object(shape).get_material().pattern.pattern_at(&object_point)
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
    use crate::raytracer::scene::Scene;

    #[test]
    fn surface_in_shadow() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let m = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let in_shadow = 1.0;
        let mut shape = Sphere::new();
        shape.material = m;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id,  &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_pattern() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let mut shape = Sphere::new();
        shape.transform = Matrix::scale(2.0, 2.0, 2.0);
        let mut m = Material::default();
        m.pattern = Pattern::test();
        m.pattern.transform = Matrix::translate(0.5, 1.0, 1.5);
        shape.material = m;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let c = pattern_at_object(id, &Tuple::point(2.5, 3.0, 3.5));
        assert_eq!(c, Color::new(0.75, 0.5, 0.25));
    }
}
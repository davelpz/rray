use crate::color::Color;
use crate::tuple::Tuple;
use crate::raytracer::object::db::get_object;
use crate::raytracer::material::pattern_at_object;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LightType {
    Point,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Light {
    pub light_type: LightType,
    pub intensity: Color,
    pub position: Tuple,
}

impl Light {
    pub fn new_point_light(position: Tuple, intensity: Color) -> Light {
        Light { light_type: LightType::Point, intensity, position }
    }
}

pub fn lighting(object_id: usize, light: &Light, point: &Tuple, eyev: &Tuple, normalv: &Tuple, in_shadow: bool) -> Color {
    let object = get_object(object_id);
    let material = object.get_material();
    // Combine the surface color with the light's color/intensity
    let color = pattern_at_object(object_id, point);

    let effective_color = color.product(&light.intensity);
    // Find the direction to the light source
    let lightv = (light.position.subtract(point)).normalize();
    // Compute the ambient contribution
    let ambient = effective_color.multiply(material.ambient);

    if in_shadow {
        return ambient;
    }

    // Light_dot_normal represents the cosine of the angle between
    // the light vector and the normal vector.
    // A negative number means the light is on the other side of the surface.
    let light_dot_normal = lightv.dot(normalv);


    let diffuse;
    let specular;
    if light_dot_normal < 0.0 {
        diffuse = Color::new(0.0, 0.0, 0.0);
        specular = Color::new(0.0, 0.0, 0.0);
    } else {
        // Compute the diffuse contribution
        diffuse = effective_color.multiply(material.diffuse).multiply(light_dot_normal);
        let reflectv = lightv.negate().reflect(&normalv);
        // reflect_dot_eye represents the cosine of the angle between the
        // reflection vector and the eye vector. A negative number means the
        // light reflects away from the eye.
        let reflect_dot_eye = reflectv.dot(eyev);
        if reflect_dot_eye <= 0.0 {
            specular = Color::new(0.0, 0.0, 0.0);
        } else {
            // Compute the specular contribution
            let factor = reflect_dot_eye.powf(material.shininess);
            specular = light.intensity.multiply(material.specular).multiply(factor);
        }
    }
    // Add the three contributions together to get the final shading
    ambient.add(&diffuse).add(&specular)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::tuple::Tuple;
    use super::Light;
    use super::lighting;
    use crate::matrix::Matrix;
    use crate::raytracer::material::Material;
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::scene::Scene;

    #[test]
    fn a_point_light_has_a_position_and_intensity() {
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Tuple::point(0.0, 0.0, 0.0);
        let light = Light::new_point_light(position, intensity);
        assert_eq!(light.intensity, intensity);
        assert_eq!(light.position, position);
    }

    #[test]
    fn lighting_with_the_eye_between_the_light_and_the_surface() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_the_eye_between_light_and_surface_eye_offset_45_degrees() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 2_f64.sqrt() / 2.0, -2_f64.sqrt() / 2.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn lighting_with_eye_opposite_surface_light_offset_45_degrees() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));
    }

    #[test]
    fn lighting_with_eye_in_the_path_of_the_reflection_vector() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, -2_f64.sqrt() / 2.0, -2_f64.sqrt() / 2.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_the_light_behind_the_surface() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_a_pattern_applied() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let mut m = Material::default();
        m.ambient = 1.0;
        m.diffuse = 0.0;
        m.specular = 0.0;
        m.pattern = Pattern::stripe(Pattern::solid(Color::new(1.0, 1.0, 1.0), Matrix::identity(4)),
                                    Pattern::solid(Color::new(0.0, 0.0, 0.0), Matrix::identity(4)),
                                    Matrix::identity(4));
        let mut object = Sphere::new();
        object.material = m;
        w.add_object(Arc::new(object));
        let id = w.ids[0];
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let c1 = lighting(id, &light, &Tuple::point(0.9, 0.0, 0.0), &eyev, &normalv, false);
        let c2 = lighting(id, &light, &Tuple::point(1.1, 0.0, 0.0), &eyev, &normalv, false);
        assert_eq!(c1, Color::new(1.0, 1.0, 1.0));
        assert_eq!(c2, Color::new(0.0, 0.0, 0.0));
    }
}
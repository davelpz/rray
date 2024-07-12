use rand::{Rng, thread_rng};
use crate::color::Color;
use crate::tuple::Tuple;
use crate::raytracer::object::db::get_object;
use crate::raytracer::material::pattern_at_object;

/// Enum representing the different types of light sources.
/// Currently only supports point lights.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LightType {
    Point,
    Area(Tuple, Tuple, Tuple, usize) // corner, u vector, v vector, number of samples
}

/// Represents a light source in the scene.
///
/// This struct encapsulates the properties of a light source, including its type
/// (e.g., point light), intensity (color and brightness), and position in the scene.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Light {
    pub light_type: LightType,
    pub intensity: Color,
    pub position: Tuple,
}

impl Light {
    /// Constructs a new point light source.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the light source in 3D space.
    /// * `intensity` - The color and intensity of the light.
    ///
    /// # Returns
    ///
    /// A new `Light` instance configured as a point light source.
    pub fn new_point_light(position: Tuple, intensity: Color) -> Light {
        Light { light_type: LightType::Point, intensity, position }
    }

    pub fn new_area_light(corner: Tuple, u: Tuple, v: Tuple, intensity: Color, samples: usize) -> Light {
        //find the center of the area light
        let center = corner.add(&u.multiply(0.5)).add(&v.multiply(0.5));
        Light { light_type: LightType::Area(corner, u, v, samples), intensity, position: center }
    }

    pub fn sample_point(&self) -> Tuple {
        match self.light_type {
            LightType::Point => self.position,
            LightType::Area(corner, u, v, _samples) => {
                let mut rng = thread_rng();
                let u_rand = rng.gen_range(0.0..1.0);
                let v_rand = rng.gen_range(0.0..1.0);
                corner.add(&u.multiply(u_rand)).add(&v.multiply(v_rand))
            }
        }
    }
}

#[allow(dead_code)]
fn random_in_unit_sphere() -> Tuple {
    let mut rng = thread_rng();
    loop {
        let p = Tuple::vector(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        if p.magnitude() < 1.0 {
            return p;
        }
    }
}

/// Computes the color at a point on an object, taking into account the light source,
/// the viewer's position, and whether the point is in shadow.
///
/// This function implements the Phong reflection model to calculate the color of a point
/// on an object's surface. It considers the object's material properties, the light's
/// intensity and position, and whether the point is in shadow.
///
/// # Arguments
///
/// * `object_id` - The ID of the object being illuminated.
/// * `light` - A reference to the light source illuminating the object.
/// * `point` - The point on the object's surface being illuminated.
/// * `eyev` - The vector from the point to the viewer's eye.
/// * `normalv` - The normal vector at the point on the object's surface.
/// * `in_shadow` - a f64 between 0.0 and 1.0 representing the amount of shadow.
///
/// # Returns
///
/// The computed color at the given point on the object.
pub fn lighting(object_id: usize, light: &Light, point: &Tuple, eyev: &Tuple, normalv: &Tuple, in_shadow: f64) -> Color {
    let object = get_object(object_id);
    let material = object.get_material();
    // Combine the surface color with the light's color/intensity
    let color = pattern_at_object(object_id, point);

    let effective_color = color.product(&light.intensity);
    // Find the direction to the light source
    let lightv = (light.position.subtract(point)).normalize();
    // Compute the ambient contribution
    let ambient = effective_color.multiply(material.ambient);

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
    // include the shadow factor
    let diffuse_specular = diffuse.add(&specular).multiply(1.0 - in_shadow);
    ambient.add(&diffuse_specular)
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
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, 0.0);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_the_eye_between_light_and_surface_eye_offset_45_degrees() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 2_f64.sqrt() / 2.0, -2_f64.sqrt() / 2.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, 0.0);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn lighting_with_eye_opposite_surface_light_offset_45_degrees() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, 0.0);
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));
    }

    #[test]
    fn lighting_with_eye_in_the_path_of_the_reflection_vector() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, -2_f64.sqrt() / 2.0, -2_f64.sqrt() / 2.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, 0.0);
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_the_light_behind_the_surface() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0)));
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, 10.0), Color::new(1.0, 1.0, 1.0));
        let mut shape = Sphere::new();
        shape.material = material;
        w.add_object(Arc::new(shape));
        let id = w.ids[0];
        let result = lighting(id, &light, &position, &eyev, &normalv, 0.0);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn lighting_with_a_pattern_applied() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));

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
        let c1 = lighting(id, &light, &Tuple::point(0.9, 0.0, 0.0), &eyev, &normalv, 0.0);
        let c2 = lighting(id, &light, &Tuple::point(1.1, 0.0, 0.0), &eyev, &normalv, 0.0);
        assert_eq!(c1, Color::new(1.0, 1.0, 1.0));
        assert_eq!(c2, Color::new(0.0, 0.0, 0.0));
    }
}
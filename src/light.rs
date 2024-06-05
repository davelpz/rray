#[allow(dead_code)]

pub mod light {
    use crate::color::color::Color;
    use crate::tuple::tuple::Tuple;
    use crate::material::material::Material;

    #[derive(Debug, Clone, PartialEq, Copy)]
    pub struct PointLight {
        pub intensity: Color,
        pub position: Tuple,
    }

    impl PointLight {
        pub fn new(intensity: Color, position: Tuple) -> PointLight {
            PointLight { intensity, position }
        }
    }

    pub fn lighting(material: &Material, light: &PointLight, point: &Tuple, eyev: &Tuple, normalv: &Tuple, in_shadow: bool) -> Color {
        // Combine the surface color with the light's color/intensity
        let effective_color = material.color.product(&light.intensity);
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
}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::tuple::tuple::Tuple;
    use super::light::PointLight;
    use super::light::lighting;
    use crate::material::material::Material;

    #[test]
    fn a_point_light_has_a_position_and_intensity() {
        let intensity = Color::new(1.0, 1.0, 1.0);
        let position = Tuple::point(0.0, 0.0, 0.0);
        let light = PointLight::new(intensity, position);
        assert_eq!(light.intensity, intensity);
        assert_eq!(light.position, position);
    }

    #[test]
    fn lighting_with_the_eye_between_the_light_and_the_surface() {
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.0, -10.0));
        let result = lighting(&material, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.9, 1.9, 1.9));
    }

    #[test]
    fn lighting_with_the_eye_between_light_and_surface_eye_offset_45_degrees() {
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 2_f64.sqrt() / 2.0, -2_f64.sqrt() / 2.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.0, -10.0));
        let result = lighting(&material, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn lighting_with_eye_opposite_surface_light_offset_45_degrees() {
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 10.0, -10.0));
        let result = lighting(&material, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.7364, 0.7364, 0.7364));
    }

    #[test]
    fn lighting_with_eye_in_the_path_of_the_reflection_vector() {
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, -2_f64.sqrt() / 2.0, -2_f64.sqrt() / 2.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 10.0, -10.0));
        let result = lighting(&material, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(1.6364, 1.6364, 1.6364));
    }

    #[test]
    fn lighting_with_the_light_behind_the_surface() {
        let material = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.0, 10.0));
        let result = lighting(&material, &light, &position, &eyev, &normalv, false);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }
}
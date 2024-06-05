#[allow(dead_code)]

pub mod material {
    use crate::color::color::Color;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Material {
        pub color: Color,
        pub ambient: f64,
        pub diffuse: f64,
        pub specular: f64,
        pub shininess: f64,
    }

    impl Material {
        pub fn new(color: Color, ambient: f64, diffuse: f64, specular: f64, shininess: f64) -> Material {
            Material { color, ambient, diffuse, specular, shininess }
        }

        pub fn default() -> Material {
            Material {
                color: Color::new(1.0, 1.0, 1.0),
                ambient: 0.1,
                diffuse: 0.9,
                specular: 0.9,
                shininess: 200.0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::tuple::tuple::Tuple;
    use crate::light::light::PointLight;
    use crate::light::light::lighting;
    use crate::material::material::Material;

    #[test]
    fn surface_in_shadow() {
        let m = Material::default();
        let position = Tuple::point(0.0, 0.0, 0.0);
        let eyev = Tuple::vector(0.0, 0.0, -1.0);
        let normalv = Tuple::vector(0.0, 0.0, -1.0);
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.0, -10.0));
        let in_shadow = true;
        let result = lighting(&m, &light, &position, &eyev, &normalv, in_shadow);
        assert_eq!(result, Color::new(0.1, 0.1, 0.1));
    }
}
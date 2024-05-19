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

pub mod scene {
    use serde::Deserialize;
    use serde_json;
    use std::fs;
    use std::path::Path;

    #[derive(Deserialize)]
    pub struct Scene {
        pub camera: Camera,
        pub lights: Vec<Light>,
        pub scene: Vec<SceneObject>,
    }

    #[derive(Deserialize)]
    pub struct Camera {
        pub fov: f64,
        pub from: Vec<f64>,
        pub to: Vec<f64>,
        pub up: Vec<f64>,
    }

    #[derive(Deserialize)]
    pub struct Light {
        #[serde(rename = "type")]
        pub light_type: String,
        pub color: Vec<f64>,
        pub position: Vec<f64>,
    }

    #[derive(Deserialize)]
    pub struct SceneObject {
        #[serde(rename = "type")]
        pub object_type: String,
        pub transforms: Vec<Transform>,
        pub material: Material,
    }

    #[derive(Deserialize, Clone)]
    pub struct Transform {
        #[serde(rename = "type")]
        pub transform_type: String,
        pub x: Option<f64>,
        pub y: Option<f64>,
        pub z: Option<f64>,
        pub axis: Option<String>,
        pub angle: Option<f32>,
        pub xy: Option<f64>,
        pub xz: Option<f64>,
        pub yx: Option<f64>,
        pub yz: Option<f64>,
        pub zx: Option<f64>,
        pub zy: Option<f64>,
    }

    #[derive(Deserialize)]
    pub struct Material {
        pub pattern: Pattern,
        pub transforms: Vec<Transform>,
        pub ambient: Option<f64>,
        pub diffuse: Option<f64>,
        pub specular: Option<f64>,
        pub shininess: Option<f64>,
    }

    #[derive(Deserialize, Clone)]
    pub struct Pattern {
        #[serde(rename = "type")]
        pub pattern_type: String,
        pub color: Option<Vec<f64>>,
        pub pattern_a: Option<Box<Pattern>>,
        pub pattern_b: Option<Box<Pattern>>,
    }

    impl Default for Pattern {
        fn default() -> Self {
            Pattern {
                pattern_type: "solid".to_string(),
                color: Some(vec![0.0, 0.0, 0.0]),
                pattern_a: None,
                pattern_b: None,
            }
        }
    }

    pub fn create_scene_from_json_str(json_string: &str) -> Option<Scene> {
        let scene: Result<Scene, _> = serde_json::from_str(json_string);
        match scene {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    pub fn create_scene_from_file(path: &str) -> Option<Scene> {
        if Path::new(path).exists() {
            let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
            create_scene_from_json_str(&contents)
        } else {
            panic!("File does not exist");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::scene::create_scene_from_json_str;

    #[test]
    fn test_create_scene_from_json() {
        let json_string = r#"
            {
                "camera": {
                    "fov": 90,
                    "from": [0.0, 0.0, 0.0],
                    "to": [0.0, 0.0, 1.0],
                    "up": [0.0, 1.0, 0.0]
                },
                "lights": [
                    {
                        "type": "point",
                        "color": [1.0, 1.0, 1.0],
                        "position": [0.0, 0.0, 0.0]
                    }
                ],
                "scene": [
                    {
                        "type": "sphere",
                        "transforms": [
                            {
                                "type": "translate",
                                "x": 1.0,
                                "y": 2.0,
                                "z": 3.0
                            }
                        ],
                        "material": {
                            "pattern": {
                                "type": "stripe",
                                "pattern_a": {
                                   "type": "solid",
                                   "color": [1.0, 0.0, 0.0]
                                },
                                "pattern_b": {
                                   "type": "solid",
                                   "color": [0.0, 1.0, 0.0]
                                }
                            },
                            "transforms": [],
                            "ambient": 0.1,
                            "diffuse": 0.9,
                            "specular": 0.9,
                            "shininess": 200
                        }
                    }
                ]
            }
        "#;
        let scene = create_scene_from_json_str(json_string);
        assert!(scene.is_some());
    }
}

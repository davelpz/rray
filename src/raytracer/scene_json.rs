use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct SceneJson {
    pub camera: CameraJson,
    pub lights: Vec<LightJson>,
    pub scene: Vec<SceneObject>,
}

#[derive(Deserialize)]
pub struct CameraJson {
    pub fov: f64,
    pub from: Vec<f64>,
    pub to: Vec<f64>,
    pub up: Vec<f64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct LightJson {
    #[serde(rename = "type")]
    pub light_type: String,
    pub color: Vec<f64>,
    pub position: Vec<f64>,
}

#[derive(Deserialize)]
pub struct SceneObject {
    #[serde(rename = "type")]
    pub object_type: String,
    pub transforms: Option<Vec<TransformJson>>,
    pub material: MaterialJson,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub closed: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct TransformJson {
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
pub struct MaterialJson {
    pub pattern: PatternJson,
    pub ambient: Option<f64>,
    pub diffuse: Option<f64>,
    pub specular: Option<f64>,
    pub shininess: Option<f64>,
    pub reflective: Option<f64>,
    pub transparency: Option<f64>,
    pub refractive_index: Option<f64>,
}

#[derive(Deserialize, Clone)]
pub struct PatternJson {
    #[serde(rename = "type")]
    pub pattern_type: String,
    pub color: Option<Vec<f64>>,
    pub color_a: Option<Vec<f64>>,
    pub color_b: Option<Vec<f64>>,
    pub pattern_a: Option<Box<PatternJson>>,
    pub pattern_b: Option<Box<PatternJson>>,
    pub transforms: Option<Vec<TransformJson>>,
    pub scale: Option<f64>,
    pub octaves: Option<i32>,
    pub persistence: Option<f64>,
}

impl Default for PatternJson {
    fn default() -> Self {
        PatternJson {
            pattern_type: "solid".to_string(),
            color: Some(vec![0.0, 0.0, 0.0]),
            color_a: None,
            color_b: None,
            pattern_a: None,
            pattern_b: None,
            transforms: Some(Vec::new()),
            scale: None,
            octaves: None,
            persistence: None,
        }
    }
}

pub fn create_scene_from_json_str(json_string: &str) -> Option<SceneJson> {
    let scene: Result<SceneJson, _> = serde_json::from_str(json_string);
    match scene {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

pub fn create_scene_from_file(path: &str) -> Option<SceneJson> {
    if Path::new(path).exists() {
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        create_scene_from_json_str(&contents)
    } else {
        panic!("File does not exist");
    }
}


#[cfg(test)]
mod tests {
    use crate::raytracer::scene_json::create_scene_from_json_str;

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
                                   "color": [1.0, 0.0, 0.0],
                                   "transforms": []
                                },
                                "pattern_b": {
                                   "type": "solid",
                                   "color": [0.0, 1.0, 0.0],
                                   "transforms": []
                                },
                                "transforms": []
                            },
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
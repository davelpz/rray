use serde::Deserialize;

#[derive(Deserialize)]
pub struct Scene {
    camera: Camera,
    lights: Vec<Light>,
    scene: Vec<SceneObject>,
}

#[derive(Deserialize)]
pub struct Camera {
    rows: u32,
    cols: u32,
    fov: u32,
    from: Vec<f64>,
    to: Vec<f64>,
    up: Vec<f64>,
}

#[derive(Deserialize)]
pub struct Light {
    #[serde(rename = "type")]
    light_type: String,
    color: Vec<f64>,
    position: Vec<f64>,
}

#[derive(Deserialize)]
pub struct SceneObject {
    #[serde(rename = "type")]
    object_type: String,
    transforms: Vec<Transform>,
    material: Material,
}

#[derive(Deserialize)]
pub struct Transform {
    #[serde(rename = "type")]
    transform_type: String,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    axis: Option<String>,
    angle: Option<u32>,
}

#[derive(Deserialize)]
pub struct Material {
    pattern: Pattern,
    transforms: Vec<Transform>,
    ambient: f64,
    diffuse: f64,
    specular: f64,
    shininess: u32,
}

#[derive(Deserialize)]
pub struct Pattern {
    #[serde(rename = "type")]
    pattern_type: String,
    color: Option<Vec<f64>>,
    color_a: Option<Vec<f64>>,
    color_b: Option<Vec<f64>>,
}
use std::fs;
use std::path::Path;
use crate::raytracer::scene::Scene;
use yaml_rust2::{Yaml, YamlLoader, YamlEmitter};
use yaml_rust2::yaml::{Array, Hash};
use crate::color::Color;
use crate::matrix::Matrix;
use crate::raytracer::camera::Camera;
use crate::raytracer::light::Light;
use crate::tuple::Tuple;

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

fn color_from_vec(v: &Array) -> Color {
    let r = get_f64(&v[0]);
    let g = get_f64(&v[1]);
    let b = get_f64(&v[2]);
    Color::new(r, g, b)
}

fn point_from_vec(v: &Array) -> Tuple {
    let x = get_f64(&v[0]);
    let y = get_f64(&v[1]);
    let z = get_f64(&v[2]);
    Tuple::point(x, y, z)
}

fn vector_from_vec(v: &Array) -> Tuple {
    let x = get_f64(&v[0]);
    let y = get_f64(&v[1]);
    let z = get_f64(&v[2]);
    Tuple::vector(x, y, z)
}

fn print_type(node: &Yaml) {
    match node {
        Yaml::Real(value) => println!("It's a Real with value: {}", value),
        Yaml::Integer(value) => println!("It's an Integer with value: {}", value),
        Yaml::String(value) => println!("It's a String with value: {}", value),
        Yaml::Boolean(value) => println!("It's a Boolean with value: {}", value),
        Yaml::Array(values) => println!("It's an Array with {} items", values.len()),
        Yaml::Hash(values) => println!("It's a Hash with {} items", values.len()),
        Yaml::Alias(value) => println!("It's an Alias with value: {}", value),
        Yaml::Null => println!("It's a Null"),
        Yaml::BadValue => println!("It's a BadValue"),
        _ => println!("Unknown or new variant of Yaml"),
    }
}

fn get_f64(node: &Yaml) -> f64 {
    match node {
        Yaml::Real(value) => value.parse::<f64>().unwrap(),
        Yaml::Integer(value) => *value as f64,
        _ => panic!("{:?} not a number", node),
    }
}

fn get_f64_hash(hash: &Hash, key: &str) -> f64 {
    let value = &hash[&Yaml::String(key.to_string())];
    get_f64(value)
}

fn create_camera(doc: &Yaml, width: usize, height: usize) -> Camera {
    let camera = doc["camera"].as_hash().expect("camera definition not found");
    //print_type(camera);
    let fov = get_f64_hash(camera, "fov");
    let from = camera[&Yaml::String("from".to_string())].as_vec().expect("camera.from not found");
    let to = camera[&Yaml::String("to".to_string())].as_vec().expect("camera.to not found");
    let up = camera[&Yaml::String("up".to_string())].as_vec().expect("camera.up not found");

    let mut c = Camera::new(
        width,
        height,
        degrees_to_radians(fov)
    );

    c.transform = Matrix::view_transform(
        point_from_vec(from),
        point_from_vec(to),
        vector_from_vec(up)
    );

    c
}

fn create_light(doc: &Yaml) -> Light {
    let lights = doc["lights"].as_vec().expect("lights not found");

    if lights.is_empty() {
        panic!("No lights found in scene");
    }

    //only supporting one light for now
    let light = &lights[0];
    let light_type = light["type"].as_str().expect("light.light_type not found");
    let color = light["color"].as_vec().expect("light.color not found");
    let position = light["position"].as_vec().expect("light.position not found");

    if light_type != "point" {
        panic!("Only point lights are supported");
    }

    Light::new_point_light(
        point_from_vec(position),
        color_from_vec(color)
    )
}

pub fn render_scene_from_str(contents: &str, width: usize, height: usize, png_file: &str) {
    let docs = YamlLoader::load_from_str(contents).unwrap();
    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    let camera = create_camera(doc, width, height);
    let scene = Scene::new(create_light(doc));

    let scene_yaml = doc["scene"].as_vec().expect("scene not found");

    for scene_object in scene_yaml {
        let object_type = scene_object["type"].as_str().expect("type not found");
        println!("object_type: {}", object_type);
    }

    let image = camera.render(&scene);
    //image.write_to_file(png_file);
}

pub fn render_scene_from_file(path: &str, width: usize, height: usize, png_file: &str) {
    if Path::new(path).exists() {
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        render_scene_from_str(&contents, width, height, png_file)
    } else {
        panic!("File does not exist");
    }
}


#[cfg(test)]
mod tests {
    use crate::raytracer::scene_builder_yaml::{render_scene_from_file};

    #[test]
    #[ignore]
    fn test_render_scene_from_file() {
        render_scene_from_file("example1.yaml", 800, 600, "canvas.png");
    }
}
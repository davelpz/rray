use std::fs;
use std::path::Path;
use std::sync::Arc;

use yaml_rust2::{Yaml, YamlLoader};
use yaml_rust2::yaml::{Array, Hash};

use crate::color::Color;
use crate::matrix::Matrix;
use crate::raytracer::camera::Camera;
use crate::raytracer::light::Light;
use crate::raytracer::load_obj::load_obj_file;
use crate::raytracer::material::Material;
use crate::raytracer::material::pattern::Pattern;
use crate::raytracer::object::cone::Cone;
use crate::raytracer::object::csg::{Csg, CsgOperation};
use crate::raytracer::object::cube::Cube;
use crate::raytracer::object::cylinder::Cylinder;
use crate::raytracer::object::group::Group;
use crate::raytracer::object::Object;
use crate::raytracer::object::plane::Plane;
use crate::raytracer::object::sphere::Sphere;
use crate::raytracer::object::torus::Torus;
use crate::raytracer::object::triangle::Triangle;
use crate::raytracer::scene::Scene;
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

#[allow(dead_code)]
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
    }
}

fn get_f64(node: &Yaml) -> f64 {
    match node {
        Yaml::Real(value) => value.parse::<f64>().unwrap(),
        Yaml::Integer(value) => *value as f64,
        _ => panic!("{:?} not a number", node),
    }
}

fn get_f64_default(node: &Yaml, default: f64) -> f64 {
    match node {
        Yaml::Real(value) => value.parse::<f64>().unwrap(),
        Yaml::Integer(value) => *value as f64,
        _ => default,
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
        degrees_to_radians(fov),
    );

    c.transform = Matrix::view_transform(
        point_from_vec(from),
        point_from_vec(to),
        vector_from_vec(up),
    );

    c
}

fn create_lights(doc: &Yaml) -> Vec<Light> {
    let mut created_lights: Vec<Light> = vec![];

    let lights = doc["lights"].as_vec().expect("lights not found");

    if lights.is_empty() {
        panic!("No lights found in scene");
    }

    for light in lights {
        let light_type = light["type"].as_str().expect("light.light_type not found");
        let color = light["color"].as_vec().expect("light.color not found");

        match light_type {
            "point" => {
                let position = light["position"].as_vec().expect("light.position not found");
                created_lights.push(Light::new_point_light(
                    point_from_vec(position),
                    color_from_vec(color),
                ));
            }
            "area" => {
                let corner = point_from_vec(&light["corner"].as_vec().unwrap());
                let uvec = vector_from_vec(&light["uvec"].as_vec().unwrap());
                let vvec = vector_from_vec(&light["vvec"].as_vec().unwrap());
                let level = light["level"].as_i64().unwrap_or(5) as usize;
                created_lights.push(Light::new_area_light(
                    corner,
                    uvec,
                    vvec,
                    color_from_vec(color),
                    level,
                ));
            }
            _ => panic!("Unknown light type: {}", light_type),
        }
    }

    created_lights
}
fn create_csg(shape: &Yaml) -> Arc<dyn Object> {
    let operation_str = shape["operation"].as_str().expect("operation not found");
    let operation: Result<CsgOperation, _> = operation_str.parse();
    let operation = operation.expect(format!("Unknown operation: {}", operation_str).as_str());
    let mut csg = Csg::new(operation);
    let left = create_shape(&shape["left"]);
    let right = create_shape(&shape["right"]);
    csg.set_left(left);
    csg.set_right(right);
    Arc::new(csg)
}

fn create_group(shape: &Yaml) -> Arc<dyn Object> {
    let mut group = Group::new();
    let children = shape["children"].as_vec().expect("children not found");

    for child in children {
        let hidden = child["hidden"].as_bool().unwrap_or(false);
        if !hidden {
            group.add_child(create_shape(child));
        }
    }

    Arc::new(group)
}

fn create_matrix(transform: &Yaml) -> Matrix {
    let transform_type = transform["type"].as_str().expect("transform type not found");
    match transform_type {
        "translate" => {
            let amount = transform["amount"].as_vec().expect("amount not found");
            let x = get_f64(&amount[0]);
            let y = get_f64(&amount[1]);
            let z = get_f64(&amount[2]);
            Matrix::translate(x, y, z)
        }
        "scale" => {
            let amount = transform["amount"].as_vec().expect("amount not found");
            let x = get_f64(&amount[0]);
            let y = get_f64(&amount[1]);
            let z = get_f64(&amount[2]);
            Matrix::scale(x, y, z)
        }
        "rotate" => {
            let angle = degrees_to_radians(get_f64(&transform["angle"]));
            let axis = transform["axis"].as_str().expect("axis not found");
            match axis {
                "x" => Matrix::rotate_x(angle),
                "y" => Matrix::rotate_y(angle),
                "z" => Matrix::rotate_z(angle),
                _ => panic!("Unknown axis: {}", axis),
            }
        }
        "shear" => {
            let xy = get_f64(&transform["xy"]);
            let xz = get_f64(&transform["xz"]);
            let yx = get_f64(&transform["yx"]);
            let yz = get_f64(&transform["yz"]);
            let zx = get_f64(&transform["zx"]);
            let zy = get_f64(&transform["zy"]);
            Matrix::shear(xy, xz, yx, yz, zx, zy)
        }
        _ => panic!("Unknown transform type: {}", transform_type),
    }
}

fn create_transforms(transforms: &Array) -> Matrix {
    let mut m = Matrix::identity(4);
    for t in transforms.iter().rev() {
        m = m * create_matrix(t);
    }
    m
}

fn create_pattern(pattern: &Yaml) -> Pattern {
    let transform = create_transforms(&pattern["transforms"].as_vec().unwrap_or(&vec![]));
    let pattern_type = pattern["type"].as_str().expect("pattern type not found");
    let color = &pattern["color"];
    let black = vec![Yaml::Real("0.0".to_string()), Yaml::Real("0.0".to_string()), Yaml::Real("0.0".to_string())];
    let color: &Array = if color.is_badvalue() {
        &black
    } else {
        color.as_vec().unwrap()
    };
    let color_a = &pattern["color_a"];
    let color_b = &pattern["color_b"];
    let pattern_a = &pattern["pattern_a"];
    let pattern_b = &pattern["pattern_b"];

    match pattern_type {
        "solid" => {
            Pattern::solid(color_from_vec(color), transform)
        }
        "stripe" => {
            Pattern::stripe(get_sub_pattern(&transform, color_a, pattern_a),
                            get_sub_pattern(&transform, color_b, pattern_b),
                            transform.clone())
        }
        "gradient" => {
            Pattern::gradient(get_sub_pattern(&transform, color_a, pattern_a),
                              get_sub_pattern(&transform, color_b, pattern_b),
                              transform.clone())
        }
        "ring" => {
            Pattern::ring(get_sub_pattern(&transform, color_a, pattern_a),
                          get_sub_pattern(&transform, color_b, pattern_b),
                          transform.clone())
        }
        "checker" => {
            Pattern::checker(get_sub_pattern(&transform, color_a, pattern_a),
                              get_sub_pattern(&transform, color_b, pattern_b),
                              transform.clone())
        }
        "blend" => {
            let scale = get_f64_default(&pattern["scale"], 0.5);
            Pattern::blend(get_sub_pattern(&transform, color_a, pattern_a),
                           get_sub_pattern(&transform, color_b, pattern_b),
                           scale,
                           transform.clone())
        }
        "perturbed" => {
            let scale = get_f64_default(&pattern["scale"], 0.2);
            let octaves = get_f64_default(&pattern["octaves"], 3.0);
            let persistence = get_f64_default(&pattern["persistence"], 0.5);
            Pattern::perturbed(get_sub_pattern(&transform, color_a, pattern_a),
                               scale,
                               octaves as usize,
                               persistence,
                               transform.clone())
        }
        "noise" => {
            let octaves = get_f64_default(&pattern["octaves"], 1.0);
            let persistence = get_f64_default(&pattern["persistence"], 1.0);
            let scale = get_f64_default(&pattern["scale"], 1.0);
            Pattern::noise(get_sub_pattern(&transform, color_a, pattern_a),
                           get_sub_pattern(&transform, color_b, pattern_b),
                           scale,
                           octaves as usize,
                           persistence,
                           transform.clone())
        }
        _ => Pattern::solid(Color::new(0.0, 0.0, 0.0), transform.clone()),
    }
}

fn get_sub_pattern(transform: &Matrix, color: &Yaml, pattern_yaml: &Yaml) -> Pattern {
    let pattern = if color.is_array() {
        Pattern::solid(color_from_vec(color.as_vec().unwrap()), transform.clone())
    } else {
        create_pattern(pattern_yaml)
    };
    pattern
}

fn create_material(material: &Yaml) -> Material {
    let mut m = Material::default();
    if !material.is_badvalue() {
        m.ambient = get_f64_default(&material["ambient"], 0.1);
        m.diffuse = get_f64_default(&material["diffuse"], 0.9);
        m.specular = get_f64_default(&material["specular"], 0.9);
        m.shininess = get_f64_default(&material["shininess"], 200.0);
        m.reflective = get_f64_default(&material["reflective"], 0.0);
        m.transparency = get_f64_default(&material["transparency"], 0.0);
        m.refractive_index = get_f64_default(&material["refractive_index"], 1.0);
        m.pattern = create_pattern(&material["pattern"]);
    }
    m
}

fn create_shape(shape: &Yaml) -> Arc<dyn Object> {
    let object_type = shape["type"].as_str().expect("type not found");
    let mut s: Arc<dyn Object> = match object_type {
        "sphere" => Arc::new(Sphere::new()),
        "glass_sphere" => Arc::new(Sphere::glass_sphere()),
        "plane" => Arc::new(Plane::new()),
        "cube" => Arc::new(Cube::new()),
        "cylinder" => {
            let minimum = get_f64_default(&shape["minimum"], -f64::INFINITY);
            let maximum = get_f64_default(&shape["maximum"], f64::INFINITY);
            let closed = shape["closed"].as_bool().unwrap_or(false);
            Arc::new(Cylinder::new(minimum, maximum, closed))
        }
        "cone" => {
            let minimum = get_f64_default(&shape["minimum"], -f64::INFINITY);
            let maximum = get_f64_default(&shape["maximum"], f64::INFINITY);
            let closed = shape["closed"].as_bool().unwrap_or(false);
            Arc::new(Cone::new(minimum, maximum, closed))
        }
        "triangle" => {
            let p1 = point_from_vec(&shape["p1"].as_vec().unwrap());
            let p2 = point_from_vec(&shape["p2"].as_vec().unwrap());
            let p3 = point_from_vec(&shape["p3"].as_vec().unwrap());
            Arc::new(Triangle::new(p1, p2, p3))
        }
        "torus" => {
            let minor_radius = get_f64(&shape["minor_radius"]);
            Arc::new(Torus::new(minor_radius))
        }
        "obj_file" => {
            let file = shape["obj_file"].as_str().unwrap();
            Arc::new(load_obj_file(file, create_material(&shape["material"])))
        }
        "group" => create_group(shape),
        "csg" => create_csg(shape),
        _ => panic!("Unknown object type: {}", object_type),
    };
    Arc::get_mut(&mut s).unwrap().set_transform(create_transforms(shape["transforms"].as_vec().unwrap_or(&vec![])));
    Arc::get_mut(&mut s).unwrap().set_material(create_material(&shape["material"]));
    s
}

/// Renders a scene based on YAML string input.
///
/// This function takes a YAML string that defines a scene, including camera settings, lights, and objects,
/// and renders it to an image file. The rendering process involves parsing the YAML to extract scene elements,
/// setting up the camera with the specified field of view and transformations, adding lights to the scene,
/// and creating objects with specified materials and transformations. Finally, it renders the scene using
/// the camera and saves the rendered image to a file.
///
/// # Arguments
///
/// * `contents` - A string slice containing the YAML formatted scene description.
/// * `width` - The width of the output image in pixels, before applying anti-aliasing.
/// * `height` - The height of the output image in pixels, before applying anti-aliasing.
/// * `png_file` - The path where the rendered image will be saved.
/// * `aa` - The anti-aliasing factor to be used in rendering. A higher value results in smoother edges but increases rendering time.
///
/// # Panics
///
/// This function panics if the YAML content cannot be parsed, if required scene elements like the camera or lights
/// are not found in the YAML, or if specified objects have unsupported types or missing properties.
pub fn render_scene_from_str(contents: &str, width: usize, height: usize, png_file: &str, aa: usize) {
    let docs = YamlLoader::load_from_str(contents).unwrap();
    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    let camera = create_camera(doc, width * aa, height * aa);
    let mut scene = Scene::new();
    for light in create_lights(doc) {
        scene.add_light(light);
    }

    let scene_yaml = doc["scene"].as_vec().expect("scene not found");

    for scene_object in scene_yaml {
        let hidden = scene_object["hidden"].as_bool().unwrap_or(false);
        if !hidden {
            let shape = create_shape(scene_object);
            scene.add_object(shape);
        }
    }

    let image = camera.render(&scene);
    image.write_to_file(png_file, aa);
}

/// Renders a scene from a YAML file.
///
/// This function reads a scene configuration from a YAML file specified by `path`, then renders
/// the scene to an image file. The rendering process involves creating a camera, lights, and objects
/// as defined in the YAML file, and then using the camera to render the scene to the specified PNG file.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the YAML file containing the scene configuration.
/// * `width` - The width of the output image in pixels.
/// * `height` - The height of the output image in pixels.
/// * `png_file` - The path where the rendered image will be saved.
/// * `aa` - The anti-aliasing factor to be used in rendering. A higher value results in smoother edges but increases rendering time.
///
/// # Panics
///
/// Panics if the YAML file specified by `path` does not exist or cannot be read, or if the file does not contain a valid scene configuration.
pub fn render_scene_from_file(path: &str, width: usize, height: usize, png_file: &str, aa: usize) {
    if Path::new(path).exists() {
        let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
        render_scene_from_str(&contents, width, height, png_file, aa)
    } else {
        panic!("File does not exist");
    }
}


#[cfg(test)]
mod tests {
    use crate::raytracer::scene_builder_yaml::render_scene_from_file;

    #[test]
    #[ignore]
    fn test_render_scene_from_file() {
        render_scene_from_file("example1.yaml", 800, 400, "canvas.png",1);
    }
}
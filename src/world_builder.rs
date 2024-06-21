use crate::camera::Camera;
use crate::color::Color;
use crate::light::Light;
use crate::material::Material;
use crate::matrix::Matrix;
use crate::pattern::Pattern;
use crate::scene;
use crate::scene::{Scene, SceneObject, Transform};
use crate::shape::Shape;
use crate::tuple::Tuple;
use crate::world::world::World;

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

fn point_from_vec(v: &Vec<f64>) -> Tuple {
    Tuple::point(v[0], v[1], v[2])
}

fn vector_from_vec(v: &Vec<f64>) -> Tuple {
    Tuple::vector(v[0], v[1], v[2])
}

fn color_from_vec(v: &Vec<f64>) -> Color {
    Color::new(v[0], v[1], v[2])
}

fn create_camera(scene: &Scene, width: usize, height: usize) -> Camera {
    let mut c = Camera::new(
        width,
        height,
        degrees_to_radians(scene.camera.fov));
    let from = point_from_vec(&scene.camera.from);
    let to = point_from_vec(&scene.camera.to);
    let up = vector_from_vec(&scene.camera.up);
    c.transform = Matrix::view_transform(from, to, up);
    c
}

fn create_matrix(transform: &Transform) -> Matrix {
    match transform.transform_type.as_str() {
        "translate" => {
            let x = transform.x.unwrap_or(0.0);
            let y = transform.y.unwrap_or(0.0);
            let z = transform.z.unwrap_or(0.0);
            Matrix::translate(x, y, z)
        },
        "scale" => {
            let x = transform.x.unwrap_or(0.0);
            let y = transform.y.unwrap_or(0.0);
            let z = transform.z.unwrap_or(0.0);
            Matrix::scale(x, y, z)
        },
        "rotate" => {
            let angle = transform.angle.unwrap_or(0.0);
            match transform.axis.as_deref() {
                Some("x") => Matrix::rotate_x(degrees_to_radians(angle as f64)),
                Some("y") => Matrix::rotate_y(degrees_to_radians(angle as f64)),
                Some("z") => Matrix::rotate_z(degrees_to_radians(angle as f64)),
                _ => Matrix::identity(4),
            }
        },
        "shear" => {
            let xy = transform.xy.unwrap_or(0.0);
            let xz = transform.xz.unwrap_or(0.0);
            let yx = transform.yx.unwrap_or(0.0);
            let yz = transform.yz.unwrap_or(0.0);
            let zx = transform.zx.unwrap_or(0.0);
            let zy = transform.zy.unwrap_or(0.0);
            Matrix::shear(xy, xz, yx, yz, zx, zy)
        },
        _ => Matrix::identity(4),
    }
}

fn create_transforms(transforms: &Vec<Transform>) -> Matrix {
    let mut m = Matrix::identity(4);
    for t in transforms.iter().rev() {
        m = m * create_matrix(t);
    }
    m
}

#[allow(dead_code)]
fn create_pattern(pattern: &scene::Pattern) -> Pattern {
    let transform = create_transforms(&pattern.transforms.clone().unwrap_or(vec![]));
    let pattern = pattern.clone();
    if pattern.pattern_type == "solid" {
        return Pattern::solid(color_from_vec(&pattern.color.unwrap_or(vec![0.0, 0.0, 0.0])), transform.clone());
    } else {
        let pattern_a = if pattern.color_a.is_some() {
            Pattern::solid(color_from_vec(&pattern.color_a.unwrap()), transform.clone())
        } else {
            create_pattern(&pattern.pattern_a.unwrap_or_default())
        };
        let pattern_b = if pattern.color_b.is_some() {
            Pattern::solid(color_from_vec(&pattern.color_b.unwrap()), transform.clone())
        } else {
            create_pattern(&pattern.pattern_b.unwrap_or_default())
        };
        match pattern.pattern_type.as_str() {
            "stripe" => Pattern::stripe(pattern_a, pattern_b, transform.clone()),
            "gradient" => Pattern::gradient(pattern_a, pattern_b, transform.clone()),
            "ring" => Pattern::ring(pattern_a, pattern_b, transform.clone()),
            "checker" => Pattern::checker(pattern_a, pattern_b, transform.clone()),
            "blend" => {
                let scale = pattern.scale.unwrap_or(0.5);
                Pattern::blend(pattern_a, pattern_b, scale, transform.clone())
            },
            "perturbed" => {
                let scale = pattern.scale.unwrap_or(0.2);
                let octaves = pattern.octaves.unwrap_or(3);
                let persistence = pattern.persistence.unwrap_or(0.5);
                Pattern::perturbed(pattern_a, scale, octaves as usize, persistence, transform.clone())
            },
            "noise" => {
                let octaves = pattern.octaves.unwrap_or(1);
                let persistence = pattern.persistence.unwrap_or(1.0);
                let scale = pattern.scale.unwrap_or(1.0);
                Pattern::noise(pattern_a, pattern_b, scale, octaves as usize, persistence, transform.clone())
            },
            _ => Pattern::solid(Color::new(0.0, 0.0, 0.0), transform.clone()),
        }
    }
}

fn create_material(material: &scene::Material) -> Material {
    let mut m = Material::default();
    m.ambient = material.ambient.unwrap_or(0.1);
    m.diffuse = material.diffuse.unwrap_or(0.9);
    m.specular = material.specular.unwrap_or(0.9);
    m.shininess = material.shininess.unwrap_or(200.0);
    m.reflective = material.reflective.unwrap_or(0.0);
    m.transparency = material.transparency.unwrap_or(0.0);
    m.refractive_index = material.refractive_index.unwrap_or(1.0);
    m.pattern = create_pattern(&material.pattern);
    m
}

fn create_shape(scene_object: &SceneObject) -> Shape {
    let mut s = match scene_object.object_type.as_str() {
        "sphere" => Shape::sphere(),
        "plane" => Shape::plane(),
        "cube" => Shape::cube(),
        "glass_sphere" => Shape::glass_sphere(),
        "cylinder" => {
            let minimum = scene_object.minimum.unwrap_or(-f64::INFINITY);
            let maximum = scene_object.maximum.unwrap_or(f64::INFINITY);
            let closed = scene_object.closed.unwrap_or(false);
            Shape::cylinder(minimum, maximum, closed)
        },
        "cone" => {
            let minimum = scene_object.minimum.unwrap_or(-f64::INFINITY);
            let maximum = scene_object.maximum.unwrap_or(f64::INFINITY);
            let closed = scene_object.closed.unwrap_or(false);
            Shape::cone(minimum, maximum, closed)
        },
        _ => Shape::sphere(),
    };
    s.transform = create_transforms(&scene_object.transforms.clone().unwrap_or(vec![]));
    s.material = create_material(&scene_object.material);
    s
}

pub fn render_scene(scene: Scene, width: usize, height: usize, file: &str) {
    let c = create_camera(&scene, width, height);

    //right now, only one light is supported
    let mut w = World::new(Light::new_point_light(
        point_from_vec(&scene.lights[0].position),
        color_from_vec(&scene.lights[0].color)));

    for scene_object in scene.scene.iter() {
        let s = create_shape(scene_object);
        w.objects.push(s);
    }

    let image = c.render(&w);
    image.write_to_file(file);
}
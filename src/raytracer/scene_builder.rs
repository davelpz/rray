use std::sync::Arc;
use crate::color::Color;
use crate::matrix::Matrix;
use crate::tuple::Tuple;
use crate::raytracer::camera::Camera;
use crate::raytracer::light::Light;
use crate::raytracer::load_obj::load_obj_file;
use crate::raytracer::material::Material;
use crate::raytracer::material::pattern::Pattern;
use crate::raytracer::object::cone::Cone;
use crate::raytracer::object::cube::Cube;
use crate::raytracer::object::cylinder::Cylinder;
use crate::raytracer::object::Object;
use crate::raytracer::object::plane::Plane;
use crate::raytracer::object::sphere::Sphere;
use crate::raytracer::object::group::Group;
use crate::raytracer::object::triangle::Triangle;
use crate::raytracer::scene::{Scene};
use crate::raytracer::scene_json::{MaterialJson, PatternJson, SceneJson, SceneObject, TransformJson};



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

fn create_camera(scene: &SceneJson, width: usize, height: usize) -> Camera {
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

fn create_matrix(transform: &TransformJson) -> Matrix {
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

fn create_transforms(transforms: &Vec<TransformJson>) -> Matrix {
    let mut m = Matrix::identity(4);
    for t in transforms.iter().rev() {
        m = m * create_matrix(t);
    }
    m
}

#[allow(dead_code)]
fn create_pattern(pattern: &PatternJson) -> Pattern {
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

fn create_material(material: &MaterialJson) -> Material {
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

fn create_group(scene_object: &SceneObject) -> Arc<dyn Object> {
    let mut g = Group::new();
    if scene_object.children.is_some() {
        let children = scene_object.children.as_ref().unwrap();
        for child in children {
            let hide = child.hidden.unwrap_or(false);
            if !hide {
                let s = create_shape(&child);
                g.add_child(s);
            }
        }
    }
    Arc::new(g)
}

fn create_shape(scene_object: &SceneObject) -> Arc<dyn Object> {
    let mut s: Arc<dyn Object> = match scene_object.object_type.as_str() {
        "sphere" => Arc::new(Sphere::new()),
        "glass_sphere" => Arc::new(Sphere::glass_sphere()),
        "plane" => Arc::new(Plane::new()),
        "cube" => Arc::new(Cube::new()),
        "cylinder" => {
            let minimum = scene_object.minimum.unwrap_or(-f64::INFINITY);
            let maximum = scene_object.maximum.unwrap_or(f64::INFINITY);
            let closed = scene_object.closed.unwrap_or(false);
            Arc::new(Cylinder::new(minimum, maximum, closed))
        },
        "cone" => {
            let minimum = scene_object.minimum.unwrap_or(-f64::INFINITY);
            let maximum = scene_object.maximum.unwrap_or(f64::INFINITY);
            let closed = scene_object.closed.unwrap_or(false);
            Arc::new(Cone::new(minimum, maximum, closed))
        },
        "group" => create_group(scene_object),
        "triangle" => {
            let point = scene_object.p1.clone().unwrap_or(vec![0.0, 0.0, 0.0]);
            let p1 = point_from_vec(&point);
            let point = scene_object.p2.clone().unwrap_or(vec![0.0, 0.0, 0.0]);
            let p2 = point_from_vec(&point);
            let point = scene_object.p3.clone().unwrap_or(vec![0.0, 0.0, 0.0]);
            let p3 = point_from_vec(&point);
            Arc::new(Triangle::new(p1, p2, p3))
        },
        "obj_file" => {
            let file = scene_object.obj_file.clone().unwrap_or("".to_string());
            Arc::new(load_obj_file(&file,create_material(&scene_object.material.clone().unwrap_or_default())))
        },
        _ => Arc::new(Sphere::new()),
    };
    Arc::get_mut(&mut s).unwrap().set_transform(create_transforms(&scene_object.transforms.clone().unwrap_or(vec![])));
    Arc::get_mut(&mut s).unwrap().set_material(create_material(&scene_object.material.clone().unwrap_or_default()));
    s
}

pub fn render_scene(scene: SceneJson, width: usize, height: usize, file: &str) {
    let c = create_camera(&scene, width, height);

    //right now, only one light is supported
    let mut w = Scene::new(Light::new_point_light(
        point_from_vec(&scene.lights[0].position),
        color_from_vec(&scene.lights[0].color)));

    for scene_object in scene.scene.iter() {
        let hide = scene_object.hidden.unwrap_or(false);
        if !hide {
            let s = create_shape(scene_object);
            w.add_object(s);
        }
    }

    let image = c.render(&w);
    image.write_to_file(file);
}

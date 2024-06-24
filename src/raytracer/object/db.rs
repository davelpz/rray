use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::Object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

lazy_static! {
    static ref GLOBAL_OBJECTS: Arc<Mutex<Vec<Arc<dyn Object + Send>>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn get_object(id: usize) -> Arc<dyn Object + Send> {
    let objects = GLOBAL_OBJECTS.lock().unwrap();
    if id < objects.len() {
        objects[id].clone()
    } else {
        panic!("Object not found: {}", id)
    }
}

#[allow(dead_code)]
fn number_of_objects() -> usize {
    let objects = GLOBAL_OBJECTS.lock().unwrap();
    objects.len()
}

#[allow(dead_code)]
fn clear_global_objects() {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    objects.clear();
}

pub fn insert_sentinel() -> usize {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    let id = objects.len();
    let sentinel = Arc::new(Sentinel {id, parent_id: 0});
    objects.push(sentinel);
    id
}

pub fn replace_sentinel(id: usize, object: Arc<dyn Object + Send>) {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    objects[id] = object;
}

struct Sentinel {
    id: usize,
    parent_id: usize
}

impl Object for Sentinel {
    fn intersect(&self, _ray: &Ray) -> Vec<Intersection> {
        vec![]
    }

    fn normal_at(&self, _point: &Tuple) -> Tuple {
        Tuple::vector(0.0, 0.0, 0.0)
    }

    fn get_transform(&self) -> &Matrix {
        panic!("Sentinel has no transform")
    }

    fn get_material(&self) -> &Material {
        panic!("Sentinel has no material")
    }

    fn set_transform(&mut self, _transform: Matrix) {
    }

    fn set_material(&mut self, _material: Material) {
    }

    fn debug_string(&self) -> String {
        format!("Sentinel")
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_parent_id(&self) -> Option<usize> {
        Some(self.parent_id)
    }

    fn set_parent_id(&mut self, _id: usize) {
        self.parent_id = _id;
    }
}
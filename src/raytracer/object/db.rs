use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::{AABB, Object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

// Global storage for all objects in the scene, allowing for thread-safe access.
lazy_static! {
    static ref GLOBAL_OBJECTS: Arc<Mutex<Vec<Arc<dyn Object + Send>>>> = Arc::new(Mutex::new(Vec::new()));
}

/// Retrieves an object from the global storage by its ID.
///
/// # Arguments
///
/// * `id` - The unique identifier for the object.
///
/// # Returns
///
/// Returns an `Arc<dyn Object + Send>` pointing to the object if found.
///
/// # Panics
///
/// Panics if an object with the given ID does not exist.
pub fn get_object(id: usize) -> Arc<dyn Object + Send> {
    let objects = GLOBAL_OBJECTS.lock().unwrap();
    if id < objects.len() {
        objects[id].clone()
    } else {
        panic!("Object not found: {}", id)
    }
}

/// Returns the number of objects currently stored in global storage.
///
/// # Returns
///
/// The number of objects as `usize`.
#[allow(dead_code)]
fn number_of_objects() -> usize {
    let objects = GLOBAL_OBJECTS.lock().unwrap();
    objects.len()
}

/// Clears all objects from the global storage.
#[allow(dead_code)]
fn clear_global_objects() {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    objects.clear();
}

/// Generates a new unique ID for an object and stores a sentinel object in its place.
///
/// # Returns
///
/// The new unique ID as `usize`.
pub fn get_next_id() -> usize {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    let id = objects.len();
    let sentinel = Arc::new(Sentinel {id, parent_id: None});
    objects.push(sentinel);
    id
}

/// Adds an object to the global storage, replacing the sentinel object at its ID.
///
/// # Arguments
///
/// * `object` - An `Arc<dyn Object + Send>` pointing to the object to be added.
pub fn add_object(object: Arc<dyn Object + Send>) {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    let id = object.get_id();
    objects[id] = object;
}

/// A placeholder object used to reserve an ID in the global storage.
struct Sentinel {
    id: usize,
    parent_id: Option<usize>,
}

impl Object for Sentinel {
    fn intersect(&self, _ray: &Ray) -> Vec<Intersection> {
        vec![]
    }

    fn local_intersect(&self, _ray: &Ray) -> Vec<Intersection> {
        panic!("Sentinel should not be intersected")
    }

    fn normal_at(&self, _point: &Tuple, _hit: &Intersection) -> Tuple {
        Tuple::vector(0.0, 0.0, 0.0)
    }

    fn local_normal_at(&self, _local_point: &Tuple, _hit: &Intersection) -> Tuple {
        panic!("Sentinel has no local normal")
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
        self.parent_id
    }

    fn set_parent_id(&mut self, id: usize) {
        self.parent_id = Some(id);
    }

    fn get_aabb(&self) -> AABB {
        AABB::new(Tuple::point(0.0, 0.0, 0.0), Tuple::point(0.0, 0.0, 0.0))
    }

    fn includes(&self, _object_id: usize) -> bool {
        false
    }
}
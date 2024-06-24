use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::raytracer::object::Object;

lazy_static! {
    static ref GLOBAL_OBJECTS: Arc<Mutex<Vec<Arc<dyn Object + Send>>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn add_object(mut object: Arc<dyn Object + Send>) -> usize {
    let mut objects = GLOBAL_OBJECTS.lock().unwrap();
    let id = objects.len();
    Arc::get_mut(&mut object).unwrap().set_id(id);
    objects.push(object);
    id
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

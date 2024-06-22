use crate::object::Object;
use crate::ray::{Intersection, Ray};
use crate::shape::{EPSILON, Shape};
use crate::tuple::Tuple;

pub fn local_intersect<'a>(plane: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    if ray.direction.y.abs() < EPSILON {
        vec![]
    } else {
        let t = -ray.origin.y / ray.direction.y;
        let plane: Box<dyn Object> = Box::new(plane.clone());
        vec![Intersection { t, object: &plane }]
    }
}

pub fn local_normal_at(_: &Shape, _world_point: &Tuple) -> Tuple {
    Tuple::vector(0.0, 1.0, 0.0)
}
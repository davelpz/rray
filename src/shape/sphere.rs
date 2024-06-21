use crate::ray::{Intersection, Ray};
use crate::shape::Shape;
use crate::tuple::Tuple;

const ORIGIN: Tuple = Tuple { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

pub fn local_intersect<'a>(sphere: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let sphere_to_ray = ray.origin.subtract(&ORIGIN);
    let a = ray.direction.dot(&ray.direction);
    let b = 2.0 * ray.direction.dot(&sphere_to_ray);
    let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
    let discriminant: f64 = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        vec![]
    } else {
        let t1: f64 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2: f64 = (-b + discriminant.sqrt()) / (2.0 * a);
        vec![Intersection { t: t1, object: sphere },
             Intersection { t: t2, object: sphere }]
    }
}

pub fn local_normal_at(_sphere: &Shape, world_point: &Tuple) -> Tuple {
    world_point.subtract(&Tuple::point(0.0, 0.0, 0.0))
}

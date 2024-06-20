use crate::ray::{Intersection, Ray};
use crate::shape::{Shape, ShapeType};
use crate::tuple::Tuple;
use crate::tuple::EPSILON;

pub fn local_intersect<'a>(cylinder: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let a = ray.direction.x.powi(2) + ray.direction.z.powi(2);
    if a.abs() < EPSILON {
        return vec![];
    }
    let b = 2.0 * ray.origin.x * ray.direction.x + 2.0 * ray.origin.z * ray.direction.z;
    let c = ray.origin.x.powi(2) + ray.origin.z.powi(2) - 1.0;
    let discriminant = b.powi(2) - 4.0 * a * c;
    if discriminant < 0.0 {
        return vec![];
    }
    let mut t0 = (-b - discriminant.sqrt()) / (2.0 * a);
    let mut t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    if t0 > t1 {
        std::mem::swap(&mut t0, &mut t1);
    }

    let ShapeType::Cylinder(minimum, maximum, _closed) = cylinder.shape_type else { panic!("ShapeType is not Cylinder") };
    let mut xs: Vec<Intersection> = vec![];
    let y0 = ray.origin.y + t0 * ray.direction.y;
    if minimum < y0 && y0 < maximum {
        xs.push(Intersection::new(t0, cylinder));
    }
    let y1 = ray.origin.y + t1 * ray.direction.y;
    if minimum < y1 && y1 < maximum {
        xs.push(Intersection::new(t1, cylinder));
    }

    xs
}

pub fn local_normal_at(_cylinder: &Shape, local_point: &Tuple) -> Tuple {
    Tuple::vector(local_point.x, 0.0, local_point.z)
}
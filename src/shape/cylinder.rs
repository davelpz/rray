use crate::ray::{Intersection, Ray};
use crate::shape::{Shape, ShapeType};
use crate::tuple::Tuple;
use crate::tuple::EPSILON;

pub fn local_intersect<'a>(cylinder: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let mut xs: Vec<Intersection> = vec![];
    let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
    if a.abs() > EPSILON {
        let b = 2.0 * ray.origin.x * ray.direction.x + 2.0 * ray.origin.z * ray.direction.z;
        let c = ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - 1.0;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return vec![];
        }
        let mut t0 = (-b - discriminant.sqrt()) / (2.0 * a);
        let mut t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let ShapeType::Cylinder(minimum, maximum, _closed) = cylinder.shape_type else { panic!("ShapeType is not Cylinder") };
        let y0 = ray.origin.y + t0 * ray.direction.y;
        if minimum < y0 && y0 < maximum {
            xs.push(Intersection::new(t0, cylinder));
        }
        let y1 = ray.origin.y + t1 * ray.direction.y;
        if minimum < y1 && y1 < maximum {
            xs.push(Intersection::new(t1, cylinder));
        }
    }

    intersect_caps(cylinder, ray).iter().for_each(|i| xs.push(i.clone()));

    xs
}

// Check if the intersection at `t` is within the radius of the cylinder
// at the end caps
fn check_cap(ray: &Ray, t: f64) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let z = ray.origin.z + t * ray.direction.z;
    (x * x + z * z) <= 1.0
}

fn intersect_caps<'a>(cylinder: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let ShapeType::Cylinder(minimum, maximum, closed) = cylinder.shape_type else { panic!("ShapeType is not Cylinder") };
    let mut xs: Vec<Intersection> = vec![];

    // Caps only matter if the cylinder is closed, and might be
    // intersected by the ray
    if !closed || ray.direction.y.abs() < EPSILON {
        return xs;
    }

    // Check for an intersection with the lower end cap by intersecting
    // the ray with the plane at y = minimum
    let t = (minimum - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t) {
        xs.push(Intersection::new(t, cylinder));
    }

    // Check for an intersection with the upper end cap by intersecting
    // the ray with the plane at y = maximum
    let t = (maximum - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t) {
        xs.push(Intersection::new(t, cylinder));
    }

    xs
}

pub fn local_normal_at(cylinder: &Shape, local_point: &Tuple) -> Tuple {
    let ShapeType::Cylinder(minimum, maximum, _closed) = cylinder.shape_type else { panic!("ShapeType is not Cylinder") };

    // Compute the square of the distance from the y-axis
    let dist = local_point.x * local_point.x + local_point.z * local_point.z;

    if dist < 1.0 && local_point.y >= maximum - EPSILON {
        Tuple::vector(0.0, 1.0, 0.0)
    } else if dist < 1.0 && local_point.y <= minimum + EPSILON {
        Tuple::vector(0.0, -1.0, 0.0)
    } else {
        Tuple::vector(local_point.x, 0.0, local_point.z)
    }
}

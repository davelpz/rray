use crate::ray::{Intersection, Ray};
use crate::shape::{Shape, ShapeType};
use crate::tuple::Tuple;
use crate::tuple::EPSILON;

pub fn local_intersect<'a>(cone: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let mut xs: Vec<Intersection> = vec![];
    let ShapeType::Cone(minimum, maximum, _closed) = cone.shape_type else { panic!("ShapeType is not Cone") };
    let a = ray.direction.x * ray.direction.x - ray.direction.y * ray.direction.y + ray.direction.z * ray.direction.z;
    let b = 2.0 * ray.origin.x * ray.direction.x - 2.0 * ray.origin.y * ray.direction.y + 2.0 * ray.origin.z * ray.direction.z;

    if a.abs() < EPSILON && b.abs() < EPSILON {
        intersect_caps(cone, ray).iter().for_each(|i| xs.push(i.clone()));
        return xs;
    }

    let c = ray.origin.x * ray.origin.x - ray.origin.y * ray.origin.y + ray.origin.z * ray.origin.z;

    if a.abs() < EPSILON {
        let t = -c / (2.0 * b);
        let y = ray.origin.y + t * ray.direction.y;
        if minimum < y && y < maximum {
            xs.push(Intersection::new(t, cone));
            return xs;
        }
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return vec![];
    }

    let mut t0 = (-b - discriminant.sqrt()) / (2.0 * a);
    let mut t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    if t0 > t1 {
        std::mem::swap(&mut t0, &mut t1);
    }

    let y0 = ray.origin.y + t0 * ray.direction.y;
    if minimum < y0 && y0 < maximum {
        xs.push(Intersection::new(t0, cone));
    }
    let y1 = ray.origin.y + t1 * ray.direction.y;
    if minimum < y1 && y1 < maximum {
        xs.push(Intersection::new(t1, cone));
    }

    intersect_caps(cone, ray).iter().for_each(|i| xs.push(i.clone()));

    xs
}

// Check if the intersection at `t` is within the radius of the cone
// at the end caps
fn check_cap(ray: &Ray, t: f64) -> bool {
    let x = ray.origin.x + t * ray.direction.x;
    let y = ray.origin.y + t * ray.direction.y;
    let z = ray.origin.z + t * ray.direction.z;
    (x * x + z * z) <= y * y
}

fn intersect_caps<'a>(cone: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let ShapeType::Cone(minimum, maximum, closed) = cone.shape_type else { panic!("ShapeType is not Cone") };
    let mut xs: Vec<Intersection> = vec![];

    // Caps only matter if the cone is closed, and might be
    // intersected by the ray
    if !closed || ray.direction.y.abs() < EPSILON {
        return xs;
    }

    // Check for an intersection with the lower end cap by intersecting
    // the ray with the plane at y = minimum
    let t = (minimum - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t) {
        xs.push(Intersection::new(t, cone));
    }

    // Check for an intersection with the upper end cap by intersecting
    // the ray with the plane at y = maximum
    let t = (maximum - ray.origin.y) / ray.direction.y;
    if check_cap(ray, t) {
        xs.push(Intersection::new(t, cone));
    }

    xs
}

pub fn local_normal_at(cone: &Shape, local_point: &Tuple) -> Tuple {
    let ShapeType::Cone(minimum, maximum, _closed) = cone.shape_type else { panic!("ShapeType is not Cone") };

    // Compute the square of the distance from the y-axis
    let dist = local_point.x * local_point.x + local_point.z * local_point.z;

    if dist < 1.0 && local_point.y >= maximum - EPSILON {
        Tuple::vector(0.0, 1.0, 0.0)
    } else if dist < 1.0 && local_point.y <= minimum + EPSILON {
        Tuple::vector(0.0, -1.0, 0.0)
    } else {
        let mut y = dist.sqrt();
        if local_point.y > 0.0 {
            y = -y;
        }

        Tuple::vector(local_point.x, y, local_point.z)
    }
}

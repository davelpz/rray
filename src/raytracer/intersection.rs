use crate::EPSILON;
use crate::raytracer::computations::Computations;
use crate::raytracer::ray::Ray;
use crate::raytracer::object::db::get_object;

/// Represents an intersection point on an object.
///
/// This struct captures the intersection of a ray with an object in the scene,
/// including the distance from the ray origin (`t`), the ID of the object intersected,
/// and optional texture coordinates (`u`, `v`) for texture mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct Intersection {
    pub t: f64,
    pub object: usize,
    pub u: f64,
    pub v: f64,
}

impl Intersection {
    /// Creates a new `Intersection`.
    ///
    /// # Arguments
    ///
    /// * `t` - The distance from the ray origin to the intersection.
    /// * `object` - The ID of the object intersected by the ray.
    /// * `u` - The horizontal texture coordinate at the intersection.
    /// * `v` - The vertical texture coordinate at the intersection.
    ///
    /// # Returns
    ///
    /// A new instance of `Intersection`.
    pub fn new(t: f64, object: usize, u: f64, v: f64) -> Intersection {
        Intersection { t, object, u, v }
    }

    /// Prepares the computations for shading this intersection.
    ///
    /// This method calculates various geometric properties needed for shading,
    /// such as the point of intersection, the eye vector, the normal vector,
    /// whether the intersection is inside the object, and more.
    ///
    /// # Arguments
    ///
    /// * `r` - The ray that produced this intersection.
    /// * `xs` - A list of all intersections with the object, for refraction calculations.
    ///
    /// # Returns
    ///
    /// A `Computations` struct containing the calculated properties.
    pub fn prepare_computations(&self, r: &Ray, xs: &Vec<Intersection>) -> Computations {
        let point = r.position(self.t);
        let eyev = r.direction.negate();
        let object = get_object(self.object);
        let normalv = object.normal_at(&point, self);
        let inside = normalv.dot(&eyev) < 0.0;
        let normalv = if inside { normalv.negate() } else { normalv };
        let over_point = point.add(&normalv.multiply(EPSILON));
        let under_point = point.subtract(&normalv.multiply(EPSILON));
        let reflectv = r.direction.reflect(&normalv);

        let mut n1 = 1.0;
        let mut n2 = 1.0;
        let mut containers: Vec<usize> = vec![];
        for i in xs {
            if *i == *self {
                if containers.is_empty() {
                    n1 = 1.0;
                } else {
                    let id = containers.last().unwrap();
                    let object = get_object(*id);
                    n1 = object.get_material().refractive_index;
                }
            }

            if containers.contains(&i.object) {
                if let Some(index) = containers.iter().position(|shape| *shape == i.object) {
                    containers.remove(index);
                }
            } else {
                containers.push(i.object);
            }

            if *i == *self {
                if containers.is_empty() {
                    n2 = 1.0;
                } else {
                    let id = containers.last().unwrap();
                    let object = get_object(*id);
                    n2 = object.get_material().refractive_index;
                }
            }
        }

        Computations { t: self.t, object: self.object, point, eyev, normalv, inside, over_point, under_point, reflectv, n1, n2 }
    }
}

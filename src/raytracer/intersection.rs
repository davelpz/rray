use crate::EPSILON;
use crate::raytracer::computations::Computations;
use crate::raytracer::ray::Ray;
use crate::raytracer::scene::get_object;
#[derive(Debug, Clone, PartialEq)]
pub struct Intersection {
    pub t: f64,
    pub object: usize,
}

impl Intersection {
    pub fn new(t: f64, object: usize) -> Intersection {
        Intersection { t, object }
    }

    pub fn prepare_computations(&self, r: &Ray, xs: &Vec<Intersection>) -> Computations {
        let point = r.position(self.t);
        let eyev = r.direction.negate();
        let object = get_object(self.object);
        let normalv = object.normal_at(&point);
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

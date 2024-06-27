pub(crate) mod sphere;
pub(crate) mod plane;
pub(crate) mod cone;
pub(crate) mod cube;
pub(crate) mod cylinder;
pub(crate) mod db;
pub(crate) mod group;

use std::fmt::{Debug, Formatter};
use crate::EPSILON;
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

pub trait Object: Sync + Send {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection>;
    fn normal_at(&self, point: &Tuple) -> Tuple;
    fn get_transform(&self) -> &Matrix;
    fn get_material(&self) -> &Material;
    fn set_transform(&mut self, transform: Matrix);
    fn set_material(&mut self, material: Material);
    fn debug_string(&self) -> String;
    fn get_id(&self) -> usize;
    fn get_parent_id(&self) -> Option<usize>;
    fn set_parent_id(&mut self, id: usize);
    fn get_aabb(&self) -> AABB;
}

impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Debug for dyn Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_string())
    }
}

pub fn world_to_object(object_id: usize, world_point: &Tuple) -> Tuple {
    let object = get_object(object_id);
    let mut point = world_point.clone();
    if let Some(parent_id) = object.get_parent_id() {
        point = world_to_object(parent_id, &point);
    }
    object.get_transform().inverse().multiply_tuple(&point)
}

pub fn normal_to_world(object_id: usize, object_normal: &Tuple) -> Tuple {
    let object = get_object(object_id);
    let mut normal = object.get_transform().inverse().transpose().multiply_tuple(&object_normal);
    normal.w = 0.0;
    normal = normal.normalize();
    if let Some(parent_id) = object.get_parent_id() {
        normal = normal_to_world(parent_id, &normal);
    }
    normal
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Tuple,
    pub max: Tuple,
}

impl AABB {
    pub fn new(min: Tuple, max: Tuple) -> AABB {
        AABB { min, max }
    }

    fn check_axis(origin: f64, direction: f64, min: f64, max: f64) -> (f64, f64) {
        let tmin_numerator = min - origin;
        let tmax_numerator = max - origin;

        let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (tmin_numerator * f64::INFINITY, tmax_numerator * f64::INFINITY)
        };

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        (tmin, tmax)
    }

    pub fn intersect(&self, r: &Ray) -> bool {
        let (tmin_x, tmax_x) = AABB::check_axis(r.origin.x, r.direction.x, self.min.x, self.max.x);
        let (tmin_y, tmax_y) = AABB::check_axis(r.origin.y, r.direction.y, self.min.y, self.max.y);
        let (tmin_z, tmax_z) = AABB::check_axis(r.origin.z, r.direction.z, self.min.z, self.max.z);

        let tmin = tmin_x.max(tmin_y.max(tmin_z));
        let tmax = tmax_x.min(tmax_y.min(tmax_z));

        tmin <= tmax
    }

    fn adjust_min_max(&mut self, x: f64, y:f64, z: f64) {
        self.min = Tuple::point(self.min.x.min(x),
                                self.min.y.min(y),
                                self.min.z.min(z));
        self.max = Tuple::point(self.max.x.max(x),
                                self.max.y.max(y),
                                self.max.z.max(z));
    }

    pub fn adjust_aabb(&mut self, aabb: &AABB) {
        self.adjust_min_max(aabb.min.x, aabb.min.y, aabb.min.z);
        self.adjust_min_max(aabb.max.x, aabb.max.y, aabb.max.z);
    }

    pub fn apply_transform(&self, transform: &Matrix) -> AABB {
          let corners = [
            Tuple::point(self.min.x, self.min.y, self.min.z),
            Tuple::point(self.min.x, self.min.y, self.max.z),
            Tuple::point(self.min.x, self.max.y, self.min.z),
            Tuple::point(self.min.x, self.max.y, self.max.z),
            Tuple::point(self.max.x, self.min.y, self.min.z),
            Tuple::point(self.max.x, self.min.y, self.max.z),
            Tuple::point(self.max.x, self.max.y, self.min.z),
            Tuple::point(self.max.x, self.max.y, self.max.z),
        ];

        let mut aabb = AABB::new(Tuple::point(f64::INFINITY, f64::INFINITY, f64::INFINITY), Tuple::point(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY));
        for corner in corners.iter() {
            let transformed = transform.multiply_tuple(corner);
            aabb.adjust_min_max(transformed.x, transformed.y, transformed.z);
        }
        aabb
    }

}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::raytracer::light::Light;
    use crate::raytracer::object::db::get_object;
    use crate::raytracer::object::Object;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::ray::Ray;
    use crate::raytracer::scene::Scene;
    use crate::tuple::Tuple;

    #[test]
    fn test_intersect() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Arc::new(Sphere::new());
        let s: Arc<dyn Object> = s;
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 6.0);

        let r = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        {
            let xs = s.intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, -1.0);
            assert_eq!(xs[1].t, 1.0);
        }

        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        {
            let xs = s.intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, -6.0);
            assert_eq!(xs[1].t, -4.0);
            assert_eq!(xs[0].object, s.get_id());
            assert_eq!(xs[1].object, s.get_id());
        }
    }

    #[test]
    fn test_transform() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::scale(2.0, 2.0, 2.0);
        let s = Arc::new(s);
        let s: Arc<dyn Object> = s;
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::translate(5.0, 0.0, 0.0);
        let s = Arc::new(s);
        let s: Arc<dyn Object> = s;
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn normal_at_surface_point() {
        let mut scene = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        let s = Arc::new(Sphere::new());
        let sid = scene.add_object(s);
        let s: Arc<dyn Object> = get_object(sid);
        let point = Tuple::point(1.0, 0.0, 0.0);
        let expected_normal = Tuple::vector(1.0, 0.0, 0.0);
        assert_eq!(s.normal_at(&point), expected_normal);

        let point = Tuple::point(0.0, 1.0, 0.0);
        let expected_normal = Tuple::vector(0.0, 1.0, 0.0);
        assert_eq!(s.normal_at(&point), expected_normal);

        let point = Tuple::point(0.0, 0.0, 1.0);
        let expected_normal = Tuple::vector(0.0, 0.0, 1.0);
        assert_eq!(s.normal_at(&point), expected_normal);
    }

    #[test]
    fn normal_at_non_axial_point() {
        let mut scene = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        let s = Arc::new(Sphere::new());
        let sid = scene.add_object(s);
        let s: Arc<dyn Object> = get_object(sid);
        let sqrt_of_three_over_three = 3f64.sqrt() / 3.0;
        let point = Tuple::point(sqrt_of_three_over_three, sqrt_of_three_over_three, sqrt_of_three_over_three);
        let expected_normal = Tuple::vector(sqrt_of_three_over_three, sqrt_of_three_over_three, sqrt_of_three_over_three);
        assert_eq!(s.normal_at(&point), expected_normal);
        assert_eq!(s.normal_at(&point).magnitude(), 1.0);
    }
}
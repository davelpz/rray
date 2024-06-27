pub(crate) mod sphere;
pub(crate) mod plane;
pub(crate) mod cone;
pub(crate) mod cube;
pub(crate) mod cylinder;
pub(crate) mod db;
pub(crate) mod group;

use std::fmt::{Debug, Formatter};
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
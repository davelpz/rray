use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::{AABB, Object};
use crate::raytracer::object::{normal_to_world, world_to_object};
use crate::raytracer::object::db::{add_object, get_next_id, get_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

pub struct Group {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub child_ids: Vec<usize>,
    aabb_cache: Arc<RwLock<Option<AABB>>>,  // Cache for the AABB wrapped in RwLock and Arc for thread safety
}

impl Group {
    pub fn new() -> Group {
        Group {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            child_ids: Vec::new(),
            aabb_cache: Arc::new(RwLock::new(None)),  // Initialize the cache as None
        }
    }

    fn invalidate_aabb_cache(&self) {
        // Invalidate the cache by acquiring a write lock and setting the value to None
        let mut cache = self.aabb_cache.write().unwrap();
        *cache = None;
    }

    fn get_aabb_cache(&self) -> RwLockReadGuard<Option<AABB>> {
        self.aabb_cache.read().unwrap()
    }

    fn set_aabb_cache(&self, aabb: AABB) {
        let mut cache = self.aabb_cache.write().unwrap();
        *cache = Some(aabb);
    }

    pub fn add_child(&mut self, mut object: Arc<dyn Object + Send>) -> usize {
        Arc::get_mut(&mut object).unwrap().set_parent_id(self.id);
        self.invalidate_aabb_cache();
        let child_id = object.get_id();
        add_object(object);
        self.child_ids.push(child_id);
        child_id
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = Vec::new();
        if self.get_aabb().intersect(ray) {
            for child_id in &self.child_ids {
                let child = get_object(*child_id);
                let mut child_xs = child.intersect(ray);
                xs.append(&mut child_xs);
            }
            xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        }
        xs
    }

    pub fn local_normal_at(&self, _vector: &Tuple) -> Tuple {
        panic!("Groups do not have normals")
    }


}

impl Object for Group {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        self.local_intersect(&trans_ray)
    }

    fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let local_point = world_to_object(self.id, world_point);
        let local_normal = self.local_normal_at(&local_point);
        normal_to_world(self.id, &local_normal)
    }

    fn get_transform(&self) -> &Matrix {
        &self.transform
    }

    fn get_material(&self) -> &Material {
        panic!("Groups do not have materials")
    }

    fn set_transform(&mut self, transform: Matrix) {
        self.transform = transform;
    }

    fn set_material(&mut self, _material: Material) {
    }

    fn debug_string(&self) -> String {
        format!("Group: transform: {:?}", self.transform)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_parent_id(&self) -> Option<usize> {
        self.parent_id
    }

    fn set_parent_id(&mut self, id: usize) {
        self.parent_id = Some(id);
    }

    fn get_aabb(&self) -> AABB {
        // Acquire a read lock and check if the cache is valid
        if let Some(cached_aabb) = *self.get_aabb_cache() {
            return cached_aabb;
        }

        let mut aabb: AABB = AABB::new(
            Tuple::point(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            Tuple::point(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        );

        for child_id in &self.child_ids {
            let child = get_object(*child_id);
            let child_aabb = child.get_aabb().apply_transform(child.get_transform());
            aabb.adjust_aabb(&child_aabb);
        }

        // Cache the computed AABB
        self.set_aabb_cache(aabb);

        aabb
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::raytracer::camera::Camera;
    use crate::raytracer::light::Light;
    use crate::raytracer::object::{normal_to_world, world_to_object};
    use crate::raytracer::object::cylinder::Cylinder;
    use crate::raytracer::object::db::get_object;
    use crate::raytracer::object::group::Group;
    use crate::raytracer::object::Object;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::ray::Ray;
    use crate::raytracer::scene::Scene;
    use crate::tuple::Tuple;

    #[test]
    fn intersecting_a_ray_with_an_empty_group() {
        let g = Group::new();
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.local_intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn intersecting_a_ray_with_a_nonempty_group() {
        let mut g = Group::new();
        g.id = 100;
        let s1 = Sphere::new();
        let mut s2 = Sphere::new();
        s2.set_transform(Matrix::translate(0.0, 0.0, -3.0));
        let mut s3 = Sphere::new();
        s3.set_transform(Matrix::translate(5.0, 0.0, 0.0));
        let s1_id = g.add_child(Arc::new(s1));
        let s2_id = g.add_child(Arc::new(s2));
        g.add_child(Arc::new(s3));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.local_intersect(&r);
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].object, s2_id);
        assert_eq!(xs[1].object, s2_id);
        assert_eq!(xs[2].object, s1_id);
        assert_eq!(xs[3].object, s1_id);
    }

    #[test]
    fn intersecting_a_transformed_group() {
        let mut g = Group::new();
        g.id = 100;
        g.set_transform(Matrix::scale(2.0, 2.0, 2.0));
        let mut s = Sphere::new();
        s.set_transform(Matrix::translate(5.0, 0.0, 0.0));
        g.add_child(Arc::new(s));
        let r = Ray::new(Tuple::point(10.0, 0.0, -10.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = g.intersect(&r);
        assert_eq!(xs.len(), 2);
    }

    #[test]
    fn converting_a_point_from_world_to_object_space() {
        let mut scene = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut g1 = Group::new();
        g1.set_transform(Matrix::rotate_y(std::f64::consts::PI / 2.0));

        let mut g2 = Group::new();
        g2.set_transform(Matrix::scale(2.0, 2.0, 2.0));

        let mut s = Sphere::new();
        let s_id = s.get_id();
        s.set_transform(Matrix::translate(5.0, 0.0, 0.0));
        let s: Arc<dyn Object + Send> = Arc::new(s);
        g2.add_child(s);

        g1.add_child(Arc::new(g2));

        scene.add_object(Arc::new(g1));

        let p = world_to_object(s_id, &Tuple::point(-2.0, 0.0, -10.0));
        assert_eq!(p, Tuple::point(0.0, 0.0, -1.0));
    }

    #[test]
    fn converting_a_normal_from_object_to_world_space() {
        let mut scene = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut g1 = Group::new();
        g1.set_transform(Matrix::rotate_y(std::f64::consts::PI / 2.0));

        let mut g2 = Group::new();
        g2.set_transform(Matrix::scale(1.0, 2.0, 3.0));

        let mut s = Sphere::new();
        let s_id = s.get_id();
        s.set_transform(Matrix::translate(5.0, 0.0, 0.0));
        let s: Arc<dyn Object + Send> = Arc::new(s);
        g2.add_child(s);

        g1.add_child(Arc::new(g2));

        scene.add_object(Arc::new(g1));

        let n = normal_to_world(s_id, &Tuple::vector(3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0, 3.0_f64.sqrt() / 3.0));
        assert_eq!(n, Tuple::vector(0.28571428571428575, 0.42857142857142855, -0.8571428571428571));
    }

    #[test]
    fn finding_the_normal_on_a_child_object() {
        let mut scene = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let mut g1 = Group::new();
        g1.set_transform(Matrix::rotate_y(std::f64::consts::PI / 2.0));
        let mut g2 = Group::new();
        g2.set_transform(Matrix::scale(1.0, 2.0, 3.0));
        let mut s = Sphere::new();
        s.set_transform(Matrix::translate(5.0, 0.0, 0.0));
        let s_id = s.get_id();
        let s: Arc<dyn Object + Send> = Arc::new(s);
        g2.add_child(s);
        g1.add_child(Arc::new(g2));
        scene.add_object(Arc::new(g1));

        let s = get_object(s_id);
        let n = s.normal_at(&Tuple::point(1.7321, 1.1547, -5.5774));
        assert_eq!(n, Tuple::vector(0.28570368184140726, 0.42854315178114105, -0.8571605294481017));
    }

    fn hexagon_corner() -> Sphere {
        let mut corner = Sphere::new();
        corner.set_transform(Matrix::translate(0.0, 0.0, -1.0) * Matrix::scale(0.25, 0.25, 0.25));
        corner
    }

    fn hexagon_edge() -> Cylinder {
        let mut edge = Cylinder::new(0.0, 1.0, true);
        edge.set_transform(Matrix::translate(0.0, 0.0, -1.0)
            * Matrix::rotate_y(-std::f64::consts::PI / 6.0)
            * Matrix::rotate_z(-std::f64::consts::PI / 2.0)
            * Matrix::scale(0.25, 1.0, 0.25));
        edge
    }

    fn hexagon_side() -> Group {
        let mut side = Group::new();
        side.add_child(Arc::new(hexagon_corner()));
        side.add_child(Arc::new(hexagon_edge()));
        side
    }

    fn hexagon() -> Group {
        let mut hex = Group::new();
        for n in 0..6 {
            let mut side = hexagon_side();
            side.set_transform(Matrix::rotate_y(n as f64 * std::f64::consts::PI / 3.0));
            hex.add_child(Arc::new(side));
        }
        hex
    }

    fn degrees_to_radians(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }

    #[test]
    #[ignore]
    fn constructing_a_hexagon() {
        use crate::color::Color;

        let mut c = Camera::new(800, 800, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 2.0, -5.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = Scene::new(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let mut g = hexagon();
        g.set_transform(Matrix::rotate_x(degrees_to_radians(-20.0)));
        w.add_object(Arc::new(g));

        let image = c.render(&w);

        image.write_to_file("canvas.png");
    }
}

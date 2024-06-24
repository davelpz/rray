use std::sync::Arc;
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::Object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use crate::raytracer::object::db::{get_object, insert_sentinel, replace_sentinel};

pub struct Group {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub child_ids: Vec<usize>,
}

impl Group {
    pub fn new() -> Group {
        Group {
            id: insert_sentinel(),
            parent_id: None,
            transform: Matrix::identity(4),
            child_ids: Vec::new(),
        }
    }

    pub fn add_child(&mut self, mut object: Arc<dyn Object + Send>) -> usize {
        Arc::get_mut(&mut object).unwrap().set_parent_id(self.id);
        let child_id = object.get_id();
        replace_sentinel(child_id, object);
        self.child_ids.push(child_id);
        child_id
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = Vec::new();
        for child_id in &self.child_ids {
            let child = get_object(*child_id);
            let mut child_xs = child.intersect(ray);
            xs.append(&mut child_xs);
        }
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        xs
    }

    pub fn local_normal_at(&self) -> Tuple {
        panic!("Groups do not have normals")
    }

    pub fn world_to_object(object_id: usize, world_point: &Tuple) -> Tuple {
        let object = get_object(object_id);
        let mut point = world_point.clone();
        if let Some(parent_id) = object.get_parent_id() {
            point = Group::world_to_object(parent_id, &point);
        }
        object.get_transform().inverse().multiply_tuple(&point)
    }
}

impl Object for Group {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        self.local_intersect(&trans_ray)
    }

    fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let local_point = self.transform.inverse().multiply_tuple(world_point);
        let local_normal = self.local_normal_at();
        let mut world_normal = self.transform.inverse().transpose().multiply_tuple(&local_normal);
        world_normal.w = 0.0;
        world_normal.normalize()
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
        panic!("Groups do not have materials")
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::raytracer::light::Light;
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

        let p = Group::world_to_object(s_id, &Tuple::point(-2.0, 0.0, -10.0));
        assert_eq!(p, Tuple::point(0.0, 0.0, -1.0));
    }
}

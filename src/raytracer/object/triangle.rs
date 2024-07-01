use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_next_id;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

#[derive(Debug, PartialEq)]
pub struct Triangle {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub material: Material,
    pub p1: Tuple,
    pub p2: Tuple,
    pub p3: Tuple,
    pub e1: Tuple,
    pub e2: Tuple,
    pub normal: Tuple,
}

impl Triangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple) -> Triangle {
        let e1 = p2.subtract(&p1);
        let e2 = p3.subtract(&p1);
        let normal = e2.cross(&e1).normalize();
        Triangle {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
            p1,
            p2,
            p3,
            e1,
            e2,
            normal,
        }
    }

    pub fn local_normal_at(&self, _local_point: &Tuple) -> Tuple {
        self.normal
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let dir_cross_e2 = ray.direction.cross(&self.e2);
        let det = self.e1.dot(&dir_cross_e2);
        if det.abs() < crate::EPSILON {
            return vec![];
        }

        let f = 1.0 / det;
        let p1_to_origin = ray.origin.subtract(&self.p1);
        let u = f * p1_to_origin.dot(&dir_cross_e2);
        if u < 0.0 || u > 1.0 {
            return vec![];
        }

        let origin_cross_e1 = p1_to_origin.cross(&self.e1);
        let v = f * ray.direction.dot(&origin_cross_e1);
        if v < 0.0 || (u + v) > 1.0 {
            return vec![];
        }

        let t = f * self.e2.dot(&origin_cross_e1);
        vec![Intersection { t, object: self.id }]
    }
}

impl Object for Triangle {
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
        &self.material
    }

    fn set_transform(&mut self, transform: Matrix) {
        self.transform = transform;
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn debug_string(&self) -> String {
        format!("Triangle: transform: {:?}, material: {:?}", self.transform, self.material)
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
        let min = Tuple::point(
            self.p1.x.min(self.p2.x.min(self.p3.x)),
            self.p1.y.min(self.p2.y.min(self.p3.y)),
            self.p1.z.min(self.p2.z.min(self.p3.z)));
        let max = Tuple::point(
            self.p1.x.max(self.p2.x.max(self.p3.x)),
            self.p1.y.max(self.p2.y.max(self.p3.y)),
            self.p1.z.max(self.p2.z.max(self.p3.z)));
        AABB { min, max }
    }
}


#[cfg(test)]
mod tests {
    use crate::raytracer::object::triangle::Triangle;
    use crate::raytracer::ray::Ray;
    use crate::tuple::Tuple;

    #[test]
   fn intersecting_a_ray_parallel_to_the_triangle() {
         let t = Triangle::new(
              Tuple::point(0.0, 1.0, 0.0),
              Tuple::point(-1.0, 0.0, 0.0),
              Tuple::point(1.0, 0.0, 0.0),
         );
         let r = Ray::new(
              Tuple::point(0.0, -1.0, -2.0),
              Tuple::vector(0.0, 1.0, 0.0),
         );
         let xs = t.local_intersect(&r);
         assert_eq!(xs.len(), 0);
   }

    #[test]
    fn a_ray_misses_the_p1_p3_edge() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(1.0, 1.0, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn a_ray_misses_the_p1_p2_edge() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(-1.0, 1.0, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn a_ray_misses_the_p2_p3_edge() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(0.0, -1.0, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn a_ray_strikes_a_triangle() {
        let t = Triangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(0.0, 0.5, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 2.0);
    }
}
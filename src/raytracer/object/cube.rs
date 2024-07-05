use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use crate::EPSILON;
use crate::raytracer::object::db::get_next_id;

pub struct Cube {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub material: Material,
}

impl Cube {
    pub fn new() -> Cube {
        Cube {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let (xtmin, xtmax) = Cube::check_axis(ray.origin.x, ray.direction.x);
        let (ytmin, ytmax) = Cube::check_axis(ray.origin.y, ray.direction.y);
        let (ztmin, ztmax) = Cube::check_axis(ray.origin.z, ray.direction.z);
        let tmin = xtmin.max(ytmin).max(ztmin);
        let tmax = xtmax.min(ytmax).min(ztmax);
        if tmin > tmax {
            vec![]
        } else {
            vec![Intersection::new(tmin, self.id, 0.0, 0.0),
                 Intersection::new(tmax, self.id, 0.0, 0.0)]
        }
    }

    fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
        let tmin_numerator = -1.0 - origin;
        let tmax_numerator = 1.0 - origin;
        let (tmin, tmax) = if direction.abs() >= EPSILON {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (tmin_numerator * f64::INFINITY, tmax_numerator * f64::INFINITY)
        };
        if tmin > tmax {
            (tmax, tmin)
        } else {
            (tmin, tmax)
        }
    }

    pub fn local_normal_at(&self, local_point: &Tuple, _hit: &Intersection) -> Tuple {
        let maxc = local_point.x.abs().max(local_point.y.abs()).max(local_point.z.abs());
        if maxc == local_point.x.abs() {
            Tuple::vector(local_point.x, 0.0, 0.0)
        } else if maxc == local_point.y.abs() {
            Tuple::vector(0.0, local_point.y, 0.0)
        } else {
            Tuple::vector(0.0, 0.0, local_point.z)
        }
    }
}

impl Object for Cube {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        self.local_intersect(&trans_ray)
    }

    fn normal_at(&self, world_point: &Tuple, _hit: &Intersection) -> Tuple {
        let local_point = world_to_object(self.id, world_point);
        let local_normal = self.local_normal_at(&local_point, _hit);
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
        format!("Cube: transform: {:?}, material: {:?}", self.transform, self.material)
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
        let min = Tuple::point(-1.0, -1.0, -1.0);
        let max = Tuple::point(1.0, 1.0, 1.0);
        AABB { min, max }
    }
}

#[cfg(test)]
mod tests {
    use crate::raytracer::intersection::Intersection;
    use crate::raytracer::ray::Ray;
    use crate::tuple::Tuple;
    use super::Cube;

    #[test]
    fn test_check_axis() {
        let (tmin, tmax) = Cube::check_axis(5.0, 1.0);
        assert_eq!(tmin, -6.0);
        assert_eq!(tmax, -4.0);

        let (tmin, tmax) = Cube::check_axis(5.0, -1.0);
        assert_eq!(tmin, 4.0);
        assert_eq!(tmax, 6.0);
    }

    #[test]
    fn ray_intersects_a_cube() {
        let c = Cube::new();
        let origins = vec![
            Tuple::point(5.0, 0.5, 0.0),
            Tuple::point(-5.0, 0.5, 0.0),
            Tuple::point(0.5, 5.0, 0.0),
            Tuple::point(0.5, -5.0, 0.0),
            Tuple::point(0.5, 0.0, 5.0),
            Tuple::point(0.5, 0.0, -5.0),
            Tuple::point(0.0, 0.5, 0.0),
        ];
        let directions = vec![
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0),
        ];
        let t1s = vec![4.0, 4.0, 4.0, 4.0, 4.0, 4.0, -1.0];
        let t2s = vec![6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 1.0];
        for i in 0..7 {
            let r = Ray::new(origins[i], directions[i]);
            let xs = c.local_intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, t1s[i]);
            assert_eq!(xs[1].t, t2s[i]);
        }
    }

    #[test]
    fn ray_misses_a_cube() {
        let c = Cube::new();
        let origins = vec![
            Tuple::point(-2.0, 0.0, 0.0),
            Tuple::point(0.0, -2.0, 0.0),
            Tuple::point(0.0, 0.0, -2.0),
            Tuple::point(2.0, 0.0, 2.0),
            Tuple::point(0.0, 2.0, 2.0),
            Tuple::point(2.0, 2.0, 0.0),
        ];
        let directions = vec![
            Tuple::vector(0.2673, 0.5345, 0.8018),
            Tuple::vector(0.8018, 0.2673, 0.5345),
            Tuple::vector(0.5345, 0.8018, 0.2673),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
        ];
        for i in 0..6 {
            let r = Ray::new(origins[i], directions[i]);
            let xs = c.local_intersect(&r);
            assert_eq!(xs.len(), 0);
        }
    }

    #[test]
    fn normal_on_the_surface_of_a_cube() {
        let c = Cube::new();
        let points = vec![
            Tuple::point(1.0, 0.5, -0.8),
            Tuple::point(-1.0, -0.2, 0.9),
            Tuple::point(-0.4, 1.0, -0.1),
            Tuple::point(0.3, -1.0, -0.7),
            Tuple::point(-0.6, 0.3, 1.0),
            Tuple::point(0.4, 0.4, -1.0),
            Tuple::point(1.0, 1.0, 1.0),
            Tuple::point(-1.0, -1.0, -1.0),
        ];
        let normals = vec![
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
        ];

        for i in 0..8 {
            let p = points[i];
            let n = normals[i];
            let normal = c.local_normal_at(&p, &Intersection::new(0.0, 0, 0.0, 0.0));
            assert_eq!(normal, n);
        }
    }
}
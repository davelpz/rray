use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_next_id;
use crate::raytracer::object::{AABB, Object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

/// Represents a geometric triangle in a 3D scene.
///
/// A triangle is defined by three points in space (`p1`, `p2`, `p3`), which determine its vertices.
/// It also includes edge vectors (`e1`, `e2`) calculated from these points, and a normal vector
/// (`normal`) that is perpendicular to the triangle's surface. This structure is used within the
/// ray tracing system to represent flat surfaces and is capable of calculating intersections with rays,
/// determining surface normals at points of intersection, and supporting transformations and material
/// properties.
///
/// # Fields
///
/// * `id`: A unique identifier for the triangle, used for tracking and managing objects within the scene.
/// * `parent_id`: An optional identifier for a parent object, allowing for hierarchical object composition.
///   This can be `None` if the triangle does not have a parent.
/// * `transform`: A transformation matrix that applies translation, rotation, and scaling to the triangle,
///   positioning it within the 3D scene.
/// * `material`: The material properties of the triangle, defining how it interacts with light and shadows
///   within the scene.
/// * `p1`, `p2`, `p3`: The vertices of the triangle, represented as points in space.
/// * `e1`, `e2`: Edge vectors of the triangle, calculated from the vertices.
/// * `normal`: The normal vector of the triangle's plane, calculated from the cross product of `e2` and `e1`.
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

/// Implementation of `Triangle` functionalities.
///
/// This implementation provides the necessary methods to create and manipulate triangle objects within a 3D scene.
/// It includes methods for constructing a new triangle, calculating intersections with rays, determining the normal
/// at a point of intersection, and managing the triangle's transformation and material properties. By implementing
/// the `Object` trait, `Triangle` instances can be treated as first-class objects within the ray tracing system,
/// allowing them to interact with rays, lights, and other objects in a consistent manner.
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
}

impl Object for Triangle {
    fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
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
        vec![Intersection { t, object: self.id, u, v}]
    }

    fn local_normal_at(&self, _local_point: &Tuple, _hit: &Intersection) -> Tuple {
        self.normal
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

    fn includes(&self, object_id: usize) -> bool {
        self.id == object_id
    }

    fn uv_mapping(&self, point: &Tuple) -> (f64, f64) {
        let v0 = self.p2.subtract(&self.p1);
        let v1 = self.p3.subtract(&self.p1);
        let v2 = point.subtract(&self.p1);

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);
        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);

        let denom = d00 * d11 - d01 * d01;

        let lambda1 = (d11 * d20 - d01 * d21) / denom;
        let lambda2 = (d00 * d21 - d01 * d20) / denom;
        let _lambda0 = 1.0 - lambda1 - lambda2;

        // Implicit UV mapping: p0 -> (0, 0), p1 -> (1, 0), p2 -> (0, 1)
        let u = lambda1; // Interpolate u using barycentric coordinates
        let v = lambda2; // Interpolate v using barycentric coordinates

        (u, v)
    }
}


#[cfg(test)]
mod tests {
    use crate::raytracer::object::Object;
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
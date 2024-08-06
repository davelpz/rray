use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_next_id;
use crate::raytracer::object::{AABB, Object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

/// Represents a smooth triangle for use in a ray tracing context.
///
/// A smooth triangle is a triangle that allows for smooth shading across its surface by interpolating
/// normal vectors at its vertices. This is useful for creating more realistic lighting effects, as it
/// avoids the faceted look of flat shading.
///
/// # Fields
///
/// * `id` - A unique identifier for the smooth triangle, used for tracking and managing objects within the scene.
/// * `parent_id` - An optional identifier for a parent object, allowing for hierarchical object composition.
///   This can be `None` if the smooth triangle does not have a parent.
/// * `transform` - A transformation matrix that applies translation, rotation, and scaling to the smooth triangle,
///   positioning it within the 3D scene.
/// * `material` - The material properties of the smooth triangle, defining how it interacts with light and shadows
///   within the scene.
/// * `p1`, `p2`, `p3` - The vertices of the triangle, represented as points in space.
/// * `n1`, `n2`, `n3` - The normal vectors at each of the triangle's vertices, used for smooth shading.
/// * `e1`, `e2` - Edge vectors of the triangle, calculated as `p2 - p1` and `p3 - p1` respectively.
/// * `normal` - The normal vector of the triangle's plane, calculated from the cross product of `e2` and `e1`.
#[derive(Debug, PartialEq)]
pub struct SmoothTriangle {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub material: Material,
    pub p1: Tuple,
    pub p2: Tuple,
    pub p3: Tuple,
    pub n1: Tuple,
    pub n2: Tuple,
    pub n3: Tuple,
    pub e1: Tuple,
    pub e2: Tuple,
    pub normal: Tuple,
}

/// Implementation of `SmoothTriangle` functionalities.
///
/// Provides methods for creating a new `SmoothTriangle`, calculating intersections with rays,
/// determining the normal at a point of intersection, and managing the triangle's transformation
/// and material properties. It implements the `Object` trait, enabling `SmoothTriangle` instances
/// to be treated as first-class objects within the ray tracing system.
impl SmoothTriangle {
    pub fn new(p1: Tuple, p2: Tuple, p3: Tuple, n1: Tuple, n2: Tuple, n3: Tuple) -> SmoothTriangle {
        let e1 = p2.subtract(&p1);
        let e2 = p3.subtract(&p1);
        let normal = e2.cross(&e1).normalize();
        SmoothTriangle {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
            p1,
            p2,
            p3,
            n1,
            n2,
            n3,
            e1,
            e2,
            normal,
        }
    }
}

impl Object for SmoothTriangle {
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

    fn local_normal_at(&self, _local_point: &Tuple, hit: &Intersection) -> Tuple {
        self.n2 * hit.u + self.n3 * hit.v + self.n1 * (1.0 - hit.u - hit.v)
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
    use std::sync::Arc;
    use crate::color::Color;
    use crate::raytracer::intersection::Intersection;
    use crate::raytracer::light::Light;
    use crate::raytracer::object::db::get_object;
    use crate::raytracer::object::smooth_triangle::SmoothTriangle;
    use crate::raytracer::ray::Ray;
    use crate::tuple::Tuple;
    use crate::raytracer::object::Object;

    #[test]
   fn intersecting_a_ray_parallel_to_the_triangle() {
         let t = SmoothTriangle::new(
              Tuple::point(0.0, 1.0, 0.0),
              Tuple::point(-1.0, 0.0, 0.0),
              Tuple::point(1.0, 0.0, 0.0),
                Tuple::vector(0.0, 1.0, 0.0),
                Tuple::vector(-1.0, 0.0, 0.0),
                Tuple::vector(1.0, 0.0, 0.0),
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
        let t = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
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
        let t = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
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
        let t = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
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
        let t = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
        );
        let r = Ray::new(
            Tuple::point(0.0, 0.5, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = t.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0].t, 2.0);
    }

    #[test]
    fn intersection_with_a_smooth_triangle_stores_uv() {
        let r = Ray::new(
            Tuple::point(-0.2, 0.3, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let tri = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
        );
        let xs = tri.local_intersect(&r);
        assert_eq!(xs.len(), 1);
        assert!((xs[0].u - 0.45).abs() < crate::EPSILON);
        assert!((xs[0].v - 0.25).abs() < crate::EPSILON);
    }

    #[test]
    fn smooth_triangle_uses_u_v_to_interpolate_the_normal() {
        let mut scene = crate::raytracer::scene::Scene::new();
        scene.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let tri = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
        );
        let tri_id = scene.add_object(Arc::new(tri));
        let tri = get_object(tri_id);
        let i = Intersection { t: 0.0, object: tri_id, u: 0.45, v: 0.25 };
        let n = tri.normal_at(&Tuple::point(0.0, 0.0, 0.0), &i);
        assert_eq!(n, Tuple::vector(-0.5547, 0.83205, 0.0));
    }

    #[test]
    fn preparing_the_normal_on_a_smooth_triangle() {
        let mut scene = crate::raytracer::scene::Scene::new();
        scene.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let tri = SmoothTriangle::new(
            Tuple::point(0.0, 1.0, 0.0),
            Tuple::point(-1.0, 0.0, 0.0),
            Tuple::point(1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
        );
        let tri_id = scene.add_object(Arc::new(tri));
        let i = Intersection { t: 0.0, object: tri_id, u: 0.45, v: 0.25 };
        let r = Ray::new(
            Tuple::point(-0.2, 0.3, -2.0),
            Tuple::vector(0.0, 0.0, 1.0),
        );
        let xs = vec![i];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert_eq!(comps.normalv, Tuple::vector(-0.5547, 0.83205, 0.0));
    }
}
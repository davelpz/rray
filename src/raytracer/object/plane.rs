use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::EPSILON;
use crate::raytracer::object::db::get_next_id;
use crate::tuple::Tuple;

/// Represents an infinite plane in a 3D scene.
///
/// This struct is part of a ray tracing system, defining a plane that extends infinitely in two dimensions.
/// It is characterized by its position, orientation (through a transformation matrix), and material properties.
///
/// # Fields
///
/// * `id` - A unique identifier for the plane, used for tracking and managing objects within the scene.
/// * `parent_id` - An optional identifier for a parent object, allowing for hierarchical object composition.
///   This can be `None` if the plane does not have a parent.
/// * `transform` - A transformation matrix that applies translation, rotation, and scaling to the plane,
///   positioning it within the 3D scene.
/// * `material` - The material properties of the plane, defining how it interacts with light and shadows
///   within the scene.
pub struct Plane {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub material: Material,
}

/// Implementation of `Plane` functionalities.
///
/// This implementation provides the necessary methods for a `Plane` object within the ray tracing system.
/// It includes methods for creating a new plane, calculating intersections with rays in the plane's local space,
/// determining the normal at a point of intersection, and managing the plane's transformation and material properties.
/// Additionally, it implements the `Object` trait, enabling `Plane` instances to be treated as first-class objects
/// within the ray tracing system.
impl Plane {
    pub fn new() -> Plane {
        Plane {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        if ray.direction.y.abs() < EPSILON {
            vec![]
        } else {
            let t = -ray.origin.y / ray.direction.y;
            vec![Intersection { t, object: self.id, u: 0.0, v: 0.0}]
        }
    }
    pub fn local_normal_at(&self, _local_point: &Tuple, _hit: &Intersection) -> Tuple {
        Tuple::vector(0.0, 1.0, 0.0)
    }
}

impl Object for Plane {
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
        format!("Plane: transform: {:?}, material: {:?}", self.transform, self.material)
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
        let min = Tuple::point(f64::NEG_INFINITY, 0.0, f64::NEG_INFINITY);
        let max = Tuple::point(f64::INFINITY, 0.0, f64::INFINITY);
        AABB::new(min, max)
    }

    fn includes(&self, object_id: usize) -> bool {
        self.id == object_id
    }

}
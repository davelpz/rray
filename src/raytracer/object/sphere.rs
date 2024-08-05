use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_next_id;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

/// Represents a sphere in a 3D scene.
///
/// A sphere is defined by its center, radius, and material properties. In this implementation,
/// the sphere's center is assumed to be at the origin of its local coordinate system, and its
/// radius is implicitly 1 unit (but can be scaled using the transform matrix). Material properties
/// and transformations can be applied to simulate various effects and positions within the scene.
///
/// # Fields
///
/// * `id` - A unique identifier for the sphere, used for tracking and managing objects within the scene.
/// * `parent_id` - An optional identifier for a parent object, allowing for hierarchical object composition.
///   This can be `None` if the sphere does not have a parent.
/// * `transform` - A transformation matrix that applies translation, rotation, and scaling to the sphere,
///   positioning it within the 3D scene.
/// * `material` - The material properties of the sphere, defining how it interacts with light and shadows
///   within the scene.
pub struct Sphere {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub material: Material,
}

/// Implementation of `Sphere` functionalities.
///
/// Provides methods for creating new `Sphere` instances, including a default sphere and a glass sphere,
/// calculating intersections with rays, determining the normal at a point of intersection, and managing
/// the sphere's transformation and material properties. It implements the `Object` trait, enabling
/// `Sphere` instances to be treated as first-class objects within the ray tracing system.
impl Sphere {
    pub fn new() -> Sphere {
        Sphere {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn glass_sphere() -> Sphere {
        let mut m = Material::default();
        m.transparency = 1.0;
        m.refractive_index = 1.5;
        Sphere {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: m,
        }
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let sphere_to_ray = ray.origin.subtract(&ORIGIN);
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant: f64 = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            vec![]
        } else {
            let t1: f64 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2: f64 = (-b + discriminant.sqrt()) / (2.0 * a);
            vec![Intersection { t: t1, object: self.id, u: 0.0, v: 0.0},
                 Intersection { t: t2, object: self.id, u: 0.0, v: 0.0}]
        }
    }

    pub fn local_normal_at(&self, local_point: &Tuple, _hit: &Intersection) -> Tuple {
        local_point.subtract(&Tuple::point(0.0, 0.0, 0.0))
    }
}

const ORIGIN: Tuple = Tuple { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

impl Object for Sphere {
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
        format!("Sphere: transform: {:?}, material: {:?}", self.transform, self.material)
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

    fn includes(&self, object_id: usize) -> bool {
        self.id == object_id
    }

    fn uv_mapping(&self, point: &Tuple) -> (f64, f64) {
        let theta = point.z.atan2(point.x);
        let phi = (point.y / point.length_squared().sqrt()).acos();
        let u = (theta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
        let v = 1.0 - (phi / std::f64::consts::PI);
        (u, v)
    }
}
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use crate::EPSILON;
use crate::raytracer::object::db::get_next_id;

/// Represents a cylinder in a ray tracing context.
///
/// This struct defines a cylinder by its unique identifier, optional parent identifier,
/// minimum and maximum extents along the y-axis, a flag indicating whether the cylinder
/// is closed at both ends, its transformation matrix, and material properties. It is used
/// within the ray tracing system to represent cylinders as objects that can interact with rays.
///
/// # Fields
///
/// * `id` - A unique identifier for the cylinder, used for tracking objects within the scene.
/// * `parent_id` - An optional identifier for a parent object, allowing for hierarchical
///   object composition. This can be `None` if the cylinder does not have a parent.
/// * `minimum` - The minimum extent of the cylinder along the y-axis.
/// * `maximum` - The maximum extent of the cylinder along the y-axis.
/// * `closed` - A boolean flag indicating whether the cylinder is closed at both ends.
/// * `transform` - A transformation matrix that applies translation, rotation, and scaling
///   to the cylinder, positioning it within the 3D scene.
/// * `material` - The material properties of the cylinder, defining how it interacts with light
///   and shadows within the scene.
pub struct Cylinder {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transform: Matrix,
    pub material: Material,
}

/// Implementation of `Cylinder` functionalities.
///
/// This implementation provides the necessary methods to integrate `Cylinder` objects into the ray tracing system,
/// allowing them to be treated as first-class objects within the scene. It includes methods for calculating
/// intersections with rays, determining surface normals at points of intersection, managing transformations
/// and material properties of the cylinder, and handling end caps for closed cylinders.
impl Cylinder {
    pub fn new(minimum: f64, maximum: f64, closed: bool) -> Cylinder {
        Cylinder {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
            minimum,
            maximum,
            closed,
        }
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = vec![];
        let a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
        if a.abs() > EPSILON {
            let b = 2.0 * ray.origin.x * ray.direction.x + 2.0 * ray.origin.z * ray.direction.z;
            let c = ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - 1.0;
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                return vec![];
            }
            let mut t0 = (-b - discriminant.sqrt()) / (2.0 * a);
            let mut t1 = (-b + discriminant.sqrt()) / (2.0 * a);
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
            }

            let y0 = ray.origin.y + t0 * ray.direction.y;
            if self.minimum < y0 && y0 < self.maximum {
                xs.push(Intersection::new(t0, self.id, 0.0, 0.0));
            }
            let y1 = ray.origin.y + t1 * ray.direction.y;
            if self.minimum < y1 && y1 < self.maximum {
                xs.push(Intersection::new(t1, self.id, 0.0, 0.0));
            }
        }

        self.intersect_caps(ray).iter().for_each(|i| xs.push(i.clone()));

        xs
    }

    // Check if the intersection at `t` is within the radius of the cylinder
    // at the end caps
    fn check_cap(ray: &Ray, t: f64) -> bool {
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        (x * x + z * z) <= 1.0
    }

    fn intersect_caps(&self, ray: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = vec![];

        // Caps only matter if the cylinder is closed, and might be
        // intersected by the ray
        if !self.closed || ray.direction.y.abs() < EPSILON {
            return xs;
        }

        // Check for an intersection with the lower end cap by intersecting
        // the ray with the plane at y = minimum
        let t = (self.minimum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t) {
            xs.push(Intersection::new(t, self.id, 0.0, 0.0));
        }

        // Check for an intersection with the upper end cap by intersecting
        // the ray with the plane at y = maximum
        let t = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t) {
            xs.push(Intersection::new(t, self.id, 0.0, 0.0));
        }

        xs
    }

    pub fn local_normal_at(&self, local_point: &Tuple, _hit: &Intersection) -> Tuple {
        // Compute the square of the distance from the y-axis
        let dist = local_point.x * local_point.x + local_point.z * local_point.z;

        if dist < 1.0 && local_point.y >= self.maximum - EPSILON {
            Tuple::vector(0.0, 1.0, 0.0)
        } else if dist < 1.0 && local_point.y <= self.minimum + EPSILON {
            Tuple::vector(0.0, -1.0, 0.0)
        } else {
            Tuple::vector(local_point.x, 0.0, local_point.z)
        }
    }
}

impl Object for Cylinder {
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
        format!("Cylinder: transform: {:?}, material: {:?}", self.transform, self.material)
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
        AABB::new(
            Tuple::point(-1.0, self.minimum, -1.0),
            Tuple::point(1.0, self.maximum, 1.0),
        )
    }

    fn includes(&self, object_id: usize) -> bool {
        self.id == object_id
    }

    fn uv_mapping(&self, point: &Tuple) -> (f64, f64) {
        if self.closed && (point.y <= self.minimum || point.y >= self.maximum) {
            let u = (point.x + 1.0) / 2.0;
            let v = (point.z + 1.0) / 2.0;
            (u, v)
        } else {
            let theta = point.z.atan2(point.x);
            let u = (theta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);

            // Wrap the v coordinate to repeat this texture along the y-axis
            let v = point.y % 1.0;
            let v = if v < 0.0 { 1.0 + v } else { v };

            (u, v)
        }
    }
}
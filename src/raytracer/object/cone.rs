use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use crate::EPSILON;
use crate::raytracer::object::db::get_next_id;

/// Represents a cone in a 3D ray tracing context.
///
/// A cone is defined by its minimum and maximum extents along the y-axis, which determine its height,
/// and whether its ends are closed or open. It also includes a transformation matrix to position,
/// rotate, and scale the cone in the 3D world, as well as a material that defines its surface appearance.
///
/// # Fields
///
/// * `id` - A unique identifier for the cone, used for tracking objects in the scene.
/// * `parent_id` - An optional identifier for a parent object, allowing for hierarchical object composition.
/// * `minimum` - The minimum y-coordinate of the cone, defining the lower bound of its height.
/// * `maximum` - The maximum y-coordinate of the cone, defining the upper bound of its height.
/// * `closed` - A boolean indicating whether the ends of the cone are closed (true) or open (false).
/// * `transform` - A transformation matrix applied to the cone for positioning, rotation, and scaling.
/// * `material` - The material of the cone, defining how it interacts with light in the scene.
pub struct Cone {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transform: Matrix,
    pub material: Material,
}

/// Implementation of the `Cone` struct, providing methods for creating cones,
/// calculating intersections, normals, and managing transformations and materials.
///
/// This implementation includes methods for:
/// - Creating a new cone with specified minimum and maximum y-coordinates, and whether it is closed.
/// - Calculating intersections with rays, taking into account the cone's geometry and transformations.
/// - Determining the normal at a given point on the cone.
/// - Managing the cone's transformation matrix and material properties.
///
/// The `Cone` struct is part of a ray tracing system, designed to represent conical shapes within a 3D scene.
/// It extends the generic `Object` trait, allowing it to interact seamlessly with the ray tracing engine,
/// including support for hierarchical scene graphs through parent IDs.
impl Cone {
    pub fn new(minimum: f64, maximum: f64, closed: bool) -> Cone {
        Cone {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
            minimum,
            maximum,
            closed,
        }
    }
    pub fn local_intersect(&self, trans_ray: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = vec![];
        let minimum = self.minimum;
        let maximum = self.maximum;
        let a = trans_ray.direction.x * trans_ray.direction.x - trans_ray.direction.y * trans_ray.direction.y + trans_ray.direction.z * trans_ray.direction.z;
        let b = 2.0 * trans_ray.origin.x * trans_ray.direction.x - 2.0 * trans_ray.origin.y * trans_ray.direction.y + 2.0 * trans_ray.origin.z * trans_ray.direction.z;

        if a.abs() < EPSILON && b.abs() < EPSILON {
            self.intersect_caps(&trans_ray).iter().for_each(|i| xs.push(i.clone()));
            return xs;
        }

        let c = trans_ray.origin.x * trans_ray.origin.x - trans_ray.origin.y * trans_ray.origin.y + trans_ray.origin.z * trans_ray.origin.z;

        if a.abs() < EPSILON {
            let t = -c / (2.0 * b);
            let y = trans_ray.origin.y + t * trans_ray.direction.y;
            if minimum < y && y < maximum {
                xs.push(Intersection::new(t, self.id, 0.0, 0.0));
                return xs;
            }
        }

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return vec![];
        }

        let mut t0 = (-b - discriminant.sqrt()) / (2.0 * a);
        let mut t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let y0 = trans_ray.origin.y + t0 * trans_ray.direction.y;
        if minimum < y0 && y0 < maximum {
            xs.push(Intersection::new(t0, self.id, 0.0, 0.0));
        }
        let y1 = trans_ray.origin.y + t1 * trans_ray.direction.y;
        if minimum < y1 && y1 < maximum {
            xs.push(Intersection::new(t1, self.id, 0.0, 0.0));
        }

        self.intersect_caps(&trans_ray).iter().for_each(|i| xs.push(i.clone()));

        xs
    }

    // Check if the intersection at `t` is within the radius of the cone
// at the end caps
    fn check_cap(ray: &Ray, t: f64) -> bool {
        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;
        let z = ray.origin.z + t * ray.direction.z;
        (x * x + z * z) <= y * y
    }

    fn intersect_caps(&self, ray: &Ray) -> Vec<Intersection> {
        let minimum = self.minimum;
        let maximum = self.maximum;
        let closed = self.closed;
        let mut xs: Vec<Intersection> = vec![];

        // Caps only matter if the cone is closed, and might be
        // intersected by the ray
        if !closed || ray.direction.y.abs() < EPSILON {
            return xs;
        }

        // Check for an intersection with the lower end cap by intersecting
        // the ray with the plane at y = minimum
        let t = (minimum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, t) {
            xs.push(Intersection::new(t, self.id, 0.0, 0.0));
        }

        // Check for an intersection with the upper end cap by intersecting
        // the ray with the plane at y = maximum
        let t = (maximum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, t) {
            xs.push(Intersection::new(t, self.id, 0.0, 0.0));
        }

        xs
    }

    pub fn local_normal_at(&self, local_point: &Tuple, _hit: &Intersection) -> Tuple {
        let minimum = self.minimum;
        let maximum = self.maximum;

        // Compute the square of the distance from the y-axis
        let dist = local_point.x * local_point.x + local_point.z * local_point.z;

        if dist < 1.0 && local_point.y >= maximum - EPSILON {
            Tuple::vector(0.0, 1.0, 0.0)
        } else if dist < 1.0 && local_point.y <= minimum + EPSILON {
            Tuple::vector(0.0, -1.0, 0.0)
        } else {
            let mut y = dist.sqrt();
            if local_point.y > 0.0 {
                y = -y;
            }

            Tuple::vector(local_point.x, y, local_point.z)
        }
    }
}

/// Provides the `Object` trait implementations for `Cone`.
///
/// This implementation allows a `Cone` to interact with the ray tracing system by defining how rays intersect with it,
/// how to calculate the normal at a point on the cone, and how to manage its transformation and material properties.
/// It integrates the cone into the broader system, allowing it to be treated as any other object in the scene.
///
/// # Methods
///
/// - `intersect`: Calculates the intersections of a ray with the cone, considering the cone's transformation.
/// - `normal_at`: Computes the normal vector at a given point on the cone, taking into account its transformation.
/// - `get_transform`: Returns the transformation matrix of the cone.
/// - `get_material`: Returns the material of the cone.
/// - `set_transform`: Sets the transformation matrix of the cone.
/// - `set_material`: Sets the material of the cone.
/// - `debug_string`: Returns a string representation of the cone for debugging purposes.
/// - `get_id`: Returns the unique identifier of the cone.
/// - `get_parent_id`: Returns the optional parent identifier of the cone.
/// - `set_parent_id`: Sets the parent identifier of the cone.
/// - `get_aabb`: Calculates the axis-aligned bounding box (AABB) of the cone.
/// - `includes`: Checks if the given object identifier matches the cone's identifier.
impl Object for Cone {
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
        format!("Cone: transform: {:?}, material: {:?}", self.transform, self.material)
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
        let limit = self.minimum.abs().max(self.maximum.abs());
        let min = Tuple::point(-limit, self.minimum, -limit);
        let max = Tuple::point(limit, self.maximum, limit);
        AABB { min, max }
    }

    fn includes(&self, object_id: usize) -> bool {
        self.id == object_id
    }

    fn uv_mapping(&self, point: &Tuple) -> (f64, f64) {
        let y_min_dist = (point.y - self.minimum).abs();
        let y_max_dist = (point.y - self.maximum).abs();

        if self.closed && (y_min_dist <= EPSILON || y_max_dist <= EPSILON) {
            // Point on the cap
            let radius = point.y.abs();
            let u = (point.x / radius + 1.0) / 2.0;
            let v = (point.z / radius + 1.0) / 2.0;
            (u, v)
        } else {
            // Calculate the angle theta around the y-axis
            let theta = (point.z.atan2(point.x) + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);

            // Point on the conical surface
            let height_range = self.maximum - self.minimum;
            let normalized_y = (point.y - self.minimum) / height_range;

            // Normalize u for better texture mapping
            let u = normalized_y;

            (u, theta)
        }
    }
}
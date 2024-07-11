pub(crate) mod sphere;
pub(crate) mod plane;
pub(crate) mod cone;
pub(crate) mod cube;
pub(crate) mod cylinder;
pub(crate) mod db;
pub(crate) mod group;
pub(crate) mod triangle;
pub(crate) mod smooth_triangle;
pub(crate) mod csg;
pub(crate) mod torus;

use std::fmt::{Debug, Formatter};
use crate::EPSILON;
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

/// Represents a generic object in the ray tracing scene.
///
/// This trait defines a common interface for all objects that can be placed in a ray tracing scene.
/// It includes methods for calculating intersections with rays, determining surface normals at points
/// of intersection, and managing object transformations and materials. Additionally, it provides
/// functionality for debugging, hierarchical scene graph management, and bounding volume hierarchy
/// optimizations.
///
/// # Methods
///
/// * `intersect` - Calculates the intersections of a ray with the object, returning a list of intersection points.
/// * `normal_at` - Computes the normal vector at a given point on the object's surface, useful for shading calculations.
/// * `get_transform` - Retrieves the object's transformation matrix.
/// * `get_material` - Retrieves the material properties of the object.
/// * `set_transform` - Sets the object's transformation matrix.
/// * `set_material` - Assigns new material properties to the object.
/// * `debug_string` - Generates a string representation of the object for debugging purposes.
/// * `get_id` - Returns a unique identifier for the object.
/// * `get_parent_id` - Returns the identifier of the object's parent in a scene graph, if any.
/// * `set_parent_id` - Sets the identifier of the object's parent in a scene graph.
/// * `get_aabb` - Computes the axis-aligned bounding box (AABB) of the object for spatial partitioning optimizations.
/// * `includes` - Checks if the object includes another object by ID, useful for CSG operations and scene graph management.
pub trait Object: Sync + Send {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection>;
    fn normal_at(&self, point: &Tuple, hit: &Intersection) -> Tuple;
    fn get_transform(&self) -> &Matrix;
    fn get_material(&self) -> &Material;
    fn set_transform(&mut self, transform: Matrix);
    fn set_material(&mut self, material: Material);
    fn debug_string(&self) -> String;
    fn get_id(&self) -> usize;
    fn get_parent_id(&self) -> Option<usize>;
    fn set_parent_id(&mut self, id: usize);
    fn get_aabb(&self) -> AABB;
    fn includes(&self, object_id: usize) -> bool;
}

impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl Debug for dyn Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_string())
    }
}

/// Transforms a point from world space to object space.
///
/// This function recursively transforms a point from the world coordinate system to the object's
/// local coordinate system. It accounts for the object's transformations and those of its parent
/// objects in the scene graph hierarchy, if any. This is particularly useful for ray tracing calculations
/// where interactions with objects are often computed in the object's local space for simplicity.
///
/// # Arguments
///
/// * `object_id` - The unique identifier of the object to which the point will be transformed.
/// * `world_point` - A reference to the `Tuple` representing the point in world space.
///
/// # Returns
///
/// Returns the transformed point in the object's local coordinate system as a `Tuple`.
pub fn world_to_object(object_id: usize, world_point: &Tuple) -> Tuple {
    let object = get_object(object_id);
    let mut point = world_point.clone();
    if let Some(parent_id) = object.get_parent_id() {
        point = world_to_object(parent_id, &point);
    }
    object.get_transform().inverse().multiply_tuple(&point)
}

/// Transforms a normal vector from object space to world space.
///
/// This function takes a normal vector defined in the object's local coordinate system and transforms
/// it into the world coordinate system. This is essential for lighting calculations where the normal
/// needs to be in the same coordinate system as the light sources and the camera. The transformation
/// accounts for the object's transformations and recursively applies the transformations of its parent
/// objects in the scene graph hierarchy, if any. The function ensures the transformed normal vector
/// remains a unit vector and has its `w` component set to 0.0, maintaining its direction but not its
/// position.
///
/// # Arguments
///
/// * `object_id` - The unique identifier of the object whose normal vector is being transformed.
/// * `object_normal` - A reference to the `Tuple` representing the normal vector in the object's local space.
///
/// # Returns
///
/// Returns the transformed normal vector in the world coordinate system as a `Tuple`.
pub fn normal_to_world(object_id: usize, object_normal: &Tuple) -> Tuple {
    let object = get_object(object_id);
    let mut normal = object.get_transform().inverse().transpose().multiply_tuple(&object_normal);
    normal.w = 0.0;
    normal = normal.normalize();
    if let Some(parent_id) = object.get_parent_id() {
        normal = normal_to_world(parent_id, &normal);
    }
    normal
}

/// Represents an Axis-Aligned Bounding Box (AABB) in a ray tracing scene.
///
/// An AABB is a simple way to describe a volume in 3D space and is used for various optimizations,
/// such as reducing the number of intersection tests required for ray tracing. It is defined by two points:
/// the minimum point, which is the lower corner of the box, and the maximum point, which is the upper corner
/// of the box. These points are represented in world coordinates.
///
/// # Fields
///
/// * `min` - A `Tuple` representing the minimum point of the AABB in world coordinates.
/// * `max` - A `Tuple` representing the maximum point of the AABB in world coordinates.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Tuple,
    pub max: Tuple,
}

/// Implementation of the Axis-Aligned Bounding Box (AABB) for ray tracing optimization.
///
/// The `AABB` struct is crucial for optimizing ray tracing calculations by reducing the number
/// of intersection tests needed. This implementation provides methods for checking ray intersection
/// with the bounding box, adjusting the bounding box dimensions based on new points or another AABB,
/// and applying transformations to the bounding box to fit it around transformed objects.
///
/// # Methods
///
/// * `new` - Constructs a new `AABB` with specified minimum and maximum points.
/// * `check_axis` - Helper function to determine the intersection of a ray with a slab (a single axis of the AABB).
/// * `intersect` - Determines if a ray intersects with the bounding box.
/// * `adjust_min_max` - Adjusts the minimum and maximum points of the bounding box based on a new point.
/// * `adjust_aabb` - Expands the bounding box to include another `AABB`.
/// * `apply_transform` - Applies a transformation to the bounding box, recalculating its bounds to encompass
///   the transformed shape.
impl AABB {
    pub fn new(min: Tuple, max: Tuple) -> AABB {
        AABB { min, max }
    }

    fn check_axis(origin: f64, direction: f64, min: f64, max: f64) -> (f64, f64) {
        let tmin_numerator = min - origin;
        let tmax_numerator = max - origin;

        let (mut tmin, mut tmax) = if direction.abs() >= EPSILON {
            (tmin_numerator / direction, tmax_numerator / direction)
        } else {
            (tmin_numerator * f64::INFINITY, tmax_numerator * f64::INFINITY)
        };

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        (tmin, tmax)
    }

    /// Determines if a ray intersects with the Axis-Aligned Bounding Box (AABB).
    ///
    /// This method checks if a given ray intersects with the bounding box by comparing the intersection
    /// times (t values) for each axis (x, y, and z). It calculates these times based on the ray's origin
    /// and direction, as well as the minimum and maximum points of the AABB. If the intervals of t values
    /// for each axis overlap, then there is an intersection, indicating that the ray passes through the AABB.
    ///
    /// # Arguments
    ///
    /// * `r` - A reference to the `Ray` being tested for intersection with the AABB.
    ///
    /// # Returns
    ///
    /// Returns `true` if the ray intersects with the AABB, otherwise `false`.
    pub fn intersect(&self, r: &Ray) -> bool {
        let (tmin_x, tmax_x) = AABB::check_axis(r.origin.x, r.direction.x, self.min.x, self.max.x);
        let (tmin_y, tmax_y) = AABB::check_axis(r.origin.y, r.direction.y, self.min.y, self.max.y);
        let (tmin_z, tmax_z) = AABB::check_axis(r.origin.z, r.direction.z, self.min.z, self.max.z);

        let tmin = tmin_x.max(tmin_y.max(tmin_z));
        let tmax = tmax_x.min(tmax_y.min(tmax_z));

        tmin <= tmax
    }

    fn adjust_min_max(&mut self, x: f64, y:f64, z: f64) {
        self.min = Tuple::point(self.min.x.min(x),
                                self.min.y.min(y),
                                self.min.z.min(z));
        self.max = Tuple::point(self.max.x.max(x),
                                self.max.y.max(y),
                                self.max.z.max(z));
    }

    /// Expands the current Axis-Aligned Bounding Box (AABB) to include another AABB.
    ///
    /// This method adjusts the current AABB's minimum and maximum points to ensure it encompasses
    /// the entirety of another AABB passed as an argument. It does so by comparing and setting the
    /// minimum and maximum points along each axis (x, y, z) between the current AABB and the one provided.
    /// This is useful for constructing bounding volumes that contain multiple objects or for updating
    /// bounding volumes after transformations.
    ///
    /// # Arguments
    ///
    /// * `aabb` - A reference to another `AABB` that this AABB should encompass.
    pub fn adjust_aabb(&mut self, aabb: &AABB) {
        self.adjust_min_max(aabb.min.x, aabb.min.y, aabb.min.z);
        self.adjust_min_max(aabb.max.x, aabb.max.y, aabb.max.z);
    }

    /// Determines if a ray intersects with the Axis-Aligned Bounding Box (AABB).
    ///
    /// This method checks if a given ray intersects with the bounding box by comparing the intersection
    /// times (t values) for each axis (x, y, and z). It calculates these times based on the ray's origin
    /// and direction, as well as the minimum and maximum points of the AABB. If the intervals of t values
    /// for each axis overlap, then there is an intersection, indicating that the ray passes through the AABB.
    ///
    /// # Arguments
    ///
    /// * `r` - A reference to the `Ray` being tested for intersection with the AABB.
    ///
    /// # Returns
    ///
    /// Returns `true` if the ray intersects with the AABB, otherwise `false`.
    pub fn apply_transform(&self, transform: &Matrix) -> AABB {
          let corners = [
            Tuple::point(self.min.x, self.min.y, self.min.z),
            Tuple::point(self.min.x, self.min.y, self.max.z),
            Tuple::point(self.min.x, self.max.y, self.min.z),
            Tuple::point(self.min.x, self.max.y, self.max.z),
            Tuple::point(self.max.x, self.min.y, self.min.z),
            Tuple::point(self.max.x, self.min.y, self.max.z),
            Tuple::point(self.max.x, self.max.y, self.min.z),
            Tuple::point(self.max.x, self.max.y, self.max.z),
        ];

        let mut aabb = AABB::new(Tuple::point(f64::INFINITY, f64::INFINITY, f64::INFINITY), Tuple::point(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY));
        for corner in corners.iter() {
            let transformed = transform.multiply_tuple(corner);
            aabb.adjust_min_max(transformed.x, transformed.y, transformed.z);
        }
        aabb
    }

}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::raytracer::intersection::Intersection;
    use crate::raytracer::light::Light;
    use crate::raytracer::object::db::get_object;
    use crate::raytracer::object::Object;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::ray::Ray;
    use crate::raytracer::scene::Scene;
    use crate::tuple::Tuple;

    #[test]
    fn test_intersect() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Arc::new(Sphere::new());
        let s: Arc<dyn Object> = s;
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 6.0);

        let r = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        {
            let xs = s.intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, -1.0);
            assert_eq!(xs[1].t, 1.0);
        }

        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        {
            let xs = s.intersect(&r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, -6.0);
            assert_eq!(xs[1].t, -4.0);
            assert_eq!(xs[0].object, s.get_id());
            assert_eq!(xs[1].object, s.get_id());
        }
    }

    #[test]
    fn test_transform() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::scale(2.0, 2.0, 2.0);
        let s = Arc::new(s);
        let s: Arc<dyn Object> = s;
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::translate(5.0, 0.0, 0.0);
        let s = Arc::new(s);
        let s: Arc<dyn Object> = s;
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    fn normal_at_surface_point() {
        let mut scene = Scene::new();
        scene.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        let s = Arc::new(Sphere::new());
        let sid = scene.add_object(s);
        let s: Arc<dyn Object> = get_object(sid);
        let point = Tuple::point(1.0, 0.0, 0.0);
        let expected_normal = Tuple::vector(1.0, 0.0, 0.0);
        assert_eq!(s.normal_at(&point, &Intersection::new(0.0,sid,0.0,0.0)), expected_normal);

        let point = Tuple::point(0.0, 1.0, 0.0);
        let expected_normal = Tuple::vector(0.0, 1.0, 0.0);
        assert_eq!(s.normal_at(&point, &Intersection::new(0.0,sid,0.0,0.0)), expected_normal);

        let point = Tuple::point(0.0, 0.0, 1.0);
        let expected_normal = Tuple::vector(0.0, 0.0, 1.0);
        assert_eq!(s.normal_at(&point, &Intersection::new(0.0,sid,0.0,0.0)), expected_normal);
    }

    #[test]
    fn normal_at_non_axial_point() {
        let mut scene = Scene::new();
        scene.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        let s = Arc::new(Sphere::new());
        let sid = scene.add_object(s);
        let s: Arc<dyn Object> = get_object(sid);
        let sqrt_of_three_over_three = 3f64.sqrt() / 3.0;
        let point = Tuple::point(sqrt_of_three_over_three, sqrt_of_three_over_three, sqrt_of_three_over_three);
        let expected_normal = Tuple::vector(sqrt_of_three_over_three, sqrt_of_three_over_three, sqrt_of_three_over_three);
        assert_eq!(s.normal_at(&point, &Intersection::new(0.0,sid,0.0,0.0)), expected_normal);
        assert_eq!(s.normal_at(&point, &Intersection::new(0.0,sid,0.0,0.0)).magnitude(), 1.0);
    }
}
use std::str::FromStr;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::{add_object, get_next_id, get_object};
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

/// Represents the operation to be performed in a Constructive Solid Geometry (CSG) context.
///
/// CSG allows for the creation of complex surfaces or objects by using boolean operations
/// to combine simpler objects. This enum defines the types of operations that can be applied
/// to combine objects in a CSG system.
///
/// Variants:
/// - `Union`: Combines two objects into a single object that encompasses the volume of both.
/// - `Intersection`: Creates a new object from the overlapping volume of two objects.
/// - `Difference`: Subtracts the volume of the second object from the first, creating a new object.
pub enum CsgOperation {
    Union,
    Intersection,
    Difference,
}

/// Implements the `FromStr` trait for `CsgOperation`.
///
/// This implementation allows for the creation of `CsgOperation` instances from string slices,
/// enabling easy parsing and construction of CSG operations from textual representations.
/// It supports parsing the strings "union", "intersection", and "difference" into their
/// respective `CsgOperation` variants.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
/// use crate::CsgOperation;
///
/// let op = CsgOperation::from_str("union").unwrap();
/// assert_eq!(op, CsgOperation::Union);
///
/// let op = CsgOperation::from_str("intersection").unwrap();
/// assert_eq!(op, CsgOperation::Intersection);
///
/// let op = CsgOperation::from_str("difference").unwrap();
/// assert_eq!(op, CsgOperation::Difference);
/// ```
///
/// # Errors
///
/// Returns an error if the string does not match any of the CSG operation variants.
impl FromStr for CsgOperation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "union" => Ok(CsgOperation::Union),
            "intersection" => Ok(CsgOperation::Intersection),
            "difference" => Ok(CsgOperation::Difference),
            _ => Err(()),
        }
    }
}

/// Represents a Constructive Solid Geometry (CSG) node in a ray tracing context.
///
/// A CSG node allows for the combination of two objects using boolean operations
/// to create complex shapes. This struct holds the information necessary to perform
/// these operations, including the operation type, references to the child objects,
/// and a transformation matrix for positioning in the 3D world.
///
/// # Fields
///
/// * `id` - A unique identifier for the CSG node, used for tracking objects in the scene.
/// * `parent_id` - An optional identifier for a parent object, allowing for hierarchical object composition.
/// * `transform` - A transformation matrix applied to the CSG node for positioning, rotation, and scaling.
/// * `operation` - The boolean operation (`Union`, `Intersection`, `Difference`) to be performed on the child objects.
/// * `left` - The unique identifier of the left child object.
/// * `right` - The unique identifier of the right child object.
/// * `aabb_cache` - A cache for the axis-aligned bounding box (AABB) of the CSG node, wrapped in `RwLock` and `Arc` for thread safety.
pub struct Csg {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub operation: CsgOperation,
    pub left: usize,
    pub right: usize,
    aabb_cache: Arc<RwLock<Option<AABB>>>,  // Cache for the AABB wrapped in RwLock and Arc for thread safety
}

/// The `Csg` struct represents a Constructive Solid Geometry (CSG) node within a ray tracing context.
///
/// CSG is a modeling technique that uses boolean operations like union, intersection, and difference
/// to combine simpler objects into complex shapes. This struct encapsulates the logic for these operations,
/// managing the combination of child objects, their transformations, and the resulting shape's bounding box.
///
/// # Fields
///
/// - `id`: Unique identifier for the CSG node, used for scene management.
/// - `parent_id`: Optional identifier for a parent object, enabling hierarchical scene structures.
/// - `transform`: Transformation matrix for positioning, rotating, and scaling the CSG node.
/// - `operation`: Specifies the boolean operation (Union, Intersection, Difference) to apply.
/// - `left`: Identifier for the left child object.
/// - `right`: Identifier for the right child object.
/// - `aabb_cache`: Cached axis-aligned bounding box (AABB) for the CSG node, enhancing performance.
///
/// # Methods
///
/// - `new`: Constructor that initializes a CSG node with a specific operation.
/// - `get_aabb_cache`: Retrieves a read lock on the AABB cache.
/// - `set_aabb_cache`: Updates the AABB cache with a new value.
/// - `set_left`: Sets the left child object and updates its parent ID to this CSG node's ID.
/// - `local_intersect`: Performs intersection tests with the child objects, filtering the results based on the CSG operation.
/// - `local_normal_at`: CSG nodes do not have a normal vector; calling this method will panic.
/// - `set_right`: Sets the right child object and updates its parent ID to this CSG node's ID.
/// - `intersection_allowed`: Determines if an intersection is allowed based on the CSG operation and the hit statuses of child objects.
/// - `filter_intersections`: Filters a list of intersections, returning only those that are allowed by the CSG operation.
///
/// # Implementations
///
/// Implements the `Object` trait, allowing CSG nodes to be treated as objects within the ray tracing system.
/// This includes methods for intersection tests, normal vector calculations, and transformation management,
/// adapted to the context of CSG operations.
impl Csg {
    pub fn new(operation: CsgOperation) -> Csg {
        Csg {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            operation,
            left: usize::MAX,
            right: usize::MAX,
            aabb_cache: Arc::new(RwLock::new(None)),  // Initialize the cache as None
        }
    }

    fn get_aabb_cache(&self) -> RwLockReadGuard<Option<AABB>> {
        self.aabb_cache.read().unwrap()
    }

    fn set_aabb_cache(&self, aabb: AABB) {
        let mut cache = self.aabb_cache.write().unwrap();
        *cache = Some(aabb);
    }

    pub fn set_left(&mut self, mut object: Arc<dyn Object + Send>) -> usize {
        Arc::get_mut(&mut object).unwrap().set_parent_id(self.id);
        let child_id = object.get_id();
        add_object(object);
        self.left = child_id;
        child_id
    }

    pub fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let left = get_object(self.left);
        let right = get_object(self.right);
        let mut xs = left.intersect(ray);
        xs.append(&mut right.intersect(ray));
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        self.filter_intersections(&xs)
    }

    pub fn local_normal_at(&self, _vector: &Tuple, _hit: &Intersection) -> Tuple {
        panic!("CSG do not have normals")
    }

    pub fn set_right(&mut self, mut object: Arc<dyn Object + Send>) -> usize {
        Arc::get_mut(&mut object).unwrap().set_parent_id(self.id);
        let child_id = object.get_id();
        add_object(object);
        self.right = child_id;
        child_id
    }

    pub fn intersection_allowed(&self, lhit: bool, inl: bool, inr: bool) -> bool {
        match self.operation {
            CsgOperation::Union => {
                (lhit && !inr) || (!lhit && !inl)
            }
            CsgOperation::Intersection => {
                (lhit && inr) || (!lhit && inl)
            }
            CsgOperation::Difference => {
                (lhit && !inr) || (!lhit && inl)
            }
        }
    }

    pub fn filter_intersections(&self, xs: &Vec<Intersection>) -> Vec<Intersection> {
        let mut inl = false;
        let mut inr = false;
        let mut result = Vec::new();

        for i in xs {
            let left = get_object(self.left);
            let lhit = left.includes(i.object);
            if self.intersection_allowed(lhit, inl, inr) {
                result.push(i.clone());
            }
            if lhit {
                inl = !inl;
            } else {
                inr = !inr;
            }
        }
        result
    }
}

/// Implementation of the `Object` trait for `Csg`.
///
/// This implementation allows `Csg` nodes to participate in the ray tracing system as first-class objects.
/// It provides methods for intersection tests, normal vector calculations, and transformation management,
/// adapted to the context of Constructive Solid Geometry (CSG) operations. Since CSG nodes represent
/// composite objects, their behavior differs from simple geometric shapes, especially in how intersections
/// and normals are calculated.
///
/// # Methods
///
/// - `intersect`: Overrides the trait method to perform intersection tests by transforming the ray into
///   the CSG node's local space, then delegating to `local_intersect` which considers the CSG operation.
/// - `normal_at`: Calculates the normal at a given point on the surface of the CSG node. This method
///   panics because CSG nodes themselves do not have a normal vector; normals are derived from their child objects.
/// - `get_transform`: Returns a reference to the transformation matrix of the CSG node, allowing it to be
///   positioned, rotated, and scaled within the scene.
/// - `get_material`: Panics when called because CSG nodes do not have a material. Materials are properties
///   of the child objects.
/// - `set_transform`: Sets the transformation matrix of the CSG node.
/// - `set_material`: Intentionally left empty because CSG nodes do not have materials.
/// - `debug_string`: Provides a debug string representation of the CSG node, primarily its transformation matrix.
/// - `get_id`: Returns the unique identifier of the CSG node.
/// - `get_parent_id`: Returns the optional parent identifier, allowing for hierarchical scene construction.
/// - `set_parent_id`: Sets the parent identifier, establishing a parent-child relationship in the scene graph.
/// - `get_aabb`: Calculates and returns the axis-aligned bounding box (AABB) of the CSG node, considering
///   the bounds of its child objects and its own transformation.
/// - `includes`: Checks if the given object identifier matches either of the CSG node's child objects.
impl Object for Csg {
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
        panic!("CSG do not have materials")
    }

    fn set_transform(&mut self, transform: Matrix) {
        self.transform = transform;
    }

    fn set_material(&mut self, _material: Material) {
    }

    fn debug_string(&self) -> String {
        format!("CSG: transform: {:?}", self.transform)
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
        // Acquire a read lock and check if the cache is valid
        if let Some(cached_aabb) = *self.get_aabb_cache() {
            return cached_aabb;
        }

        let mut aabb: AABB = AABB::new(
            Tuple::point(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            Tuple::point(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        );

        let left = get_object(self.left);
        let left_aabb = left.get_aabb().apply_transform(left.get_transform());
        aabb.adjust_aabb(&left_aabb);

        let right = get_object(self.right);
        let right_aabb = right.get_aabb().apply_transform(right.get_transform());
        aabb.adjust_aabb(&right_aabb);

        // Cache the computed AABB
        self.set_aabb_cache(aabb);

        aabb
    }

    fn includes(&self, object_id: usize) -> bool {
        object_id == self.left || object_id == self.right
    }
}


#[cfg(test)]
mod tests {
    use crate::raytracer::camera::Camera;
    use crate::raytracer::light::Light;
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::cube::Cube;
    use crate::raytracer::object::plane::Plane;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::scene::Scene;
    use super::*;
    #[test]
    fn evaluating_the_rule_for_a_csg_operation() {
        let c = Csg::new(CsgOperation::Union);
        assert_eq!(c.intersection_allowed(true, true, true), false);
        assert_eq!(c.intersection_allowed(true, true, false), true);
        assert_eq!(c.intersection_allowed(true, false, true), false);
        assert_eq!(c.intersection_allowed(true, false, false), true);
        assert_eq!(c.intersection_allowed(false, true, true), false);
        assert_eq!(c.intersection_allowed(false, true, false), false);
        assert_eq!(c.intersection_allowed(false, false, true), true);
        assert_eq!(c.intersection_allowed(false, false, false), true);

        let c = Csg::new(CsgOperation::Intersection);
        assert_eq!(c.intersection_allowed(true, true, true), true);
        assert_eq!(c.intersection_allowed(true, true, false), false);
        assert_eq!(c.intersection_allowed(true, false, true), true);
        assert_eq!(c.intersection_allowed(true, false, false), false);
        assert_eq!(c.intersection_allowed(false, true, true), true);
        assert_eq!(c.intersection_allowed(false, true, false), true);
        assert_eq!(c.intersection_allowed(false, false, true), false);
        assert_eq!(c.intersection_allowed(false, false, false), false);

        let c = Csg::new(CsgOperation::Difference);
        assert_eq!(c.intersection_allowed(true, true, true), false);
        assert_eq!(c.intersection_allowed(true, true, false), true);
        assert_eq!(c.intersection_allowed(true, false, true), false);
        assert_eq!(c.intersection_allowed(true, false, false), true);
        assert_eq!(c.intersection_allowed(false, true, true), true);
        assert_eq!(c.intersection_allowed(false, true, false), true);
        assert_eq!(c.intersection_allowed(false, false, true), false);
        assert_eq!(c.intersection_allowed(false, false, false), false);
    }

    #[test]
    fn filtering_a_list_of_intersections() {
        let mut c = Csg::new(CsgOperation::Union);
        let s1 = crate::raytracer::object::sphere::Sphere::new();
        let s2 = crate::raytracer::object::sphere::Sphere::new();
        let s1_id = c.set_left(Arc::new(s1));
        let s2_id = c.set_right(Arc::new(s2));
        let i0 = Intersection::new(1.0, s1_id, 0.0, 0.0);
        let i1 = Intersection::new(2.0, s2_id, 0.0, 0.0);
        let i2 = Intersection::new(3.0, s1_id, 0.0, 0.0);
        let i3 = Intersection::new(4.0, s2_id, 0.0, 0.0);
        let xs = vec![i0.clone(), i1.clone(), i2.clone(), i3.clone()];
        let result = c.filter_intersections(&xs);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], i0);
        assert_eq!(result[1], i3);

        let mut c = Csg::new(CsgOperation::Intersection);
        let s1 = crate::raytracer::object::sphere::Sphere::new();
        let s2 = crate::raytracer::object::sphere::Sphere::new();
        let s1_id = c.set_left(Arc::new(s1));
        let s2_id = c.set_right(Arc::new(s2));
        let i0 = Intersection::new(1.0, s1_id, 0.0, 0.0);
        let i1 = Intersection::new(2.0, s2_id, 0.0, 0.0);
        let i2 = Intersection::new(3.0, s1_id, 0.0, 0.0);
        let i3 = Intersection::new(4.0, s2_id, 0.0, 0.0);
        let xs = vec![i0.clone(), i1.clone(), i2.clone(), i3.clone()];
        let result = c.filter_intersections(&xs);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], i1);
        assert_eq!(result[1], i2);

        let mut c = Csg::new(CsgOperation::Difference);
        let s1 = crate::raytracer::object::sphere::Sphere::new();
        let s2 = crate::raytracer::object::sphere::Sphere::new();
        let s1_id = c.set_left(Arc::new(s1));
        let s2_id = c.set_right(Arc::new(s2));
        let i0 = Intersection::new(1.0, s1_id, 0.0, 0.0);
        let i1 = Intersection::new(2.0, s2_id, 0.0, 0.0);
        let i2 = Intersection::new(3.0, s1_id, 0.0, 0.0);
        let i3 = Intersection::new(4.0, s2_id, 0.0, 0.0);
        let xs = vec![i0.clone(), i1.clone(), i2.clone(), i3.clone()];
        let result = c.filter_intersections(&xs);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], i0);
        assert_eq!(result[1], i1);
    }

    #[test]
    #[ignore]
    fn test_render_csg() {
        use crate::color::Color;

        let mut c = Camera::new(800, 400, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 1.5, -5.0);
        let to = Tuple::point(0.0, 1.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut floor = Plane::new();
        floor.transform = Matrix::translate(0.0, 0.0, 0.0);
        floor.material.pattern = Pattern::stripe(Pattern::solid(Color::new(1.0, 0.5, 0.5), Matrix::identity(4)),
                                                 Pattern::solid(Color::new(0.5, 1.0, 0.5), Matrix::identity(4)),
                                                 Matrix::scale(0.1, 0.1, 0.1).multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0)));
        floor.material.specular = 0.0;
        w.add_object(Arc::new(floor));

        let mut left_wall = Plane::new();
        left_wall.material.pattern = Pattern::gradient(Pattern::solid(Color::new(1.0, 0.5, 0.5), Matrix::identity(4)),
                                                       Pattern::solid(Color::new(0.5, 1.0, 0.5), Matrix::identity(4)),
                                                       Matrix::identity(4)
                                                           .multiply(&Matrix::translate(124.0, 124.0, 124.0)
                                                               .multiply(&Matrix::scale(7.0, 7.0, 7.0))
                                                           ));
        left_wall.transform = Matrix::identity(4)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / -4.0))
            .multiply(&Matrix::translate(0.0, 0.0, 5.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
        ;
        left_wall.material.specular = 0.0;
        w.add_object(Arc::new(left_wall));

        let mut right_wall = Plane::new();
        right_wall.transform = Matrix::identity(4)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0))
            .multiply(&Matrix::translate(0.0, 0.0, 5.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
        ;
        right_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        right_wall.material.specular = 0.0;
        w.add_object(Arc::new(right_wall));

        let mut csg = Csg::new(CsgOperation::Difference);

        let mut material = Material::default();
        material.pattern = Pattern::solid(Color::new(0.302, 0.71, 0.98), Matrix::identity(4));
        let mut sphere = Sphere::new();
        sphere.material = material.clone();
        sphere.transform = Matrix::identity(4)
            .multiply(&Matrix::scale(0.6, 0.6, 0.6))
            .multiply(&Matrix::translate(0.0, 1.0, 0.0))
        ;
        csg.set_right(Arc::new(sphere));

        let mut cube = Cube::new();
        cube.material = material.clone();
        cube.transform = Matrix::identity(4)
            .multiply(&Matrix::scale(0.5, 0.5, 0.5))
            .multiply(&Matrix::translate(0.0, 1.2, 0.0))
        ;
        csg.set_left(Arc::new(cube));

        w.add_object(Arc::new(csg));

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file("output.png",1 );
    }
}
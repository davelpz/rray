use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::Object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use crate::EPSILON;
use crate::raytracer::object::db::insert_sentinel;

pub struct Cylinder {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transform: Matrix,
    pub material: Material,
}

impl Cylinder {
    pub fn new(minimum: f64, maximum: f64, closed: bool) -> Cylinder {
        Cylinder {
            id: insert_sentinel(),
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
                xs.push(Intersection::new(t0, self.id));
            }
            let y1 = ray.origin.y + t1 * ray.direction.y;
            if self.minimum < y1 && y1 < self.maximum {
                xs.push(Intersection::new(t1, self.id));
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
            xs.push(Intersection::new(t, self.id));
        }

        // Check for an intersection with the upper end cap by intersecting
        // the ray with the plane at y = maximum
        let t = (self.maximum - ray.origin.y) / ray.direction.y;
        if Cylinder::check_cap(ray, t) {
            xs.push(Intersection::new(t, self.id));
        }

        xs
    }

    pub fn local_normal_at(&self, local_point: &Tuple) -> Tuple {
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

    fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let local_point = self.transform.inverse().multiply_tuple(world_point);
        let local_normal = self.local_normal_at(&local_point);
        let mut world_normal = self.transform.inverse().transpose().multiply_tuple(&local_normal);
        world_normal.w = 0.0;
        world_normal.normalize()
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
}
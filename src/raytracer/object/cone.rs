use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::Object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use crate::EPSILON;
use crate::raytracer::object::db::insert_sentinel;

pub struct Cone {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub minimum: f64,
    pub maximum: f64,
    pub closed: bool,
    pub transform: Matrix,
    pub material: Material,
}

impl Cone {
    pub fn new(minimum: f64, maximum: f64, closed: bool) -> Cone {
        Cone {
            id: insert_sentinel(),
            parent_id: None,
            transform: Matrix::identity(4),
            material: Material::default(),
            minimum,
            maximum,
            closed,
        }
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
            xs.push(Intersection::new(t, self.id));
        }

        // Check for an intersection with the upper end cap by intersecting
        // the ray with the plane at y = maximum
        let t = (maximum - ray.origin.y) / ray.direction.y;
        if Cone::check_cap(ray, t) {
            xs.push(Intersection::new(t, self.id));
        }

        xs
    }

    pub fn local_normal_at(&self, local_point: &Tuple) -> Tuple {
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

impl Object for Cone {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
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
                xs.push(Intersection::new(t, self.id));
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
            xs.push(Intersection::new(t0, self.id));
        }
        let y1 = trans_ray.origin.y + t1 * trans_ray.direction.y;
        if minimum < y1 && y1 < maximum {
            xs.push(Intersection::new(t1, self.id));
        }

        self.intersect_caps(&trans_ray).iter().for_each(|i| xs.push(i.clone()));

        xs
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
}
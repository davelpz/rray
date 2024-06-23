use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::Object;
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;

pub struct Sphere {
    pub id: usize,
    pub transform: Matrix,
    pub material: Material,
}

impl Sphere {
    pub fn new() -> Sphere {
        Sphere {
            id: 0,
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn glass_sphere() -> Sphere {
        let mut m = Material::default();
        m.transparency = 1.0;
        m.refractive_index = 1.5;
        Sphere {
            id: 0,
            transform: Matrix::identity(4),
            material: m,
        }
    }
}

const ORIGIN: Tuple = Tuple { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

impl Object for Sphere {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        let sphere_to_ray = trans_ray.origin.subtract(&ORIGIN);
        let a = trans_ray.direction.dot(&trans_ray.direction);
        let b = 2.0 * trans_ray.direction.dot(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
        let discriminant: f64 = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            vec![]
        } else {
            let t1: f64 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2: f64 = (-b + discriminant.sqrt()) / (2.0 * a);
            vec![Intersection { t: t1, object: self.id },
                 Intersection { t: t2, object: self.id}]
        }
    }

    fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let local_point = self.transform.inverse().multiply_tuple(world_point);
        let local_normal = local_point.subtract(&Tuple::point(0.0, 0.0, 0.0));
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
        format!("Sphere: transform: {:?}, material: {:?}", self.transform, self.material)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}
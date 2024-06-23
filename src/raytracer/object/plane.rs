use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::Object;
use crate::raytracer::ray::Ray;
use crate::EPSILON;
use crate::tuple::Tuple;

pub struct Plane {
    pub id: usize,
    pub transform: Matrix,
    pub material: Material,
}

impl Plane {
    pub fn new() -> Plane {
        Plane {
            id: 0,
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }
}

impl Object for Plane {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        if trans_ray.direction.y.abs() < EPSILON {
            vec![]
        } else {
            let t = -trans_ray.origin.y / trans_ray.direction.y;
            vec![Intersection { t, object: self.id }]
        }
    }

    fn normal_at(&self, _world_point: &Tuple) -> Tuple {
        let local_normal = Tuple::vector(0.0, 1.0, 0.0);
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
        format!("Plane: transform: {:?}, material: {:?}", self.transform, self.material)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}
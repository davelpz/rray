#[allow(dead_code)]

mod cube;
mod sphere;
mod plane;
mod cone;
mod cylinder;

use crate::matrix::Matrix;
use crate::tuple::Tuple;
use crate::ray::Intersection;
use crate::ray::Ray;
use crate::material::Material;
use crate::object::Object;
use uuid::Uuid;

pub const EPSILON: f64 = 0.00001;

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeType {
    Sphere,
    Plane,
    Cube,
    Cylinder(f64, f64, bool),
    Cone(f64, f64, bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub transform: Matrix,
    pub material: Material,
    pub uuid: Uuid,
}

impl Shape {
    pub fn sphere() -> Shape {
        Shape {
            shape_type: ShapeType::Sphere,
            transform: Matrix::identity(4),
            material: Material::default(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn glass_sphere() -> Shape {
        let mut m = Material::default();
        m.transparency = 1.0;
        m.refractive_index = 1.5;
        Shape {
            shape_type: ShapeType::Sphere,
            transform: Matrix::identity(4),
            material: m,
            uuid: Uuid::new_v4(),
        }
    }

    pub fn plane() -> Shape {
        Shape {
            shape_type: ShapeType::Plane,
            transform: Matrix::identity(4),
            material: Material::default(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn cube() -> Shape {
        Shape {
            shape_type: ShapeType::Cube,
            transform: Matrix::identity(4),
            material: Material::default(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn cylinder(minimum: f64, maximum: f64, closed: bool) -> Shape {
        Shape {
            shape_type: ShapeType::Cylinder(minimum, maximum, closed),
            transform: Matrix::identity(4),
            material: Material::default(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn cone(minimum: f64, maximum: f64, closed: bool) -> Shape {
        Shape {
            shape_type: ShapeType::Cone(minimum, maximum, closed),
            transform: Matrix::identity(4),
            material: Material::default(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        match self.shape_type {
            ShapeType::Sphere => sphere::local_intersect(&self, &trans_ray),
            ShapeType::Plane => plane::local_intersect(&self, &trans_ray),
            ShapeType::Cube => cube::local_intersect(&self, &trans_ray),
            ShapeType::Cylinder(_, _, _) => cylinder::local_intersect(&self, &trans_ray),
            ShapeType::Cone(_, _, _) => cone::local_intersect(&self, &trans_ray),
        }
    }

    pub fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let local_point = self.transform.inverse().multiply_tuple(world_point);
        let local_normal = match self.shape_type {
            ShapeType::Sphere => sphere::local_normal_at(&self, &local_point),
            ShapeType::Plane => plane::local_normal_at(&self, &local_point),
            ShapeType::Cube => cube::local_normal_at(&self, &local_point),
            ShapeType::Cylinder(_, _, _) => cylinder::local_normal_at(&self, &local_point),
            ShapeType::Cone(_, _, _) => cone::local_normal_at(&self, &local_point),
        };
        let mut world_normal = self.transform.inverse().transpose().multiply_tuple(&local_normal);
        world_normal.w = 0.0;
        world_normal.normalize()
    }
}

impl Object for Shape {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        self.intersect(ray)
    }

    fn normal_at(&self, world_point: &Tuple) -> Tuple {
        self.normal_at(&world_point)
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
        format!("{:?}", self)
    }

    fn get_uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[cfg(test)]
mod tests;


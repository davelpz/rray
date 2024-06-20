#[allow(dead_code)]

mod cube;
mod sphere;
mod plane;

use crate::matrix::matrix::Matrix;
use crate::tuple::tuple::Tuple;
use crate::ray::ray::Intersection;
use crate::ray::ray::Ray;
use crate::material::material::Material;

pub const EPSILON: f64 = 0.00001;

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeType {
    Sphere,
    Plane,
    Cube
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub center: Tuple,
    pub transform: Matrix,
    pub material: Material,
}

impl Shape {
    pub fn sphere() -> Shape {
        Shape {
            shape_type: ShapeType::Sphere,
            center: Tuple::point(0.0, 0.0, 0.0),
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn glass_sphere() -> Shape {
        let mut m = Material::default();
        m.transparency = 1.0;
        m.refractive_index = 1.5;
        Shape {
            shape_type: ShapeType::Sphere,
            center: Tuple::point(0.0, 0.0, 0.0),
            transform: Matrix::identity(4),
            material: m,
        }
    }

    pub fn plane() -> Shape {
        Shape {
            shape_type: ShapeType::Plane,
            center: Tuple::point(0.0, 0.0, 0.0),
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn cube() -> Shape {
        Shape {
            shape_type: ShapeType::Cube,
            center: Tuple::point(0.0, 0.0, 0.0),
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        match self.shape_type {
            ShapeType::Sphere => sphere::local_intersect(&self, &trans_ray),
            ShapeType::Plane => plane::local_intersect(&self, &trans_ray),
            ShapeType::Cube => cube::local_intersect(&self, &trans_ray),
        }
    }

    pub fn normal_at(&self, world_point: &Tuple) -> Tuple {
        let local_point = self.transform.inverse().multiply_tuple(world_point);
        let local_normal = match self.shape_type {
            ShapeType::Sphere => sphere::local_normal_at(&self, &local_point),
            ShapeType::Plane => plane::local_normal_at(&self, &local_point),
            ShapeType::Cube => cube::local_normal_at(&self, &local_point),
        };
        let mut world_normal = self.transform.inverse().transpose().multiply_tuple(&local_normal);
        world_normal.w = 0.0;
        world_normal.normalize()
    }
}


#[cfg(test)]
mod tests;

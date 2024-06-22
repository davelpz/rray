use std::any::Any;
use std::fmt::{Debug, Formatter};
use crate::material::Material;
use crate::ray::{Intersection, Ray};
use crate::tuple::Tuple;
use crate::matrix::Matrix;
use uuid::Uuid;

pub trait Object: Sync + Any {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection>;
    fn normal_at(&self, point: &Tuple) -> Tuple;
    fn get_transform(&self) -> &Matrix;
    fn get_material(&self) -> &Material;
    fn set_transform(&mut self, transform: Matrix);
    fn set_material(&mut self, material: Material);
    fn debug_string(&self) -> String;
    fn get_uuid(&self) -> Uuid;
}

impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.get_uuid() == other.get_uuid()
    }
}

impl Debug for dyn Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_string())
    }
}


use std::sync::Arc;
use crate::matrix::Matrix;
use crate::raytracer::object::db::{add_object, get_next_id};
use crate::raytracer::object::Object;

pub enum CsgOperation {
    Union,
    Intersection,
    Difference,
}

pub struct Csg {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub transform: Matrix,
    pub operation: CsgOperation,
    pub left: usize,
    pub right: usize,
}

impl Csg {
    pub fn new(operation: CsgOperation) -> Csg {
        Csg {
            id: get_next_id(),
            parent_id: None,
            transform: Matrix::identity(4),
            operation,
            left: usize::MAX,
            right: usize::MAX,
        }
    }

    pub fn set_left(&mut self, mut object: Arc<dyn Object + Send>) -> usize {
        Arc::get_mut(&mut object).unwrap().set_parent_id(self.id);
        let child_id = object.get_id();
        add_object(object);
        self.left = child_id;
        child_id
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
}


#[cfg(test)]
mod tests {
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
}
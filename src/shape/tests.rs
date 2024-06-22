use crate::matrix::Matrix;
use crate::object::Object;
use crate::ray::Ray;
use crate::tuple::Tuple;
use super::Shape;

#[test]
fn test_intersect() {
    let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
    let s = Shape::sphere();
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
        let s: Box<dyn Object> = Box::new(s.clone());
        assert_eq!(xs[0].object.get_uuid(), s.get_uuid());
        assert_eq!(xs[1].object.get_uuid(), s.get_uuid());
    }
}

#[test]
fn test_transform() {
    let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
    let mut s = Shape::sphere();
    s.transform = Matrix::scale(2.0, 2.0, 2.0);
    let xs = s.intersect(&r);
    assert_eq!(xs.len(), 2);
    assert_eq!(xs[0].t, 3.0);
    assert_eq!(xs[1].t, 7.0);

    let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
    let mut s = Shape::sphere();
    s.transform = Matrix::translate(5.0, 0.0, 0.0);
    let xs = s.intersect(&r);
    assert_eq!(xs.len(), 0);
}

#[test]
fn normal_at_surface_point() {
    let s = Shape::sphere();
    let point = Tuple::point(1.0, 0.0, 0.0);
    let expected_normal = Tuple::vector(1.0, 0.0, 0.0);
    assert_eq!(s.normal_at(&point), expected_normal);

    let point = Tuple::point(0.0, 1.0, 0.0);
    let expected_normal = Tuple::vector(0.0, 1.0, 0.0);
    assert_eq!(s.normal_at(&point), expected_normal);

    let point = Tuple::point(0.0, 0.0, 1.0);
    let expected_normal = Tuple::vector(0.0, 0.0, 1.0);
    assert_eq!(s.normal_at(&point), expected_normal);
}

#[test]
fn normal_at_non_axial_point() {
    let s = Shape::sphere();
    let sqrt_of_three_over_three = 3f64.sqrt() / 3.0;
    let point = Tuple::point(sqrt_of_three_over_three, sqrt_of_three_over_three, sqrt_of_three_over_three);
    let expected_normal = Tuple::vector(sqrt_of_three_over_three, sqrt_of_three_over_three, sqrt_of_three_over_three);
    assert_eq!(s.normal_at(&point), expected_normal);
    assert_eq!(s.normal_at(&point).magnitude(), 1.0);
}

#[allow(dead_code)]

pub mod shape {
    use crate::matrix::matrix::Matrix;
    use crate::tuple::tuple::Tuple;
    use crate::ray::ray::Intersection;
    use crate::ray::ray::Ray;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Sphere {
        pub center: Tuple,
        pub transform: Matrix,
    }

    impl Sphere {
        pub fn new() -> Sphere {
            Sphere {
                center: Tuple::point(0.0, 0.0, 0.0),
                transform: Matrix::identity(4),
            }
        }

        pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
            let trans_ray = ray.transform(&self.transform.inverse());
            let sphere_to_ray = trans_ray.origin.subtract(&self.center);
            let a = trans_ray.direction.dot(&trans_ray.direction);
            let b = 2.0 * trans_ray.direction.dot(&sphere_to_ray);
            let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                vec![]
            } else {
                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                vec![Intersection { t: t1, object: self },
                     Intersection { t: t2, object: self }]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::matrix::Matrix;
    use crate::ray::ray::Ray;
    use crate::tuple::tuple::Tuple;
    use super::shape::Sphere;

    #[test]
    fn test_intersect() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
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
            assert_eq!(xs[0].object, &s);
            assert_eq!(xs[1].object, &s);
        }
    }

    #[test]
    fn test_transform() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::scale(2.0, 2.0, 2.0);
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::translate(5.0, 0.0, 0.0);
        let xs = s.intersect(&r);
        assert_eq!(xs.len(), 0);
    }
}

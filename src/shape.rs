#[allow(dead_code)]

pub mod shape {
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

        pub fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
            let trans_ray = ray.transform(&self.transform.inverse());
            match self.shape_type {
                ShapeType::Sphere => self.local_intersect_sphere(&trans_ray),
                ShapeType::Plane => self.local_intersect_plane(&trans_ray),
                _ => vec![]
            }
        }

        fn local_intersect_sphere(&self, ray: &Ray) -> Vec<Intersection> {
            let sphere_to_ray = ray.origin.subtract(&self.center);
            let a = ray.direction.dot(&ray.direction);
            let b = 2.0 * ray.direction.dot(&sphere_to_ray);
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

        fn local_intersect_plane(&self, ray: &Ray) -> Vec<Intersection> {
            if ray.direction.y.abs() < EPSILON {
                vec![]
            } else {
                let t = -ray.origin.y / ray.direction.y;
                vec![Intersection { t, object: self }]
            }
        }

        pub fn normal_at(&self, world_point: &Tuple) -> Tuple {
            let local_point = self.transform.inverse().multiply_tuple(world_point);
            let local_normal = match self.shape_type {
                ShapeType::Sphere => self.local_normal_at_sphere(&local_point),
                ShapeType::Plane => self.local_normal_at_plane(&local_point),
                _ => Tuple::vector(0.0, 0.0, 0.0)
            };
            let mut world_normal = self.transform.inverse().transpose().multiply_tuple(&local_normal);
            world_normal.w = 0.0;
            world_normal.normalize()
        }

        fn local_normal_at_sphere(&self, world_point: &Tuple) -> Tuple {
            world_point.subtract(&Tuple::point(0.0, 0.0, 0.0))
        }

        #[allow(unused_variables)]
        fn local_normal_at_plane(&self, world_point: &Tuple) -> Tuple {
            Tuple::vector(0.0, 1.0, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::matrix::Matrix;
    use crate::ray::ray::Ray;
    use crate::tuple::tuple::Tuple;
    use super::shape::Shape;

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
            assert_eq!(xs[0].object, &s);
            assert_eq!(xs[1].object, &s);
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
}

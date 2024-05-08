#[allow(dead_code)]

pub mod ray {
    use crate::tuple::tuple::Tuple;
    use crate::matrix::matrix::Matrix;

    // Sphere struct
    #[derive(Debug, Clone, PartialEq)]
    pub struct Sphere {
        pub center: Tuple,
        pub transform: Matrix,
    }

    // Intersection struct
    #[derive(Debug, Clone, PartialEq)]
    pub struct Intersection<'a> {
        pub t: f64,
        pub object: &'a Sphere,
    }

    // Ray struct
    #[derive(Debug, Clone)]
    pub struct Ray {
        pub origin: Tuple,
        pub direction: Tuple,
    }

    impl Ray {
        pub fn new(origin: Tuple, direction: Tuple) -> Ray {
            Ray { origin, direction }
        }

        pub fn position(&self, t: f64) -> Tuple {
            self.origin.add(&self.direction.multiply(t))
        }

        pub fn intersect<'a>(&'a self, sphere: &'a Sphere) -> Vec<Intersection> {
            let trans_ray = self.transform(&sphere.transform.inverse());
            let sphere_to_ray = trans_ray.origin.subtract(&sphere.center);
            let a = trans_ray.direction.dot(&trans_ray.direction);
            let b = 2.0 * trans_ray.direction.dot(&sphere_to_ray);
            let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;
            let discriminant = b.powi(2) - 4.0 * a * c;
            if discriminant < 0.0 {
                vec![]
            } else {
                let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                vec![Intersection{t: t1, object: sphere},
                     Intersection{t: t2, object: sphere}]
            }
        }

        pub fn transform(&self, matrix: &Matrix) -> Ray {
            Ray {
                origin: matrix.multiply_tuple(&self.origin),
                direction: matrix.multiply_tuple(&self.direction),
            }
        }
    }

    pub fn hit<'a>(xs: &'a Vec<Intersection>) -> Option<&'a Intersection<'a>> {
        let mut result = None;
        let mut t = f64::MAX;
        for x in xs {
            if x.t >= 0.0 && x.t < t {
                t = x.t;
                result = Some(x);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::matrix::Matrix;
    use super::ray::Ray;
    use crate::tuple::tuple::Tuple;

    #[test]
    fn test_ray() {
        let origin = Tuple::point(1.0, 2.0, 3.0);
        let direction = Tuple::vector(4.0, 5.0, 6.0);
        let r = Ray::new(origin.clone(), direction.clone());
        assert_eq!(r.origin, origin);
        assert_eq!(r.direction, direction);
    }

    #[test]
    fn test_position() {
        let r = Ray::new(Tuple::point(2.0, 3.0, 4.0), Tuple::vector(1.0, 0.0, 0.0));
        assert_eq!(r.position(0.0), Tuple::point(2.0, 3.0, 4.0));
        assert_eq!(r.position(1.0), Tuple::point(3.0, 3.0, 4.0));
        assert_eq!(r.position(-1.0), Tuple::point(1.0, 3.0, 4.0));
        assert_eq!(r.position(2.5), Tuple::point(4.5, 3.0, 4.0));
    }

    #[test]
    fn test_intersect() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = super::ray::Sphere { center: Tuple::point(0.0, 0.0, 0.0), transform: Matrix::identity(4)};
        let xs = r.intersect(&s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 6.0);

        let r = Ray::new(Tuple::point(0.0, 1.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = r.intersect(&s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 5.0);
        assert_eq!(xs[1].t, 5.0);

        let r = Ray::new(Tuple::point(0.0, 2.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = r.intersect(&s);
        assert_eq!(xs.len(), 0);

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        {
            let xs = r.intersect(&s);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, -1.0);
            assert_eq!(xs[1].t, 1.0);
        }

        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        {
            let xs = r.intersect(&s);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, -6.0);
            assert_eq!(xs[1].t, -4.0);
            assert_eq!(xs[0].object, &s);
            assert_eq!(xs[1].object, &s);
        }
    }

    #[test]
    fn test_hit() {
        let s = super::ray::Sphere { center: Tuple::point(0.0, 0.0, 0.0), transform: Matrix::identity(4)};
        let i1 = super::ray::Intersection { t: 1.0, object: &s };
        let i2 = super::ray::Intersection { t: 2.0, object: &s };
        let xs = vec![i1, i2];
        let i = super::ray::hit(&xs);
        assert_eq!(i.unwrap().t, 1.0);

        let i1 = super::ray::Intersection { t: -1.0, object: &s };
        let i2 = super::ray::Intersection { t: 1.0, object: &s };
        let xs = vec![i1, i2];
        let i = super::ray::hit(&xs);
        assert_eq!(i.unwrap().t, 1.0);

        let i1 = super::ray::Intersection { t: -2.0, object: &s };
        let i2 = super::ray::Intersection { t: -1.0, object: &s };
        let xs = vec![i1, i2];
        let i = super::ray::hit(&xs);
        assert_eq!(i, None);

        let i1 = super::ray::Intersection { t: 5.0, object: &s };
        let i2 = super::ray::Intersection { t: 7.0, object: &s };
        let i3 = super::ray::Intersection { t: -3.0, object: &s };
        let i4 = super::ray::Intersection { t: 2.0, object: &s };
        let xs = vec![i1, i2, i3, i4];
        let i = super::ray::hit(&xs);
        assert_eq!(i.unwrap().t, 2.0);

        let xs = vec![];
        let i = super::ray::hit(&xs);
        assert_eq!(i, None);
    }

    #[test]
    fn test_transform() {
        let r = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let m = Matrix::translate(3.0, 4.0, 5.0);
        let r2 = r.transform(&m);
        assert_eq!(r2.origin, Tuple::point(4.0, 6.0, 8.0));
        assert_eq!(r2.direction, Tuple::vector(0.0, 1.0, 0.0));

        let r = Ray::new(Tuple::point(1.0, 2.0, 3.0), Tuple::vector(0.0, 1.0, 0.0));
        let m = Matrix::scale(2.0, 3.0, 4.0);
        let r2 = r.transform(&m);
        assert_eq!(r2.origin, Tuple::point(2.0, 6.0, 12.0));
        assert_eq!(r2.direction, Tuple::vector(0.0, 3.0, 0.0));

        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = super::ray::Sphere { center: Tuple::point(0.0, 0.0, 0.0), transform: Matrix::scale(2.0, 2.0, 2.0)};
        let xs = r.intersect(&s);
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].t, 3.0);
        assert_eq!(xs[1].t, 7.0);

        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = super::ray::Sphere { center: Tuple::point(0.0, 0.0, 0.0), transform: Matrix::translate(5.0, 0.0, 0.0)};
        let xs = r.intersect(&s);
        assert_eq!(xs.len(), 0);
    }

    #[test]
    #[ignore]
    fn test_render() {
        let ray_origin = Tuple::point(0.0, 0.0, -5.0);
        let wall_z = 10.0;
        let wall_size = 7.0;
        let canvas_pixels = 100;
        let pixel_size = wall_size / canvas_pixels as f64;
        let half = wall_size / 2.0;
        let mut canvas = crate::canvas::canvas::Canvas::new(canvas_pixels, canvas_pixels);
        let color = crate::color::color::Color::new(1.0, 0.0, 0.0);
        let s = super::ray::Sphere { center: Tuple::point(0.0, 0.0, 0.0),
            transform: Matrix::scale(1.0, 0.5, 1.0)};

        for y in 0..canvas_pixels {
            let world_y = half - pixel_size * y as f64;
            for x in 0..canvas_pixels {
                let world_x = -half + pixel_size * x as f64;
                let position = Tuple::point(world_x, world_y, wall_z);
                let r = Ray::new(ray_origin.clone(), position.subtract(&ray_origin).normalize());
                let xs = r.intersect(&s);
                if let Some(_i) = super::ray::hit(&xs) {
                    canvas.write_pixel(x, y, color);
                }
            }
        }

        canvas.write_to_file("canvas.png");
    }
}
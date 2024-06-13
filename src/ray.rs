#[allow(dead_code)]

pub mod ray {
    use crate::tuple::tuple::Tuple;
    use crate::matrix::matrix::Matrix;
    use crate::shape::shape::Shape;

    pub const EPSILON: f64 = 0.00001;

    // Intersection struct
    #[derive(Debug, Clone, PartialEq)]
    pub struct Intersection<'a> {
        pub t: f64,
        pub object: &'a Shape,
    }

    pub struct Computations<'a> {
        pub t: f64,
        pub object: &'a Shape,
        pub point: Tuple,
        pub eyev: Tuple,
        pub normalv: Tuple,
        pub inside: bool,
        pub over_point: Tuple,
    }

    impl<'a> Intersection<'a> {
        pub fn prepare_computations(&self, r: &Ray) -> Computations<'a> {
            let point = r.position(self.t);
            let eyev = r.direction.negate();
            let normalv = self.object.normal_at(&point);
            let inside = normalv.dot(&eyev) < 0.0;
            let normalv = if inside { normalv.negate() } else { normalv };
            let over_point = point.add(&normalv.multiply(EPSILON));
            Computations { t: self.t, object: self.object, point, eyev, normalv, inside, over_point }
        }
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
    use crate::shape::shape::Shape;
    use crate::color::color::Color;
    use crate::canvas::canvas::Canvas;
    use crate::light::light::Light;
    use crate::light::light::lighting;
    use crate::material::material::PatternType;

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
    fn test_hit() {
        let s = Shape::sphere();
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
    }

    #[test]
    fn intersections_the_hit_should_offset_the_point() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Shape::sphere();
        s.transform = Matrix::translate(0.0, 0.0, 1.0);
        let i = super::ray::Intersection { t: 5.0, object: &s };
        let comps = i.prepare_computations(&r);
        assert!(comps.over_point.z < -super::ray::EPSILON / 2.0);
        assert!(comps.point.z > comps.over_point.z);
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
        let mut s = Shape::sphere();
        s.transform = Matrix::scale(1.0, 0.5, 1.0);

        for y in 0..canvas_pixels {
            let world_y = half - pixel_size * y as f64;
            for x in 0..canvas_pixels {
                let world_x = -half + pixel_size * x as f64;
                let position = Tuple::point(world_x, world_y, wall_z);
                let r = Ray::new(ray_origin.clone(), position.subtract(&ray_origin).normalize());
                let xs = s.intersect(&r);
                if let Some(_i) = super::ray::hit(&xs) {
                    canvas.write_pixel(x, y, color);
                }
            }
        }

        canvas.write_to_file("canvas.png");
    }

    #[test]
    #[ignore]
    fn test_render2() {
        let ray_origin = Tuple::point(0.0, 0.0, -5.0);
        let wall_z = 10.0;
        let wall_size = 7.0;
        let canvas_pixels = 300;
        let pixel_size = wall_size / canvas_pixels as f64;
        let half = wall_size / 2.0;
        let mut canvas = Canvas::new(canvas_pixels, canvas_pixels);
        let mut s = Shape::sphere();
        //s.transform = Matrix::scale(1.0, 0.5, 1.0);
        s.material.pattern = PatternType::Solid(Color::new(1.0, 0.2, 1.0));

        let light_position = Tuple::point(-10.0, 10.0, -10.0);
        let light_color = Color::new(1.0, 1.0, 1.0);
        let light = Light::new_point_light(light_position, light_color);

        for y in 0..canvas_pixels {
            let world_y = half - pixel_size * y as f64;
            for x in 0..canvas_pixels {
                let world_x = -half + pixel_size * x as f64;
                let position = Tuple::point(world_x, world_y, wall_z);
                let r = Ray::new(ray_origin.clone(), position.subtract(&ray_origin).normalize());
                let xs = s.intersect(&r);
                if let Some(hit) = super::ray::hit(&xs) {
                    let point = r.position(hit.t);
                    let normal = hit.object.normal_at(&point);
                    let eye = r.direction.negate();
                    let color = lighting(&hit.object.material, &hit.object, &light, &point, &eye, &normal, false);
                    canvas.write_pixel(x, y, color);
                }
            }
        }

        canvas.write_to_file("canvas.png");
    }

    #[test]
    fn test_prepare_computations() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Shape::sphere();
        let i = super::ray::Intersection { t: 4.0, object: &s };
        let comps = i.prepare_computations(&r);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.object, i.object);
        assert_eq!(comps.point, Tuple::point(0.0, 0.0, -1.0));
        assert_eq!(comps.eyev, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, false);
    }

    #[test]
    fn test_prepare_computations_inside() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Shape::sphere();
        let i = super::ray::Intersection { t: 1.0, object: &s };
        let comps = i.prepare_computations(&r);
        assert_eq!(comps.t, i.t);
        assert_eq!(comps.object, i.object);
        assert_eq!(comps.point, Tuple::point(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
    }
}
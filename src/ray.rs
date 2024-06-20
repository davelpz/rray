#[allow(dead_code)]

use crate::tuple::Tuple;
use crate::matrix::Matrix;
use crate::shape::Shape;

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
    pub under_point: Tuple,
    pub reflectv: Tuple,
    pub n1: f64,
    pub n2: f64,
}

impl Computations<'_> {
    pub fn schlick(&self) -> f64 {
        let mut cos = self.eyev.dot(&self.normalv);
        if self.n1 > self.n2 {
            let n = self.n1 / self.n2;
            let sin2_t = n * n * (1.0 - cos * cos);
            if sin2_t > 1.0 {
                return 1.0;
            }

            let cos_t = (1.0 - sin2_t).sqrt();
            cos = cos_t;
        }

        let r0 = ((self.n1 - self.n2) / (self.n1 + self.n2)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cos).powi(5)
    }
}

impl<'a> Intersection<'a> {
    pub fn new(t: f64, object: &'a Shape) -> Intersection<'a> {
        Intersection { t, object }
    }

    pub fn prepare_computations(&self, r: &Ray, xs: &Vec<Intersection>) -> Computations<'a> {
        let point = r.position(self.t);
        let eyev = r.direction.negate();
        let normalv = self.object.normal_at(&point);
        let inside = normalv.dot(&eyev) < 0.0;
        let normalv = if inside { normalv.negate() } else { normalv };
        let over_point = point.add(&normalv.multiply(EPSILON));
        let under_point = point.subtract(&normalv.multiply(EPSILON));
        let reflectv = r.direction.reflect(&normalv);

        let mut n1 = 1.0;
        let mut n2 = 1.0;
        let mut containers: Vec<&Shape> = vec![];
        for i in xs {
            if *i == *self {
                if containers.is_empty() {
                    n1 = 1.0;
                } else {
                    n1 = containers.last().unwrap().material.refractive_index;
                }
            }

            if containers.contains(&i.object) {
                if let Some(index) = containers.iter().position(|shape| shape == &i.object) {
                    containers.remove(index);
                }
            } else {
                containers.push(i.object);
            }

            if *i == *self {
                if containers.is_empty() {
                    n2 = 1.0;
                } else {
                    n2 = containers.last().unwrap().material.refractive_index;
                }
            }
        }

        Computations { t: self.t, object: self.object, point, eyev, normalv, inside, over_point, under_point, reflectv, n1, n2 }
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
        if x.t >= 0.0 && x.t < t { //maybe take out check for t >= 0.0
            t = x.t;
            result = Some(x);
        }
    }
    result
}


#[cfg(test)]
mod tests {
    use crate::matrix::Matrix;
    use super::Ray;
    use crate::tuple::Tuple;
    use crate::shape::Shape;
    use crate::color::Color;
    use crate::canvas::Canvas;
    use crate::light::Light;
    use crate::light::lighting;
    use crate::pattern::Pattern;

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
        let i1 = super::Intersection { t: 1.0, object: &s };
        let i2 = super::Intersection { t: 2.0, object: &s };
        let xs = vec![i1, i2];
        let i = super::hit(&xs);
        assert_eq!(i.unwrap().t, 1.0);

        let i1 = super::Intersection { t: -1.0, object: &s };
        let i2 = super::Intersection { t: 1.0, object: &s };
        let xs = vec![i1, i2];
        let i = super::hit(&xs);
        assert_eq!(i.unwrap().t, 1.0);

        let i1 = super::Intersection { t: -2.0, object: &s };
        let i2 = super::Intersection { t: -1.0, object: &s };
        let xs = vec![i1, i2];
        let i = super::hit(&xs);
        assert_eq!(i, None);

        let i1 = super::Intersection { t: 5.0, object: &s };
        let i2 = super::Intersection { t: 7.0, object: &s };
        let i3 = super::Intersection { t: -3.0, object: &s };
        let i4 = super::Intersection { t: 2.0, object: &s };
        let xs = vec![i1, i2, i3, i4];
        let i = super::hit(&xs);
        assert_eq!(i.unwrap().t, 2.0);

        let xs = vec![];
        let i = super::hit(&xs);
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
        let i = super::Intersection { t: 5.0, object: &s };
        let xs = vec![i];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert!(comps.over_point.z < -super::EPSILON / 2.0);
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
        let mut canvas = crate::canvas::Canvas::new(canvas_pixels, canvas_pixels);
        let color = crate::color::Color::new(1.0, 0.0, 0.0);
        let mut s = Shape::sphere();
        s.transform = Matrix::scale(1.0, 0.5, 1.0);

        for y in 0..canvas_pixels {
            let world_y = half - pixel_size * y as f64;
            for x in 0..canvas_pixels {
                let world_x = -half + pixel_size * x as f64;
                let position = Tuple::point(world_x, world_y, wall_z);
                let r = Ray::new(ray_origin.clone(), position.subtract(&ray_origin).normalize());
                let xs = s.intersect(&r);
                if let Some(_i) = super::hit(&xs) {
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
        s.material.pattern = Pattern::solid(Color::new(1.0, 0.2, 1.0), Matrix::identity(4));

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
                if let Some(hit) = super::hit(&xs) {
                    let point = r.position(hit.t);
                    let normal = hit.object.normal_at(&point);
                    let eye = r.direction.negate();
                    let color = lighting(&hit.object, &light, &point, &eye, &normal, false);
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
        let i = super::Intersection { t: 4.0, object: &s };
        let xs = vec![i];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert_eq!(comps.t, xs[0].t);
        assert_eq!(comps.object, xs[0].object);
        assert_eq!(comps.point, Tuple::point(0.0, 0.0, -1.0));
        assert_eq!(comps.eyev, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, false);
    }

    #[test]
    fn test_prepare_computations_inside() {
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Shape::sphere();
        let xs = vec![super::Intersection { t: 1.0, object: &s }];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert_eq!(comps.t, xs[0].t);
        assert_eq!(comps.object, xs[0].object);
        assert_eq!(comps.point, Tuple::point(0.0, 0.0, 1.0));
        assert_eq!(comps.eyev, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.normalv, Tuple::vector(0.0, 0.0, -1.0));
        assert_eq!(comps.inside, true);
    }

    #[test]
    fn precomputing_the_reflection_vector() {
        let s = Shape::plane();
        let r = Ray::new(Tuple::point(0.0, 1.0, -1.0), Tuple::vector(0.0, -2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
        let xs = vec![super::Intersection { t: 2.0_f64.sqrt(), object: &s }];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert_eq!(comps.reflectv, Tuple::vector(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
    }

    #[test]
    fn finding_n1_and_n2_at_various_intersections() {
        let mut a = Shape::glass_sphere();
        a.transform = Matrix::scale(2.0, 2.0, 2.0);
        a.material.refractive_index = 1.5;

        let mut b = Shape::glass_sphere();
        b.transform = Matrix::translate(0.0, 0.0, -0.25);
        b.material.refractive_index = 2.0;

        let mut c = Shape::glass_sphere();
        c.transform = Matrix::translate(0.0, 0.0, 0.25);
        c.material.refractive_index = 2.5;

        let r = Ray::new(Tuple::point(0.0, 0.0, -4.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![
            super::Intersection { t: 2.0, object: &a },
            super::Intersection { t: 2.75, object: &b },
            super::Intersection { t: 3.25, object: &c },
            super::Intersection { t: 4.75, object: &b },
            super::Intersection { t: 5.25, object: &c },
            super::Intersection { t: 6.0, object: &a },
        ];

        let expected_n1 = vec![1.0, 1.5, 2.0, 2.5, 2.5, 1.5];
        let expected_n2 = vec![1.5, 2.0, 2.5, 2.5, 1.5, 1.0];

        for i in 0..xs.len() {
            let comps = xs[i].prepare_computations(&r, &xs);
            assert_eq!(comps.n1, expected_n1[i]);
            assert_eq!(comps.n2, expected_n2[i]);
        }
    }

    #[test]
    fn underpoint_is_offset_below_the_surface() {
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Shape::glass_sphere();
        s.transform = Matrix::translate(0.0, 0.0, 1.0);
        let i = super::Intersection { t: 5.0, object: &s };
        let xs = vec![i];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert!(comps.under_point.z > super::EPSILON / 2.0);
        assert!(comps.point.z < comps.under_point.z);
    }
}
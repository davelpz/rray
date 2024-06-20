use crate::ray::{Intersection, Ray};
use crate::tuple::{EPSILON, Tuple};
use crate::shape::Shape;

pub fn local_intersect<'a>(cube: &'a Shape, ray: &Ray) -> Vec<Intersection<'a>> {
    let (xtmin, xtmax) = check_axis(ray.origin.x, ray.direction.x);
    let (ytmin, ytmax) = check_axis(ray.origin.y, ray.direction.y);
    let (ztmin, ztmax) = check_axis(ray.origin.z, ray.direction.z);
    let tmin = xtmin.max(ytmin).max(ztmin);
    let tmax = xtmax.min(ytmax).min(ztmax);
    if tmin > tmax {
        vec![]
    } else {
        vec![Intersection::new(tmin, cube),
             Intersection::new(tmax, cube )]
    }
}

fn check_axis(origin: f64, direction: f64) -> (f64, f64) {
    let tmin_numerator = -1.0 - origin;
    let tmax_numerator = 1.0 - origin;
    let (tmin, tmax) = if direction.abs() >= EPSILON {
        (tmin_numerator / direction, tmax_numerator / direction)
    } else {
        (tmin_numerator * f64::INFINITY, tmax_numerator * f64::INFINITY)
    };
    if tmin > tmax {
        (tmax, tmin)
    } else {
        (tmin, tmax)
    }
}

pub fn local_normal_at(_cube: &Shape, local_point: &Tuple) -> Tuple {
    let maxc = local_point.x.abs().max(local_point.y.abs()).max(local_point.z.abs());
    if maxc == local_point.x.abs() {
        Tuple::vector(local_point.x, 0.0, 0.0)
    } else if maxc == local_point.y.abs() {
        Tuple::vector(0.0, local_point.y, 0.0)
    } else {
        Tuple::vector(0.0, 0.0, local_point.z)
    }
}

#[cfg(test)]
mod tests {
    use super::{local_intersect, local_normal_at, Shape};
    use crate::ray::Ray;
    use crate::tuple::Tuple;
    #[test]
    fn test_check_axis() {
        let (tmin, tmax) = super::check_axis(5.0, 1.0);
        assert_eq!(tmin, -6.0);
        assert_eq!(tmax, -4.0);

        let (tmin, tmax) = super::check_axis(5.0, -1.0);
        assert_eq!(tmin, 4.0);
        assert_eq!(tmax, 6.0);
    }

    #[test]
    fn ray_intersects_a_cube() {
        let c = Shape::cube();
        let origins = vec![
            Tuple::point(5.0, 0.5, 0.0),
            Tuple::point(-5.0, 0.5, 0.0),
            Tuple::point(0.5, 5.0, 0.0),
            Tuple::point(0.5, -5.0, 0.0),
            Tuple::point(0.5, 0.0, 5.0),
            Tuple::point(0.5, 0.0, -5.0),
            Tuple::point(0.0, 0.5, 0.0),
        ];
        let directions = vec![
          Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, 1.0),
        ];
        let t1s = vec![4.0, 4.0, 4.0, 4.0, 4.0, 4.0, -1.0];
        let t2s = vec![6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 1.0];
        for i in 0..7 {
            let r = Ray::new(origins[i], directions[i]);
            let xs = local_intersect(&c, &r);
            assert_eq!(xs.len(), 2);
            assert_eq!(xs[0].t, t1s[i]);
            assert_eq!(xs[1].t, t2s[i]);
        }
    }

    #[test]
    fn ray_misses_a_cube() {
        let c = Shape::cube();
        let origins = vec![
            Tuple::point(-2.0, 0.0, 0.0),
            Tuple::point(0.0, -2.0, 0.0),
            Tuple::point(0.0, 0.0, -2.0),
            Tuple::point(2.0, 0.0, 2.0),
            Tuple::point(0.0, 2.0, 2.0),
            Tuple::point(2.0, 2.0, 0.0),
        ];
        let directions = vec![
            Tuple::vector(0.2673, 0.5345, 0.8018),
            Tuple::vector(0.8018, 0.2673, 0.5345),
            Tuple::vector(0.5345, 0.8018, 0.2673),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
        ];
        for i in 0..6 {
            let r = Ray::new(origins[i], directions[i]);
            let xs = local_intersect(&c, &r);
            assert_eq!(xs.len(), 0);
        }
    }

    #[test]
    fn normal_on_the_surface_of_a_cube() {
        let c = Shape::cube();
        let points = vec![
            Tuple::point(1.0, 0.5, -0.8),
            Tuple::point(-1.0, -0.2, 0.9),
            Tuple::point(-0.4, 1.0, -0.1),
            Tuple::point(0.3, -1.0, -0.7),
            Tuple::point(-0.6, 0.3, 1.0),
            Tuple::point(0.4, 0.4, -1.0),
            Tuple::point(1.0, 1.0, 1.0),
            Tuple::point(-1.0, -1.0, -1.0),
        ];
        let normals = vec![
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
            Tuple::vector(0.0, 1.0, 0.0),
            Tuple::vector(0.0, -1.0, 0.0),
            Tuple::vector(0.0, 0.0, 1.0),
            Tuple::vector(0.0, 0.0, -1.0),
            Tuple::vector(1.0, 0.0, 0.0),
            Tuple::vector(-1.0, 0.0, 0.0),
        ];

        for i in 0..8 {
            let p = points[i];
            let n = normals[i];
            let normal = local_normal_at(&c, &p);
            assert_eq!(normal, n);
        }
    }
}
use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::tuple::Tuple;

/// Represents a ray in 3D space.
///
/// A ray is defined by an origin point and a direction vector. It can be used
/// to trace paths through a scene, such as for ray tracing algorithms.
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Tuple,
    pub direction: Tuple,
}

impl Ray {
    /// Constructs a new `Ray`.
    ///
    /// # Arguments
    ///
    /// * `origin` - A `Tuple` representing the origin point of the ray.
    /// * `direction` - A `Tuple` representing the direction vector of the ray.
    ///
    /// # Returns
    ///
    /// A new instance of `Ray`.
    pub fn new(origin: Tuple, direction: Tuple) -> Ray {
        Ray { origin, direction }
    }

    /// Calculates the position of a point along the ray at a given distance.
    ///
    /// # Arguments
    ///
    /// * `t` - The distance from the ray's origin to the point.
    ///
    /// # Returns
    ///
    /// A `Tuple` representing the point at distance `t` along the ray.
    pub fn position(&self, t: f64) -> Tuple {
        self.origin.add(&self.direction.multiply(t))
    }

    /// Transforms the ray by a given matrix.
    ///
    /// This method applies a transformation to the ray's origin and direction,
    /// effectively moving, scaling, or rotating the ray in 3D space.
    ///
    /// # Arguments
    ///
    /// * `matrix` - A `Matrix` representing the transformation to apply.
    ///
    /// # Returns
    ///
    /// A new `Ray` instance representing the transformed ray.
    pub fn transform(&self, matrix: &Matrix) -> Ray {
        Ray {
            origin: matrix.multiply_tuple(&self.origin),
            direction: matrix.multiply_tuple(&self.direction),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::matrix::Matrix;
    use super::Ray;
    use crate::tuple::Tuple;
    use crate::color::Color;
    use crate::raytracer::canvas::Canvas;
    use crate::raytracer::light::{Light, lighting};
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::object::plane::Plane;
    use crate::raytracer::scene::Scene;
    use crate::raytracer::object::db::get_object;

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
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::new();
        s.transform = Matrix::translate(0.0, 0.0, 1.0);
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let i = super::Intersection { t: 5.0, object: id, u: 0.0, v: 0.0};
        let xs = vec![i];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert!(comps.over_point.z < -crate::EPSILON / 2.0);
        assert!(comps.point.z > comps.over_point.z);
    }

    #[test]
    #[ignore]
    fn test_render() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let ray_origin = Tuple::point(0.0, 0.0, -5.0);
        let wall_z = 10.0;
        let wall_size = 7.0;
        let canvas_pixels = 100;
        let pixel_size = wall_size / canvas_pixels as f64;
        let half = wall_size / 2.0;
        let mut canvas = Canvas::new(canvas_pixels, canvas_pixels);
        let color = Color::new(1.0, 0.0, 0.0);
        let mut s = Sphere::new();
        s.transform = Matrix::scale(1.0, 0.5, 1.0);
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let object = get_object(id);

        for y in 0..canvas_pixels {
            let world_y = half - pixel_size * y as f64;
            for x in 0..canvas_pixels {
                let world_x = -half + pixel_size * x as f64;
                let position = Tuple::point(world_x, world_y, wall_z);
                let r = Ray::new(ray_origin.clone(), position.subtract(&ray_origin).normalize());
                let xs = object.intersect(&r);
                if let Some(_i) = Scene::hit(&xs) {
                    canvas.write_pixel(x, y, color);
                }
            }
        }

        canvas.write_to_file("canvas.png",1 );
    }

    #[test]
    #[ignore]
    fn test_render2() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let ray_origin = Tuple::point(0.0, 0.0, -5.0);
        let wall_z = 10.0;
        let wall_size = 7.0;
        let canvas_pixels = 300;
        let pixel_size = wall_size / canvas_pixels as f64;
        let half = wall_size / 2.0;
        let mut canvas = Canvas::new(canvas_pixels, canvas_pixels);
        let mut s = Sphere::new();
        //s.transform = Matrix::scale(1.0, 0.5, 1.0);
        s.material.pattern = Pattern::solid(Color::new(1.0, 0.2, 1.0), Matrix::identity(4));
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let object = get_object(id);

        let light_position = Tuple::point(-10.0, 10.0, -10.0);
        let light_color = Color::new(1.0, 1.0, 1.0);
        let light = Light::new_point_light(light_position, light_color);

        for y in 0..canvas_pixels {
            let world_y = half - pixel_size * y as f64;
            for x in 0..canvas_pixels {
                let world_x = -half + pixel_size * x as f64;
                let position = Tuple::point(world_x, world_y, wall_z);
                let r = Ray::new(ray_origin.clone(), position.subtract(&ray_origin).normalize());
                let xs = object.intersect(&r);
                if let Some(hit) = Scene::hit(&xs) {
                    let point = r.position(hit.t);
                    let hit_object = get_object(hit.object);
                    let normal = hit_object.normal_at(&point, hit);
                    let eye = r.direction.negate();
                    let color = lighting(hit.object, &light, &point, &eye, &normal, false);
                    canvas.write_pixel(x, y, color);
                }
            }
        }

        canvas.write_to_file("canvas.png",1 );
    }

    #[test]
    fn test_prepare_computations() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let i = super::Intersection { t: 4.0, object: id, u: 0.0, v: 0.0};
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
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let s = Sphere::new();
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let xs = vec![super::Intersection { t: 1.0, object: id, u: 0.0, v: 0.0}];
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
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let s = Plane::new();
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let r = Ray::new(Tuple::point(0.0, 1.0, -1.0), Tuple::vector(0.0, -2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
        let xs = vec![super::Intersection { t: 2.0_f64.sqrt(), object: id, u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert_eq!(comps.reflectv, Tuple::vector(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
    }

    #[test]
    fn finding_n1_and_n2_at_various_intersections() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let mut a = Sphere::glass_sphere();
        a.transform = Matrix::scale(2.0, 2.0, 2.0);
        a.material.refractive_index = 1.5;
        w.add_object(Arc::new(a));
        let aid = w.ids[0];

        let mut b = Sphere::glass_sphere();
        b.transform = Matrix::translate(0.0, 0.0, -0.25);
        b.material.refractive_index = 2.0;
        w.add_object(Arc::new(b));
        let bid = w.ids[1];

        let mut c = Sphere::glass_sphere();
        c.transform = Matrix::translate(0.0, 0.0, 0.25);
        c.material.refractive_index = 2.5;
        w.add_object(Arc::new(c));
        let cid = w.ids[2];

        let r = Ray::new(Tuple::point(0.0, 0.0, -4.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![
            super::Intersection { t: 2.0, object: aid, u: 0.0, v: 0.0},
            super::Intersection { t: 2.75, object: bid, u: 0.0, v: 0.0},
            super::Intersection { t: 3.25, object: cid, u: 0.0, v: 0.0},
            super::Intersection { t: 4.75, object: bid, u: 0.0, v: 0.0},
            super::Intersection { t: 5.25, object: cid, u: 0.0, v: 0.0},
            super::Intersection { t: 6.0, object: aid, u: 0.0, v: 0.0},
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
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let mut s = Sphere::glass_sphere();
        s.transform = Matrix::translate(0.0, 0.0, 1.0);
        w.add_object(Arc::new(s));
        let id = w.ids[0];
        let i = super::Intersection { t: 5.0, object: id, u: 0.0, v: 0.0};
        let xs = vec![i];
        let comps = xs[0].prepare_computations(&r, &xs);
        assert!(comps.under_point.z > crate::EPSILON / 2.0);
        assert!(comps.point.z < comps.under_point.z);
    }
}
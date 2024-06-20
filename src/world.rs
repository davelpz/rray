#[allow(dead_code)]

pub mod world {
    use crate::color::Color;
    use crate::shape::Shape;
    use crate::light::{lighting, Light};
    use crate::matrix::Matrix;
    use crate::pattern::Pattern;
    use crate::ray::{Computations, hit, Ray};
    use crate::ray::Intersection;
    use crate::tuple::Tuple;

    #[derive(Debug, PartialEq, Clone)]
    pub struct World {
        pub objects: Vec<Shape>,
        pub light: Light,
    }

    impl World {
        pub fn new(light: Light) -> World {
            World {
                objects: Vec::new(),
                light,
            }
        }

        pub fn default_world() -> World {
            let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
            let mut s1 = Shape::sphere();
            s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
            s1.material.diffuse = 0.7;
            s1.material.specular = 0.2;
            let mut s2 = Shape::sphere();
            s2.transform = Matrix::scale(0.5, 0.5, 0.5);
            World {
                light,
                objects: vec![s1, s2],
            }
        }

        pub fn intersect(&self, r: &Ray) -> Vec<Intersection> {
            let mut xs: Vec<Intersection> = Vec::new();
            for object in &self.objects {
                let mut obj_xs = object.intersect(r);
                xs.append(&mut obj_xs);
            }
            xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
            xs
        }

        pub fn shade_hit(&self, comps: &Computations, remaining: usize) -> Color {
            // TODO: support multiple light sources, loop through all lights and sum the results
            let surface = lighting(
                     &comps.object,
                     &self.light,
                     &comps.over_point,
                     &comps.eyev,
                     &comps.normalv,
                     self.is_shadowed(&comps.over_point));

            let reflected = self.reflected_color(comps, remaining);
            let refracted = self.refracted_color(comps, remaining);

            let material = &comps.object.material;

            if material.reflective > 0.0 && material.transparency > 0.0 {
                let reflectance = comps.schlick();
                return surface.add(&reflected.multiply(reflectance)).add(&refracted.multiply(1.0 - reflectance));
            } else {
                surface.add(&reflected).add(&refracted)
            }
        }

        pub fn color_at(&self, r: &Ray, remaining: usize) -> Color {
            let xs = self.intersect(r);
            if let Some(hit) = xs.iter().find(|x| x.t >= 0.0) {
                let comps = hit.prepare_computations(r,&xs);
                self.shade_hit(&comps, remaining)
            } else {
                Color::new(0.0, 0.0, 0.0)
            }
        }

        pub fn is_shadowed(&self, point: &Tuple) -> bool {
            let v = self.light.position - *point;
            let distance = v.magnitude();
            let direction = v.normalize();
            let r = Ray::new(*point, direction);
            let intersections = self.intersect(&r);
            let h = hit(&intersections);
            match h {
                Some(hit) => hit.t < distance,
                None => false
            }
        }

        pub fn reflected_color(&self, comps: &Computations, remaining: usize) -> Color {
            if remaining <= 0 || comps.object.material.reflective == 0.0 {
                return Color::new(0.0, 0.0, 0.0);
            }

            let reflect_ray = Ray::new(comps.over_point, comps.reflectv);
            let color = self.color_at(&reflect_ray, remaining - 1);
            color * comps.object.material.reflective
        }

        pub fn refracted_color(&self, comps: &Computations, remaining: usize) -> Color {
            if remaining <= 0 || comps.object.material.transparency == 0.0 {
                return Color::new(0.0, 0.0, 0.0);
            }

            // Snell's Law
            // sin(theta_i) / sin(theta_t) = n1 / n2
            // find the ratio of the first index of refraction to the second
            let n_ratio = comps.n1 / comps.n2;
            // cos(theta_i) is the same as the dot product of the two vectors
            let cos_i = comps.eyev.dot(&comps.normalv);
            // find sin(theta_t)^2 via trigonometric identity
            let sin2_t = n_ratio.powi(2) * (1.0 - cos_i.powi(2));
            if sin2_t > 1.0 { // total internal reflection
                return Color::new(0.0, 0.0, 0.0);
            }
            // find cos(theta_t) via trigonometric identity
            let cos_t = (1.0 - sin2_t).sqrt();
            // compute the direction of the refracted ray
            let direction = comps.normalv * (n_ratio * cos_i - cos_t) - comps.eyev * n_ratio;
            // create the refracted ray
            let refract_ray = Ray::new(comps.under_point, direction);
            // find the color of the refracted ray, making sure to multiply
            // by the transparency value to account for any opacity
            self.color_at(&refract_ray, remaining - 1) * comps.object.material.transparency
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::light::Light;
    use crate::matrix::Matrix;
    use crate::pattern::{Pattern, PatternType};
    use crate::tuple::Tuple;
    use super::world::World;
    use crate::shape::Shape;
    use crate::ray::{Intersection, Ray};

    #[test]
    fn creating_a_world() {
        let w = World::new(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        assert_eq!(w.objects.len(), 0);
        assert_eq!(w.light, Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn the_default_world() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        let w = World::default_world();
        assert_eq!(w.light, light);
        assert_eq!(w.objects[0], s1);
        assert_eq!(w.objects[1], s2);
    }

    #[test]
    fn intersect_a_world_with_a_ray() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = w.intersect(&r);
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].t, 4.0);
        assert_eq!(xs[1].t, 4.5);
        assert_eq!(xs[2].t, 5.5);
        assert_eq!(xs[3].t, 6.0);
    }

    #[test]
    fn shading_an_intersection() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = &w.objects[0];
        let xs = vec![Intersection{t: 4.0, object: shape}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = World::default_world();
        w.light = Light::new_point_light(Tuple::point(0.0, 0.25, 0.0), Color::new(1.0, 1.0, 1.0));
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = &w.objects[1];
        let xs = vec![Intersection{t: 0.5, object: shape}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575));
    }

    #[test]
    fn shade_hit_is_given_an_intersection_in_shadow() {
        let mut w = World::default_world();
        w.light = Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let s1 = Shape::sphere();
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::translate(0.0, 0.0, 10.0);
        w.objects = vec![s1, s2];
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: &w.objects[1]}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn the_color_when_a_ray_misses() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn the_color_when_a_ray_hits() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn the_color_with_an_intersection_behind_the_ray() {
        let mut w = World::default_world();
        w.objects[0].material.ambient = 1.0;
        w.objects[1].material.ambient = 1.0;
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.75), Tuple::vector(0.0, 0.0, -1.0));
        let c = w.color_at(&r,5);
        let obj_color = match w.objects[1].material.pattern.pattern_type {
            PatternType::Solid(c) => c,
            _ => Color::new(0.0, 0.0, 0.0)
        };
        assert_eq!(c, obj_color);
    }

    #[test]
    fn there_is_no_shadow_when_nothing_is_collinear_with_point_and_light() {
        let w = World::default_world();
        let p = Tuple::point(0.0, 10.0, 0.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn the_shadow_when_an_object_is_between_the_point_and_the_light() {
        let w = World::default_world();
        let p = Tuple::point(10.0, -10.0, 10.0);
        assert_eq!(w.is_shadowed(&p), true);
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_light() {
        let w = World::default_world();
        let p = Tuple::point(-20.0, 20.0, -20.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_point() {
        let w = World::default_world();
        let p = Tuple::point(-2.0, 2.0, -2.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn reflected_color_for_the_a_nonreflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        let w = World {
            light,
            objects: vec![s1, s2],
        };

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 1.0, object: &w.objects[1]}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps, 5);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn reflected_color_for_a_reflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        let mut s3 = Shape::plane();
        s3.material.reflective = 0.5;
        s3.transform = Matrix::translate(0.0, -1.0, 0.0);

        let w = World {
            light,
            objects: vec![s1, s2, s3],
        };

        let r = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps, 5);
        assert_eq!(color, Color::new(0.190332201495133, 0.23791525186891627, 0.14274915112134975));
    }

    #[test]
    fn shade_hit_for_a_reflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        let mut s3 = Shape::plane();
        s3.material.reflective = 0.5;
        s3.transform = Matrix::translate(0.0, -1.0, 0.0);

        let w = World {
            light,
            objects: vec![s1, s2, s3],
        };

        let r = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.shade_hit(&comps,5);
        assert_eq!(color, Color::new(0.8767572837020907, 0.924340334075874, 0.8291742333283075));
    }

    #[test]
    fn color_at_with_mutually_reflective_surfaces() {
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0));
        let mut lower = Shape::plane();
        lower.material.reflective = 1.0;
        lower.transform = Matrix::translate(0.0, -1.0, 0.0);
        let mut upper = Shape::plane();
        upper.material.reflective = 1.0;
        upper.transform = Matrix::translate(0.0, 1.0, 0.0);
        let w = World {
            light,
            objects: vec![lower, upper],
        };
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, Color::new(11.4,11.4,11.4));
    }

    #[test]
    fn reflected_color_at_the_maximum_recursive_depth() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        let mut s3 = Shape::plane();
        s3.material.reflective = 0.5;
        s3.transform = Matrix::translate(0.0, -1.0, 0.0);

        let w = World {
            light,
            objects: vec![s1, s2, s3],
        };

        let r = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps,0);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_with_an_opaque_surface() {
        let w = World::default_world();
        let shape = &w.objects[0];
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: shape}, Intersection{t: 6.0, object: shape}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_at_the_maximum_recursive_depth() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.transparency = 1.0;
        s1.material.refractive_index = 1.5;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        let w = World {
            light,
            objects: vec![s1, s2],
        };

        let shape = &w.objects[0];
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: shape}, Intersection{t: 6.0, object: shape}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,0);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.transparency = 1.0;
        s1.material.refractive_index = 1.5;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        let w = World {
            light,
            objects: vec![s1, s2],
        };

        let shape = &w.objects[0];
        let r = Ray::new(Tuple::point(0.0, 0.0, 2_f64.sqrt()/2.0), Tuple::vector(0.0, 1.0, 0.0));
        let xs = vec![Intersection{t: -2_f64.sqrt()/2.0, object: shape}, Intersection{t: 2_f64.sqrt()/2.0, object: shape}];
        let comps = xs[1].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_with_a_recracted_ray() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::test();
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.ambient = 1.0;
        let mut s2 = Shape::sphere();
        s2.material.transparency = 1.0;
        s2.material.refractive_index = 1.5;
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        let w = World {
            light,
            objects: vec![s1, s2],
        };

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.1), Tuple::vector(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection{t: -0.9899, object: &w.objects[0]},
            Intersection{t: -0.4899, object: &w.objects[1]},
            Intersection{t: 0.4899, object: &w.objects[1]},
            Intersection{t: 0.9899, object: &w.objects[0]}
        ];
        let comps = xs[2].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.9988745506795582, 0.04721898034382347));
    }

    #[test]
    fn shade_hit_with_a_transparent_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::test();
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        let mut floor = Shape::plane();
        floor.material.transparency = 0.5;
        floor.material.refractive_index = 1.5;
        floor.transform = Matrix::translate(0.0, -1.0, 0.0);
        let mut s3 = Shape::sphere();
        s3.transform = Matrix::translate(0.0, -3.5, -0.5);
        s3.material.pattern = Pattern::solid(Color::new(1.0, 0.0, 0.0), Matrix::identity(4));
        s3.material.ambient = 0.5;
        let w = World {
            light,
            objects: vec![s1, s2, floor, s3],
        };

        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![
            Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]}
        ];
        let comps = xs[0].prepare_computations(&ray, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.93642, 0.68642, 0.68642));
    }

    #[test]
    fn shade_hit_with_a_reflective_transparent_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut s1 = Shape::sphere();
        s1.material.pattern = Pattern::test();
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Shape::sphere();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        let mut floor = Shape::plane();
        floor.transform = Matrix::translate(0.0, -1.0, 0.0);
        floor.material.reflective = 0.5;
        floor.material.transparency = 0.5;
        floor.material.refractive_index = 1.5;
        let mut s3 = Shape::sphere();
        s3.transform = Matrix::translate(0.0, -3.5, -0.5);
        s3.material.pattern = Pattern::solid(Color::new(1.0, 0.0, 0.0), Matrix::identity(4));
        s3.material.ambient = 0.5;
        let w = World {
            light,
            objects: vec![s1, s2, floor, s3],
        };

        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection::new(2.0_f64.sqrt(), &w.objects[2])];
        let comps = xs[0].prepare_computations(&ray, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.9259077639258646, 0.6864251822976762, 0.6764160604069138));
    }
}
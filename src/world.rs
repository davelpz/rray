#[allow(dead_code)]

pub mod world {
    use crate::color::color::Color;
    use crate::shape::shape::Shape;
    use crate::light::light::{lighting, Light};
    use crate::matrix::matrix::Matrix;
    use crate::pattern::pattern::Pattern;
    use crate::ray::ray::{Computations, hit, Ray};
    use crate::ray::ray::Intersection;
    use crate::tuple::tuple::Tuple;

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

            surface.add(&reflected)
        }

        pub fn color_at(&self, r: &Ray, remaining: usize) -> Color {
            let xs = self.intersect(r);
            if let Some(hit) = xs.iter().find(|x| x.t >= 0.0) {
                let comps = hit.prepare_computations(r);
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
    }
}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::light::light::Light;
    use crate::matrix::matrix::Matrix;
    use crate::pattern::pattern::{Pattern, PatternType};
    use crate::tuple::tuple::Tuple;
    use super::world::World;
    use crate::shape::shape::Shape;
    use crate::ray::ray::{Intersection, Ray};

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
        let i = Intersection{t: 4.0, object: shape};
        let comps = i.prepare_computations(&r);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = World::default_world();
        w.light = Light::new_point_light(Tuple::point(0.0, 0.25, 0.0), Color::new(1.0, 1.0, 1.0));
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = &w.objects[1];
        let i = Intersection{t: 0.5, object: shape};
        let comps = i.prepare_computations(&r);
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
        let i = Intersection{t: 4.0, object: &w.objects[1]};
        let comps = i.prepare_computations(&r);
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
        let i = Intersection{t: 1.0, object: &w.objects[1]};
        let comps = i.prepare_computations(&r);
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
        let i = Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]};
        let comps = i.prepare_computations(&r);
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
        let i = Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]};
        let comps = i.prepare_computations(&r);
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
        let i = Intersection{t: 2.0_f64.sqrt(), object: &w.objects[2]};
        let comps = i.prepare_computations(&r);
        let color = w.reflected_color(&comps,0);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }
}
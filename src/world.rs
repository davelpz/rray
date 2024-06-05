#[allow(dead_code)]

pub mod world {
    use crate::color::color::Color;
    use crate::shape::shape::Sphere;
    use crate::light::light::{lighting, PointLight};
    use crate::matrix::matrix::Matrix;
    use crate::ray::ray::{Computations, Ray};
    use crate::ray::ray::Intersection;
    use crate::tuple::tuple::Tuple;

    #[derive(Debug, PartialEq, Clone)]
    pub struct World {
        pub objects: Vec<Sphere>,
        pub light: PointLight,
    }

    impl World {
        pub fn new(light: PointLight) -> World {
            World {
                objects: Vec::new(),
                light,
            }
        }

        pub fn default_world() -> World {
            let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(-10.0, 10.0, -10.0));
            let mut s1 = Sphere::new();
            s1.material.color = Color::new(0.8, 1.0, 0.6);
            s1.material.diffuse = 0.7;
            s1.material.specular = 0.2;
            let mut s2 = Sphere::new();
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

        pub fn shade_hit(&self, comps: &Computations) -> Color {
            // TODO: support multiple light sources, loop through all lights and sum the results
            lighting(&comps.object.material,
                     &self.light,
                     &comps.point,
                     &comps.eyev,
                     &comps.normalv,
                     false)
        }

        pub fn color_at(&self, r: &Ray) -> Color {
            let xs = self.intersect(r);
            if let Some(hit) = xs.iter().find(|x| x.t >= 0.0) {
                let comps = hit.prepare_computations(r);
                self.shade_hit(&comps)
            } else {
                Color::new(0.0, 0.0, 0.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::color::Color;
    use crate::light::light::PointLight;
    use crate::matrix::matrix::Matrix;
    use crate::tuple::tuple::Tuple;
    use super::world::World;
    use crate::shape::shape::Sphere;
    use crate::ray::ray::{Intersection, Ray};

    #[test]
    fn creating_a_world() {
        let w = World::new(PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.0, 0.0)));
        assert_eq!(w.objects.len(), 0);
        assert_eq!(w.light, PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.0, 0.0)));
    }

    #[test]
    fn the_default_world() {
        let light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(-10.0, 10.0, -10.0));
        let mut s1 = Sphere::new();
        s1.material.color = Color::new(0.8, 1.0, 0.6);
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        let mut s2 = Sphere::new();
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
        let c = w.shade_hit(&comps);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = World::default_world();
        w.light = PointLight::new(Color::new(1.0, 1.0, 1.0), Tuple::point(0.0, 0.25, 0.0));
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = &w.objects[1];
        let i = Intersection{t: 0.5, object: shape};
        let comps = i.prepare_computations(&r);
        let c = w.shade_hit(&comps);
        assert_eq!(c, Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575));
    }

    #[test]
    fn the_color_when_a_ray_misses() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = w.color_at(&r);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn the_color_when_a_ray_hits() {
        let w = World::default_world();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let c = w.color_at(&r);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn the_color_with_an_intersection_behind_the_ray() {
        let mut w = World::default_world();
        w.objects[0].material.ambient = 1.0;
        w.objects[1].material.ambient = 1.0;
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.75), Tuple::vector(0.0, 0.0, -1.0));
        let c = w.color_at(&r);
        assert_eq!(c, w.objects[1].material.color);
    }
}
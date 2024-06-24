use std::sync::{Arc};
use crate::color::Color;
use crate::matrix::Matrix;
use crate::tuple::Tuple;
use crate::raytracer::computations::Computations;
use crate::raytracer::material::pattern::Pattern;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::light::{Light, lighting};
use crate::raytracer::object::Object;
use crate::raytracer::object::sphere::Sphere;
use crate::raytracer::ray::{hit, Ray};
use crate::raytracer::object::db::{get_object, replace_sentinel};

pub struct Scene {
    pub light: Light,
    pub ids: Vec<usize>,
}

impl Scene {
    pub fn new(light: Light) -> Scene {
        Scene {
            light,
            ids: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Arc<dyn Object + Send>) -> usize {
        let id = object.get_id();
        replace_sentinel(id, object);
        self.ids.push(id);
        id
    }

    #[allow(dead_code)]
    pub fn get_object_at_index(&self, index: usize) -> Arc<dyn Object + Send> {
        get_object(self.ids[index])
    }

    #[allow(dead_code)]
    pub fn default_scene() -> Scene {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut scene = Scene::new(light);
        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        scene.add_object(Arc::new(s1));
        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        scene.add_object(Arc::new(s2));
        scene
    }

    pub fn intersect(&self, r: &Ray) -> Vec<Intersection> {
        let mut xs: Vec<Intersection> = Vec::new();
        for i in &self.ids {
            let object = get_object(*i);
            let mut obj_xs = object.intersect(r);
            xs.append(&mut obj_xs);
        }
        xs.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        xs
    }

    pub fn shade_hit(&self, comps: &Computations, remaining: usize) -> Color {
        // TODO: support multiple light sources, loop through all lights and sum the results
        let surface = lighting(
            comps.object,
            &self.light,
            &comps.over_point,
            &comps.eyev,
            &comps.normalv,
            self.is_shadowed(&comps.over_point));

        let reflected = self.reflected_color(comps, remaining);
        let refracted = self.refracted_color(comps, remaining);

        let object = get_object(comps.object);
        let material = object.get_material();

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
        let object = get_object(comps.object);
        if remaining <= 0 || object.get_material().reflective == 0.0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let reflect_ray = Ray::new(comps.over_point, comps.reflectv);
        let color = self.color_at(&reflect_ray, remaining - 1);
        color * object.get_material().reflective
    }

    pub fn refracted_color(&self, comps: &Computations, remaining: usize) -> Color {
        let object = get_object(comps.object);
        if remaining <= 0 || object.get_material().transparency == 0.0 {
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
        self.color_at(&refract_ray, remaining - 1) * object.get_material().transparency
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::color::Color;
    use crate::matrix::Matrix;
    use crate::raytracer::intersection::Intersection;
    use crate::raytracer::light::Light;
    use crate::raytracer::material::pattern::{Pattern, PatternType};
    use crate::raytracer::object::plane::Plane;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::ray::Ray;
    use crate::raytracer::scene::{Scene};
    use crate::tuple::Tuple;

    #[test]
    fn creating_a_world() {
        let w = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        assert_eq!(w.ids.len(), 0);
        assert_eq!(w.light, Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn intersect_a_world_with_a_ray() {
        let w = Scene::default_scene();
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
        let w = Scene::default_scene();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = w.get_object_at_index(0);
        let xs = vec![Intersection{t: 4.0, object: shape.get_id()}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = Scene::default_scene();
        w.light = Light::new_point_light(Tuple::point(0.0, 0.25, 0.0), Color::new(1.0, 1.0, 1.0));
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = w.get_object_at_index(1);
        let xs = vec![Intersection{t: 0.5, object: shape.get_id()}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575));
    }

    #[test]
    fn shade_hit_is_given_an_intersection_in_shadow() {
        let mut w = Scene::new(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let s1 = Sphere::new();
        let mut s2 = Sphere::new();
        s2.transform = Matrix::translate(0.0, 0.0, 10.0);
        w.add_object(Arc::new(s1));
        w.add_object(Arc::new(s2));
        let s2_id = w.ids[1];
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: s2_id}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn the_color_when_a_ray_misses() {
        let w = Scene::default_scene();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn the_color_when_a_ray_hits() {
        let w = Scene::default_scene();
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn the_color_with_an_intersection_behind_the_ray() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);
        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.ambient = 1.0;
        w.add_object(Arc::new(s1));
        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        w.add_object(Arc::new(s2));

        let inner = w.get_object_at_index(1);
        let inner_material = inner.get_material().clone();
        let obj_color = match inner_material.pattern.pattern_type {
            PatternType::Solid(c) => c,
            _ => Color::new(0.0, 0.0, 0.0)
        };

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.75), Tuple::vector(0.0, 0.0, -1.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, obj_color);
    }

    #[test]
    fn there_is_no_shadow_when_nothing_is_collinear_with_point_and_light() {
        let w = Scene::default_scene();
        let p = Tuple::point(0.0, 10.0, 0.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn the_shadow_when_an_object_is_between_the_point_and_the_light() {
        let w = Scene::default_scene();
        let p = Tuple::point(10.0, -10.0, 10.0);
        assert_eq!(w.is_shadowed(&p), true);
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_light() {
        let w = Scene::default_scene();
        let p = Tuple::point(-20.0, 20.0, -20.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_point() {
        let w = Scene::default_scene();
        let p = Tuple::point(-2.0, 2.0, -2.0);
        assert_eq!(w.is_shadowed(&p), false);
    }

    #[test]
    fn reflected_color_for_the_a_nonreflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        w.add_object(Arc::new(s1));

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        w.add_object(Arc::new(s2));
        let s2_id = w.ids[1];

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 1.0, object: s2_id}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps, 5);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn reflected_color_for_a_reflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);
        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        w.add_object(Arc::new(s1));

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        w.add_object(Arc::new(s2));

        let mut s3 = Plane::new();
        s3.material.reflective = 0.5;
        s3.transform = Matrix::translate(0.0, -1.0, 0.0);
        w.add_object(Arc::new(s3));
        let s3_id = w.ids[2];

        let r = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: s3_id}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps, 5);
        assert_eq!(color, Color::new(0.190332201495133, 0.23791525186891627, 0.14274915112134975));
    }

    #[test]
    fn shade_hit_for_a_reflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        w.add_object(Arc::new(s1));

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        w.add_object(Arc::new(s2));

        let mut s3 = Plane::new();
        s3.material.reflective = 0.5;
        s3.transform = Matrix::translate(0.0, -1.0, 0.0);
        w.add_object(Arc::new(s3));
        let s3_id = w.ids[2];

        let r = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: s3_id}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.shade_hit(&comps,5);
        assert_eq!(color, Color::new(0.8767572837020907, 0.924340334075874, 0.8291742333283075));
    }

    #[test]
    fn color_at_with_mutually_reflective_surfaces() {
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut lower = Plane::new();
        lower.material.reflective = 1.0;
        lower.transform = Matrix::translate(0.0, -1.0, 0.0);
        w.add_object(Arc::new(lower));

        let mut upper = Plane::new();
        upper.material.reflective = 1.0;
        upper.transform = Matrix::translate(0.0, 1.0, 0.0);
        w.add_object(Arc::new(upper));

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 1.0, 0.0));
        let c = w.color_at(&r,5);
        assert_eq!(c, Color::new(11.4,11.4,11.4));
    }

    #[test]
    fn reflected_color_at_the_maximum_recursive_depth() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        w.add_object(Arc::new(s1));

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        s2.material.ambient = 1.0;
        w.add_object(Arc::new(s2));

        let mut s3 = Plane::new();
        s3.material.reflective = 0.5;
        s3.transform = Matrix::translate(0.0, -1.0, 0.0);
        w.add_object(Arc::new(s3));
        let s3_id = w.ids[2];

        let r = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: s3_id}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps,0);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_with_an_opaque_surface() {
        let w = Scene::default_scene();
        let shape = w.get_object_at_index(0);
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: shape.get_id()}, Intersection{t: 6.0, object: shape.get_id()}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_at_the_maximum_recursive_depth() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.transparency = 1.0;
        s1.material.refractive_index = 1.5;
        w.add_object(Arc::new(s1));
        let s1_id = w.ids[0];

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        w.add_object(Arc::new(s2));

        let shape = s1_id;
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: shape}, Intersection{t: 6.0, object: shape}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,0);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::solid(Color::new(0.8, 1.0, 0.6), Matrix::identity(4));
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.transparency = 1.0;
        s1.material.refractive_index = 1.5;
        w.add_object(Arc::new(s1));
        let s1_id = w.ids[0];

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        w.add_object(Arc::new(s2));

        let shape = s1_id;
        let r = Ray::new(Tuple::point(0.0, 0.0, 2_f64.sqrt()/2.0), Tuple::vector(0.0, 1.0, 0.0));
        let xs = vec![Intersection{t: -2_f64.sqrt()/2.0, object: shape}, Intersection{t: 2_f64.sqrt()/2.0, object: shape}];
        let comps = xs[1].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_with_a_recracted_ray() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::test();
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        s1.material.ambient = 1.0;
        w.add_object(Arc::new(s1));
        let s1_id = w.ids[0];

        let mut s2 = Sphere::new();
        s2.material.transparency = 1.0;
        s2.material.refractive_index = 1.5;
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        w.add_object(Arc::new(s2));
        let s2_id = w.ids[1];

        let r = Ray::new(Tuple::point(0.0, 0.0, 0.1), Tuple::vector(0.0, 1.0, 0.0));
        let xs = vec![
            Intersection{t: -0.9899, object: s1_id},
            Intersection{t: -0.4899, object: s2_id},
            Intersection{t: 0.4899, object: s2_id},
            Intersection{t: 0.9899, object: s1_id}
        ];
        let comps = xs[2].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.9988745506795582, 0.04721898034382347));
    }

    #[test]
    fn shade_hit_with_a_transparent_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::test();
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        w.add_object(Arc::new(s1));

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        w.add_object(Arc::new(s2));

        let mut floor = Plane::new();
        floor.material.transparency = 0.5;
        floor.material.refractive_index = 1.5;
        floor.transform = Matrix::translate(0.0, -1.0, 0.0);
        w.add_object(Arc::new(floor));
        let floor_id = w.ids[2];

        let mut s3 = Sphere::new();
        s3.transform = Matrix::translate(0.0, -3.5, -0.5);
        s3.material.pattern = Pattern::solid(Color::new(1.0, 0.0, 0.0), Matrix::identity(4));
        s3.material.ambient = 0.5;
        w.add_object(Arc::new(s3));

        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![
            Intersection{t: 2.0_f64.sqrt(), object: floor_id}
        ];
        let comps = xs[0].prepare_computations(&ray, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.93642, 0.68642, 0.68642));
    }

    #[test]
    fn shade_hit_with_a_reflective_transparent_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new(light);

        let mut s1 = Sphere::new();
        s1.material.pattern = Pattern::test();
        s1.material.diffuse = 0.7;
        s1.material.specular = 0.2;
        w.add_object(Arc::new(s1));

        let mut s2 = Sphere::new();
        s2.transform = Matrix::scale(0.5, 0.5, 0.5);
        w.add_object(Arc::new(s2));

        let mut floor = Plane::new();
        floor.transform = Matrix::translate(0.0, -1.0, 0.0);
        floor.material.reflective = 0.5;
        floor.material.transparency = 0.5;
        floor.material.refractive_index = 1.5;
        w.add_object(Arc::new(floor));
        let floor_id = w.ids[2];

        let mut s3 = Sphere::new();
        s3.transform = Matrix::translate(0.0, -3.5, -0.5);
        s3.material.pattern = Pattern::solid(Color::new(1.0, 0.0, 0.0), Matrix::identity(4));
        s3.material.ambient = 0.5;
        w.add_object(Arc::new(s3));

        let ray = Ray::new(Tuple::point(0.0, 0.0, -3.0), Tuple::vector(0.0, -2.0_f64.sqrt()/2.0, 2.0_f64.sqrt()/2.0));
        let xs = vec![Intersection::new(2.0_f64.sqrt(), floor_id)];
        let comps = xs[0].prepare_computations(&ray, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.9259077639258646, 0.6864251822976762, 0.6764160604069138));
    }
}
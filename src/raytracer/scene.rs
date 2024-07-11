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
use crate::raytracer::ray::Ray;
use crate::raytracer::object::db::{get_object, add_object};

/// Represents a scene in a ray tracing engine.
///
/// A `Scene` is a collection of lights and objects that can be rendered. It stores references to lights
/// and objects within the scene, allowing for operations such as adding new lights or objects, and
/// performing ray intersections to determine color and shading at different points.
///
/// # Fields
///
/// * `light` - A vector of `Light` instances representing the light sources in the scene.
/// * `ids` - A vector of `usize` values, each corresponding to the unique identifier of an object within the scene.
pub struct Scene {
    pub light: Vec<Light>,
    pub ids: Vec<usize>,
}

/// The `Scene` struct implementation.
///
/// This implementation provides the functionality to manage a scene in a ray tracing engine.
/// It includes methods for adding lights and objects to the scene, retrieving objects by index,
/// creating a default scene setup, calculating intersections with rays, determining the color at a point,
/// shading intersections, checking for shadows, and handling reflected and refracted colors.
///
/// # Examples
///
/// Creating a new scene and adding elements to it:
///
/// ```
/// let mut scene = Scene::new();
/// let light = Light::new_point_light(Tuple::point(-10, 10, -10), Color::new(1, 1, 1));
/// scene.add_light(light);
/// let sphere = Arc::new(Sphere::new());
/// scene.add_object(sphere);
/// ```
///
/// Calculating the color at a ray intersection:
///
/// ```
/// let ray = Ray::new(Tuple::point(0, 0, -5), Tuple::vector(0, 0, 1));
/// let color = scene.color_at(&ray, 5);
/// ```
impl Scene {
    pub fn new() -> Scene {
        Scene {
            light: Vec::new(),
            ids: Vec::new(),
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.light.push(light);
    }

    pub fn add_object(&mut self, object: Arc<dyn Object + Send>) -> usize {
        let id = object.get_id();
        add_object(object);
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
        let mut scene = Scene::new();
        scene.add_light(light);
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

    /// Returns a list of intersections for a ray and the objects in the scene
    /// The intersections are sorted by distance from the ray origin
    /// The intersections are returned in world space
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

    /// Calculates the color at a given ray's intersection point within the scene.
    ///
    /// This function determines the color of the scene as seen from the ray's perspective. It first finds
    /// all intersections of the ray with objects in the scene. If there are no intersections, the function
    /// returns the color black, indicating that the ray does not hit anything and thus points to the background.
    /// If there are intersections, it finds the closest one where the intersection point is in front of the ray
    /// (i.e., has a positive `t` value). It then calculates the color at this intersection point by considering
    /// various factors such as the object's material, the lighting, and whether the point is in shadow.
    /// This function also accounts for recursive reflections by using the `remaining` parameter, which
    /// decreases with each recursive call to prevent infinite recursion.
    ///
    /// # Arguments
    ///
    /// * `r` - The ray for which to calculate the color.
    /// * `remaining` - The number of times the function can still be recursively called, used to limit
    ///   the recursion depth for reflections and refractions.
    ///
    /// # Returns
    ///
    /// The color at the intersection point closest to the ray origin, or black if the ray intersects no objects.
    pub fn color_at(&self, r: &Ray, remaining: usize) -> Color {
        let xs = self.intersect(r);
        if let Some(hit) = xs.iter().find(|x| x.t >= 0.0) {
            let comps = hit.prepare_computations(r,&xs);
            self.shade_hit(&comps, remaining)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    }

    /// Calculates the color at a point of intersection in the scene, considering various lighting effects.
    ///
    /// This method combines the Phong reflection model with additional handling for reflective and
    /// transparent materials. It first calculates the direct illumination from light sources using the
    /// Phong model. Then, it adds the effects of reflection and refraction, if applicable, based on the
    /// material properties of the intersected object. For materials that are both reflective and transparent,
    /// the Fresnel effect is approximated using Schlick's approximation to blend the reflected and refracted
    /// colors based on the viewing angle.
    ///
    /// # Arguments
    ///
    /// * `comps` - The precomputed information about the intersection, including the point of intersection,
    ///   the normal at the intersection, and other relevant data for shading.
    /// * `remaining` - The recursion limit for reflective and refractive color calculations. This prevents
    ///   infinite recursion by gradually reducing the contribution of reflected and refracted light in
    ///   successive reflections/refractions.
    ///
    /// # Returns
    ///
    /// Returns the color at the intersection point, which includes contributions from direct light sources,
    /// reflected light, and refracted light, as determined by the material properties of the intersected object.
    pub fn shade_hit(&self, comps: &Computations, remaining: usize) -> Color {
        let mut surface = Color::new(0.0, 0.0, 0.0);
        for light in &self.light {
            let light_color= self.shade_hit_light(comps, light);
            surface = surface.add(&light_color);
        }

        let reflected = self.reflected_color(comps, remaining);
        let refracted = self.refracted_color(comps, remaining);

        let object = get_object(comps.object);
        let material = object.get_material();

        if material.reflective > 0.0 && material.transparency > 0.0 {
            let reflectance = comps.schlick();
            surface.add(&reflected.multiply(reflectance)).add(&refracted.multiply(1.0 - reflectance))
        } else {
            surface.add(&reflected).add(&refracted)
        }
    }

    /// Returns the color of intersection point for a single light source
    fn shade_hit_light(&self, comps: &Computations, light: &Light) -> Color {
        lighting(
            comps.object,
            light,
            &comps.over_point,
            &comps.eyev,
            &comps.normalv,
            self.is_shadowed(&comps.over_point, light))
    }

    /// Determines if a given point is in shadow relative to a specific light source.
    ///
    /// This method checks if the point is shadowed by casting a ray from the point to the light source.
    /// It calculates the vector from the point to the light's position, then normalizes this vector to
    /// get the direction. A ray is then created from the point in this direction. The method finds all
    /// intersections of this ray with objects in the scene. If there is an intersection between the point
    /// and the light source (i.e., if the closest intersection's `t` value is less than the distance to the
    /// light source), the point is considered to be in shadow, and the method returns `true`. Otherwise,
    /// it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `point` - A reference to the `Tuple` representing the point in space to check for shadow.
    /// * `light` - A reference to the `Light` object representing the light source.
    ///
    /// # Returns
    ///
    /// Returns `true` if the point is in shadow relative to the light source; otherwise, returns `false`.
    pub fn is_shadowed(&self, point: &Tuple, light: &Light) -> bool {
        let v = light.position - *point;
        let distance = v.magnitude();
        let direction = v.normalize();
        let r = Ray::new(*point, direction);
        let intersections = self.intersect(&r);
        let h = Scene::hit(&intersections);
        match h {
            Some(hit) => hit.t < distance,
            None => false
        }
    }

    /// Returns the intersection with the smallest non-negative t value
    /// If all intersections have negative t values, return None
    pub fn hit(xs: &Vec<Intersection>) -> Option<&Intersection> {
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

    /// Calculates the color contribution from reflected light at a point of intersection.
    ///
    /// This function determines the color contribution from reflected light based on the material's
    /// reflective property and the remaining number of allowed reflections. If the recursion limit
    /// (`remaining`) is reached or the material is not reflective (`reflective == 0.0`), it returns black,
    /// indicating no reflected light contribution. Otherwise, it calculates a reflection ray based on the
    /// intersection's over point and the reflection vector, then recursively calls `color_at` to determine
    /// the color seen in the reflection. This color is then scaled by the material's reflective property to
    /// simulate the intensity of the reflected light.
    ///
    /// # Arguments
    ///
    /// * `comps` - The precomputed information about the intersection, including the point of intersection,
    ///   the normal at the intersection, and other relevant data for shading.
    /// * `remaining` - The recursion limit for reflective color calculations. This prevents infinite recursion
    ///   by gradually reducing the contribution of reflected light in successive reflections.
    ///
    /// # Returns
    ///
    /// The color contribution from reflected light at the intersection point.
    pub fn reflected_color(&self, comps: &Computations, remaining: usize) -> Color {
        let object = get_object(comps.object);
        if remaining <= 0 || object.get_material().reflective == 0.0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let reflect_ray = Ray::new(comps.over_point, comps.reflectv);
        let color = self.color_at(&reflect_ray, remaining - 1);
        color * object.get_material().reflective
    }

    /// Calculates the color contribution from refracted light at an intersection point.
    ///
    /// This method applies Snell's Law to compute the direction of the refracted ray and then determines
    /// the color seen through the transparent material. It accounts for the possibility of total internal
    /// reflection and the material's transparency level. If the material is opaque or the recursion limit
    /// for refracted color calculations is reached, it returns black, indicating no refracted light contribution.
    ///
    /// # Arguments
    ///
    /// * `comps` - The precomputed information about the intersection, including the point of intersection,
    ///   the normal at the intersection, and other relevant data for shading, such as the indices of refraction.
    /// * `remaining` - The recursion limit for refracted color calculations. This prevents infinite recursion
    ///   by gradually reducing the contribution of refracted light in successive refractions.
    ///
    /// # Returns
    ///
    /// The color contribution from refracted light at the intersection point, or black if the material is opaque
    /// or the recursion limit is reached.
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
    fn test_hit() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let s = Sphere::new();
        w.add_object(Arc::new(s));
        let id = w.ids[0];

        let i1 = super::Intersection { t: 1.0, object: id, u: 0.0, v: 0.0};
        let i2 = super::Intersection { t: 2.0, object: id, u: 0.0, v: 0.0};
        let xs = vec![i1, i2];
        let i = Scene::hit(&xs);
        assert_eq!(i.unwrap().t, 1.0);

        let i1 = super::Intersection { t: -1.0, object: id, u: 0.0, v: 0.0};
        let i2 = super::Intersection { t: 1.0, object: id, u: 0.0, v: 0.0};
        let xs = vec![i1, i2];
        let i = Scene::hit(&xs);
        assert_eq!(i.unwrap().t, 1.0);

        let i1 = super::Intersection { t: -2.0, object: id, u: 0.0, v: 0.0};
        let i2 = super::Intersection { t: -1.0, object: id, u: 0.0, v: 0.0};
        let xs = vec![i1, i2];
        let i = Scene::hit(&xs);
        assert_eq!(i, None);

        let i1 = super::Intersection { t: 5.0, object: id, u: 0.0, v: 0.0};
        let i2 = super::Intersection { t: 7.0, object: id, u: 0.0, v: 0.0};
        let i3 = super::Intersection { t: -3.0, object: id, u: 0.0, v: 0.0};
        let i4 = super::Intersection { t: 2.0, object: id, u: 0.0, v: 0.0};
        let xs = vec![i1, i2, i3, i4];
        let i = Scene::hit(&xs);
        assert_eq!(i.unwrap().t, 2.0);

        let xs = vec![];
        let i = Scene::hit(&xs);
        assert_eq!(i, None);
    }

    #[test]
    fn creating_a_world() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
        assert_eq!(w.ids.len(), 0);
        assert_eq!(w.light[0], Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0)));
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
        let xs = vec![Intersection{t: 4.0, object: shape.get_id(), u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.38066, 0.47583, 0.2855));
    }

    #[test]
    fn shading_an_intersection_from_the_inside() {
        let mut w = Scene::default_scene();
        w.light.remove(0);
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.25, 0.0), Color::new(1.0, 1.0, 1.0)));
        let r = Ray::new(Tuple::point(0.0, 0.0, 0.0), Tuple::vector(0.0, 0.0, 1.0));
        let shape = w.get_object_at_index(1);
        let xs = vec![Intersection{t: 0.5, object: shape.get_id(), u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.9049844720832575, 0.9049844720832575, 0.9049844720832575));
    }

    #[test]
    fn shade_hit_is_given_an_intersection_in_shadow() {
        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        let s1 = Sphere::new();
        let mut s2 = Sphere::new();
        s2.transform = Matrix::translate(0.0, 0.0, 10.0);
        w.add_object(Arc::new(s1));
        w.add_object(Arc::new(s2));
        let s2_id = w.ids[1];
        let r = Ray::new(Tuple::point(0.0, 0.0, 5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: s2_id, u: 0.0, v: 0.0}];
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
        let mut w = Scene::new();
        w.add_light(light);
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
        assert_eq!(w.is_shadowed(&p, &w.light[0]), false);
    }

    #[test]
    fn the_shadow_when_an_object_is_between_the_point_and_the_light() {
        let w = Scene::default_scene();
        let p = Tuple::point(10.0, -10.0, 10.0);
        assert_eq!(w.is_shadowed(&p, &w.light[0]), true);
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_light() {
        let w = Scene::default_scene();
        let p = Tuple::point(-20.0, 20.0, -20.0);
        assert_eq!(w.is_shadowed(&p, &w.light[0]), false);
    }

    #[test]
    fn there_is_no_shadow_when_an_object_is_behind_the_point() {
        let w = Scene::default_scene();
        let p = Tuple::point(-2.0, 2.0, -2.0);
        assert_eq!(w.is_shadowed(&p, &w.light[0]), false);
    }

    #[test]
    fn reflected_color_for_the_a_nonreflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
        let xs = vec![Intersection{t: 1.0, object: s2_id, u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps, 5);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn reflected_color_for_a_reflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);
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
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: s3_id, u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps, 5);
        assert_eq!(color, Color::new(0.190332201495133, 0.23791525186891627, 0.14274915112134975));
    }

    #[test]
    fn shade_hit_for_a_reflective_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: s3_id, u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.shade_hit(&comps,5);
        assert_eq!(color, Color::new(0.8767572837020907, 0.924340334075874, 0.8291742333283075));
    }

    #[test]
    fn color_at_with_mutually_reflective_surfaces() {
        let light = Light::new_point_light(Tuple::point(0.0, 0.0, 0.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
        let mut w = Scene::new();
        w.add_light(light);

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
        let xs = vec![Intersection{t: 2.0_f64.sqrt(), object: s3_id, u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let color = w.reflected_color(&comps,0);
        assert_eq!(color, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_with_an_opaque_surface() {
        let w = Scene::default_scene();
        let shape = w.get_object_at_index(0);
        let r = Ray::new(Tuple::point(0.0, 0.0, -5.0), Tuple::vector(0.0, 0.0, 1.0));
        let xs = vec![Intersection{t: 4.0, object: shape.get_id(), u:0.0, v:0.0}, Intersection{t: 6.0, object: shape.get_id(), u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_at_the_maximum_recursive_depth() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
        let xs = vec![Intersection{t: 4.0, object: shape,u:0.0, v:0.0}, Intersection{t: 6.0, object: shape, u: 0.0, v: 0.0}];
        let comps = xs[0].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,0);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_under_total_internal_reflection() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
        let xs = vec![Intersection{t: -2_f64.sqrt()/2.0, object: shape, u:0.0, v:0.0}, Intersection{t: 2_f64.sqrt()/2.0, object: shape, u:0.0, v:0.0}];
        let comps = xs[1].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn refracted_color_with_a_recracted_ray() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
            Intersection{t: -0.9899, object: s1_id, u: 0.0, v: 0.0},
            Intersection{t: -0.4899, object: s2_id, u: 0.0, v: 0.0},
            Intersection{t: 0.4899, object: s2_id, u: 0.0, v: 0.0},
            Intersection{t: 0.9899, object: s1_id, u: 0.0, v: 0.0}
        ];
        let comps = xs[2].prepare_computations(&r, &xs);
        let c = w.refracted_color(&comps,5);
        assert_eq!(c, Color::new(0.0, 0.9988745506795582, 0.04721898034382347));
    }

    #[test]
    fn shade_hit_with_a_transparent_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
            Intersection{t: 2.0_f64.sqrt(), object: floor_id, u: 0.0, v: 0.0},
        ];
        let comps = xs[0].prepare_computations(&ray, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.93642, 0.68642, 0.68642));
    }

    #[test]
    fn shade_hit_with_a_reflective_transparent_material() {
        let light = Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0));
        let mut w = Scene::new();
        w.add_light(light);

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
        let xs = vec![Intersection::new(2.0_f64.sqrt(), floor_id, 0.0, 0.0)];
        let comps = xs[0].prepare_computations(&ray, &xs);
        let c = w.shade_hit(&comps,5);
        assert_eq!(c, Color::new(0.9259077639258646, 0.6864251822976762, 0.6764160604069138));
    }
}
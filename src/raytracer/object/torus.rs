use crate::matrix::Matrix;
use crate::raytracer::intersection::Intersection;
use crate::raytracer::material::Material;
use crate::raytracer::object::db::get_next_id;
use crate::raytracer::object::{AABB, normal_to_world, Object, world_to_object};
use crate::raytracer::ray::Ray;
use crate::tuple::Tuple;
use roots::{find_roots_quartic, Roots};

pub struct Torus {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub minor_radius: f64,
    pub transform: Matrix,
    pub material: Material,
}

impl Torus {
    pub fn new(minor_radius: f64) -> Self {
        Torus {
            id: get_next_id(),
            parent_id: None,
            minor_radius,
            transform: Matrix::identity(4),
            material: Material::default(),
        }
    }

    fn local_intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let ox = ray.origin.x;
        let oy = ray.origin.y;
        let oz = ray.origin.z;
        let dx = ray.direction.x;
        let dy = ray.direction.y;
        let dz = ray.direction.z;

        let r = self.minor_radius;
        let r_sq = r * r;

        let sum_d_sq = dx * dx + dy * dy + dz * dz;
        let e = ox * ox + oy * oy + oz * oz - r_sq + 1.0;  // R^2 is 1^2 = 1
        let f = ray.origin.dot(&ray.direction);
        let four = 4.0;  // 4 * 1^2 = 4

        let a4 = sum_d_sq * sum_d_sq;
        let a3 = 4.0 * sum_d_sq * f;
        let a2 = 2.0 * sum_d_sq * e + 4.0 * f * f - four * (dx * dx + dy * dy);
        let a1 = 4.0 * e * f - 2.0 * four * (ox * dx + oy * dy);
        let a0 = e * e - four * (ox * ox + oy * oy);

        // Find the roots of the quartic equation
        let roots = find_roots_quartic(a4, a3, a2, a1, a0);

        let mut intersections = vec![];
        match roots {
            Roots::No(_) => {},
            Roots::One([t]) => {
                if t > 0.0 {
                    intersections.push(Intersection { t, object: self.id, u: 0.0, v: 0.0 });
                }
            }
            Roots::Two(ts) => {
                for t in ts {
                    if t > 0.0 {
                        intersections.push(Intersection { t, object: self.id, u: 0.0, v: 0.0 });
                    }
                }
            }
            Roots::Three(ts) => {
                for t in ts {
                    if t > 0.0 {
                        intersections.push(Intersection { t, object: self.id, u: 0.0, v: 0.0 });
                    }
                }
            }
            Roots::Four(ts) => {
                for t in ts {
                    if t > 0.0 {
                        intersections.push(Intersection { t, object: self.id, u: 0.0, v: 0.0 });
                    }
                }
            }
        }

        intersections
    }

    pub fn local_normal_at(&self, local_point: &Tuple, _hit: &Intersection) -> Tuple {
        let sum_squared = local_point.x * local_point.x + local_point.y * local_point.y + local_point.z * local_point.z;
        let param_squared = 1.0 + self.minor_radius * self.minor_radius;  // Major radius is fixed at 1.0

        let normal = Tuple::vector(
            4.0 * local_point.x * (sum_squared - param_squared),
            4.0 * local_point.y * (sum_squared - param_squared),
            4.0 * local_point.z * (sum_squared - param_squared + 2.0),
        );
        normal.normalize()
    }
}

impl Object for Torus {
    fn intersect(&self, ray: &Ray) -> Vec<Intersection> {
        let trans_ray = ray.transform(&self.transform.inverse());
        self.local_intersect(&trans_ray)
    }

    fn normal_at(&self, world_point: &Tuple, _hit: &Intersection) -> Tuple {
        let local_point = world_to_object(self.id, world_point);
        let local_normal = self.local_normal_at(&local_point, _hit);
        normal_to_world(self.id, &local_normal)
    }

    fn get_transform(&self) -> &Matrix {
        &self.transform
    }

    fn get_material(&self) -> &Material {
        &self.material
    }

    fn set_transform(&mut self, transform: Matrix) {
        self.transform = transform;
    }

    fn set_material(&mut self, material: Material) {
        self.material = material;
    }

    fn debug_string(&self) -> String {
        format!("Torus: transform: {:?}, material: {:?}", self.transform, self.material)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_parent_id(&self) -> Option<usize> {
        self.parent_id
    }

    fn set_parent_id(&mut self, id: usize) {
        self.parent_id = Some(id);
    }

    fn get_aabb(&self) -> AABB {
        let r = self.minor_radius;
        let min = Tuple::point(-1.0 - r, -1.0 - r, -r); // Major radius is 1.0
        let max = Tuple::point(1.0 + r, 1.0 + r, r);    // Major radius is 1.0
        AABB { min, max }
    }

    fn includes(&self, object_id: usize) -> bool {
        self.id == object_id
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::matrix::Matrix;
    use crate::raytracer::camera::Camera;
    use crate::raytracer::light::Light;
    use crate::raytracer::material::Material;
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::plane::Plane;
    use crate::raytracer::object::torus::Torus;
    use crate::raytracer::scene::Scene;
    use crate::tuple::Tuple;

    #[test]
    #[ignore]
    fn test_render_torus() {
        for i in (0..190).step_by(10) {
            let angle = i as f64 * std::f64::consts::PI / 180.0;
            render_torus(i, angle);
        }
    }

    fn render_torus(frame: usize, angle: f64) {
        use crate::color::Color;

        let mut c = Camera::new(800, 400, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 2.0, -5.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));
        //w.add_light(Light::new_point_light(Tuple::point(0.0, 0.0, -10.0), Color::new(0.5, 0.5, 0.5) * 2.0));

        let mut floor = Plane::new();
        floor.transform = Matrix::translate(0.0, 0.0, 0.0);
        floor.material.pattern = Pattern::stripe(Pattern::solid(Color::new(1.0, 0.5, 0.5), Matrix::identity(4)),
                                                 Pattern::solid(Color::new(0.5, 1.0, 0.5), Matrix::identity(4)),
                                                 Matrix::scale(0.1, 0.1, 0.1).multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0)));
        floor.material.specular = 0.0;
        //w.add_object(Arc::new(floor));

        let mut left_wall = Plane::new();
        left_wall.material.pattern = Pattern::gradient(Pattern::solid(Color::new(1.0, 0.5, 0.5), Matrix::identity(4)),
                                                       Pattern::solid(Color::new(0.5, 1.0, 0.5), Matrix::identity(4)),
                                                       Matrix::identity(4)
                                                           .multiply(&Matrix::translate(124.0, 124.0, 124.0)
                                                               .multiply(&Matrix::scale(7.0, 7.0, 7.0))
                                                           ));
        left_wall.transform = Matrix::identity(4)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / -4.0))
            .multiply(&Matrix::translate(0.0, 0.0, 5.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
        ;
        left_wall.material.specular = 0.0;
        w.add_object(Arc::new(left_wall));

        let mut right_wall = Plane::new();
        right_wall.transform = Matrix::identity(4)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0))
            .multiply(&Matrix::translate(0.0, 0.0, 5.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
        ;
        right_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        right_wall.material.specular = 0.0;
        w.add_object(Arc::new(right_wall));

        let mut material = Material::default();
        material.pattern = Pattern::solid(Color::new(0.302, 0.71, 0.98), Matrix::identity(4));
        let mut torus = Torus::new(0.25 * 1.0);
        torus.material = material.clone();
        torus.transform = Matrix::identity(4)
            .multiply(&Matrix::translate(0.0, 0.0, 0.0))
            //.multiply(&Matrix::scale(0.75, 0.75, 0.75))
            .multiply(&Matrix::rotate_x(angle))
        ;
        w.add_object(Arc::new(torus));

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file(format!("output{}.png", frame).as_str(),1 );
    }
}
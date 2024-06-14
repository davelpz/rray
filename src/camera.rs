#[allow(dead_code)]

pub mod camera {
    use std::sync::{Arc, Mutex};
    use crate::canvas::canvas::Canvas;
    use crate::matrix::matrix::Matrix;
    use crate::ray::ray::Ray;
    use crate::tuple::tuple::Tuple;
    use crate::world::world::World;
    use indicatif::ProgressBar;
    use crate::camera::pixel_coordinates;
    use rayon::iter::ParallelBridge;
    use rayon::prelude::ParallelIterator;

    #[derive(Debug, Clone)]
    pub struct Camera {
        pub hsize: usize,
        pub vsize: usize,
        pub field_of_view: f64,
        pub transform: Matrix,
        pub pixel_size: f64,
        pub half_width: f64,
        pub half_height: f64,
    }

    impl Camera {
        pub fn new(hsize: usize, vsize: usize, field_of_view: f64) -> Camera {
            let half_view = (field_of_view / 2.0).tan();
            let aspect = hsize as f64 / vsize as f64;
            let half_width;
            let half_height;
            if aspect >= 1.0 {
                half_width = half_view;
                half_height = half_view / aspect;
            } else {
                half_width = half_view * aspect;
                half_height = half_view;
            }
            let pixel_size = (half_width * 2.0) / hsize as f64;
            Camera {
                hsize,
                vsize,
                field_of_view,
                transform: Matrix::identity(4),
                pixel_size,
                half_width,
                half_height,
            }
        }

        pub fn ray_for_pixel(&self, px: usize, py: usize) -> Ray {
            // the offset from the edge of the canvas to the pixel's center
            let xoffset = (px as f64 + 0.5) * self.pixel_size;
            let yoffset = (py as f64 + 0.5) * self.pixel_size;

            // the untransformed coordinates of the pixel in world space
            // (remember that the camera looks toward -z, so +x is to the *left*)
            let world_x = self.half_width - xoffset;
            let world_y = self.half_height - yoffset;

            // using the camera matrix, transform the canvas point and the origin,
            // and then compute the ray's direction vector
            // (remember that the canvas is at z=-1)
            let inverse_transform = self.transform.inverse();
            let pixel = inverse_transform.multiply_tuple(&Tuple::point(world_x, world_y, -1.0));
            let origin = inverse_transform.multiply_tuple(&Tuple::point(0.0, 0.0, 0.0));
            let direction = pixel.subtract(&origin).normalize();
            Ray::new(origin, direction)
        }

        pub fn render(&self, world: &World) -> Canvas {
            let image = Arc::new(Mutex::new(Canvas::new(self.hsize, self.vsize)));
            let bar = ProgressBar::new((self.vsize * self.hsize) as u64);
            let iter = pixel_coordinates(self.vsize, self.hsize).par_bridge();
            iter.for_each(|(x, y)| {
                let ray = self.ray_for_pixel(x, y);
                let color = world.color_at(&ray);
                let mut image = image.lock().unwrap();
                image.write_pixel(x, y, color);
                drop(image); // unlock the mutex
                bar.inc(1);
            });
            bar.finish();
            Arc::try_unwrap(image).unwrap().into_inner().unwrap()
        }

        pub fn render_sequential(&self, world: &World) -> Canvas {
            let mut image = Canvas::new(self.hsize, self.vsize);
            let bar = ProgressBar::new((self.vsize * self.hsize) as u64);
            for (x, y) in pixel_coordinates(self.vsize, self.hsize) {
                let ray = self.ray_for_pixel(x, y);
                let color = world.color_at(&ray);
                image.write_pixel(x, y, color);
                bar.inc(1);
            }
            bar.finish();
            image
        }
    }
}

pub fn pixel_coordinates(vsize: usize, hsize: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..vsize).flat_map(move |y| (0..hsize).map(move |x| (x, y)))
}

#[cfg(test)]
mod tests {
    pub const EPSILON: f64 = 0.00001;

    use crate::matrix::matrix::Matrix;
    use crate::pattern::pattern::Pattern;
    use super::camera::Camera;
    use crate::tuple::tuple::Tuple;

    #[test]
    fn test_camera() {
        let c = Camera::new(160, 120, std::f64::consts::PI / 2.0);
        assert_eq!(c.hsize, 160);
        assert_eq!(c.vsize, 120);
        assert_eq!(c.field_of_view, std::f64::consts::PI / 2.0);
        assert_eq!(c.transform, Matrix::identity(4));

        let c = Camera::new(200, 125, std::f64::consts::PI / 2.0);
        assert_eq!((c.pixel_size - 0.01).abs() < EPSILON, true);

        let c = Camera::new(125, 200, std::f64::consts::PI / 2.0);
        assert_eq!((c.pixel_size - 0.01).abs() < EPSILON, true);
    }

    #[test]
    fn test_ray_for_pixel() {
        let c = Camera::new(201, 101, std::f64::consts::PI / 2.0);
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, Tuple::point(0.0, 0.0, 0.0));
        assert_eq!(r.direction, Tuple::vector(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_ray_for_pixel_corner() {
        let c = Camera::new(201, 101, std::f64::consts::PI / 2.0);
        let r = c.ray_for_pixel(0, 0);
        assert_eq!(r.origin, Tuple::point(0.0, 0.0, 0.0));
        assert_eq!(r.direction, Tuple::vector(0.66519, 0.33259, -0.66851));
    }

    #[test]
    fn test_ray_for_pixel_transformed() {
        let mut c = Camera::new(201, 101, std::f64::consts::PI / 2.0);
        c.transform = Matrix::rotate_y(std::f64::consts::PI / 4.0).multiply(&Matrix::translate(0.0, -2.0, 5.0));
        let r = c.ray_for_pixel(100, 50);
        assert_eq!(r.origin, Tuple::point(0.0, 2.0, -5.0));
        assert_eq!(r.direction, Tuple::vector(2f64.sqrt() / 2.0, 0.0, -2f64.sqrt() / 2.0));
    }

    #[test]
    fn test_pixel_coordinates() {
        let rows = 1;
        let columns = 1;
        let pixels = super::pixel_coordinates(rows, columns).collect::<Vec<_>>();
        assert_eq!(pixels, vec![(0, 0)]);

        let rows = 2;
        let columns = 1;
        let pixels = super::pixel_coordinates(rows, columns).collect::<Vec<_>>();
        assert_eq!(pixels, vec![(0, 0),(0, 1)]);

        let rows = 1;
        let columns = 2;
        let pixels = super::pixel_coordinates(rows, columns).collect::<Vec<_>>();
        assert_eq!(pixels, vec![(0, 0),(1, 0)]);

        let rows = 2;
        let columns = 2;
        let pixels = super::pixel_coordinates(rows, columns).collect::<Vec<_>>();
        assert_eq!(pixels, vec![(0, 0),(1, 0),(0, 1),(1, 1)]);

        let rows = 3;
        let columns = 2;
        let pixels = super::pixel_coordinates(rows, columns).collect::<Vec<_>>();
        assert_eq!(pixels, vec![(0, 0),(1, 0),(0, 1),(1, 1),(0, 2),(1, 2)]);
    }

    #[test]
    #[ignore]
    fn test_render_chap7() {
        use crate::color::color::Color;
        use crate::light::light::Light;
        use crate::shape::shape::Shape;
        use crate::world::world::World;

        let mut c = Camera::new(256, 256, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 1.5, -5.0);
        let to = Tuple::point(0.0, 1.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = World::new(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut floor = Shape::sphere();
        floor.transform = Matrix::scale(10.0, 0.01, 10.0);
        floor.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        floor.material.specular = 0.0;
        w.objects.push(floor);

        let mut left_wall = Shape::sphere();
        left_wall.transform = Matrix::translate(0.0, 0.0, 5.0)
            .multiply(&Matrix::rotate_y(-std::f64::consts::PI / 4.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
            .multiply(&Matrix::scale(10.0, 0.01, 10.0));
        left_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        left_wall.material.specular = 0.0;
        w.objects.push(left_wall);

        let mut right_wall = Shape::sphere();
        right_wall.transform = Matrix::translate(0.0, 0.0, 5.0)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
            .multiply(&Matrix::scale(10.0, 0.01, 10.0));
        right_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        right_wall.material.specular = 0.0;
        w.objects.push(right_wall);

        let mut middle = Shape::sphere();
        middle.transform = Matrix::translate(-0.5, 1.0, 0.5);
        middle.material.pattern = Pattern::solid(Color::new(0.1, 1.0, 0.5), Matrix::identity(4));
        middle.material.diffuse = 0.7;
        middle.material.specular = 0.3;
        w.objects.push(middle);

        let mut right = Shape::sphere();
        right.transform = Matrix::translate(1.5, 0.5, -0.5).multiply(&Matrix::scale(0.5, 0.5, 0.5));
        right.material.pattern = Pattern::solid(Color::new(0.5, 1.0, 0.1), Matrix::identity(4));
        right.material.diffuse = 0.7;
        right.material.specular = 0.3;
        w.objects.push(right);

        let mut left = Shape::sphere();
        left.transform = Matrix::translate(-1.5, 0.33, -0.75).multiply(&Matrix::scale(0.33, 0.33, 0.33));
        left.material.pattern = Pattern::solid(Color::new(1.0, 0.8, 0.1), Matrix::identity(4));
        left.material.diffuse = 0.7;
        left.material.specular = 0.3;
        w.objects.push(left);

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file("canvas.png");
    }

    #[test]
    #[ignore]
    fn test_render_chap9() {
        use crate::color::color::Color;
        use crate::light::light::Light;
        use crate::shape::shape::Shape;
        use crate::world::world::World;

        let mut c = Camera::new(400, 200, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 1.5, -5.0);
        let to = Tuple::point(0.0, 1.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = World::new(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut floor = Shape::plane();
        floor.transform = Matrix::translate(0.0, 0.0, 0.0);
        floor.material.pattern = Pattern::stripe(Pattern::solid(Color::new(1.0, 0.5, 0.5), Matrix::identity(4)),
                                                 Pattern::solid(Color::new(0.5, 1.0, 0.5), Matrix::identity(4)),
                                                 Matrix::scale(0.1, 0.1, 0.1).multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0)));
        floor.material.specular = 0.0;
        w.objects.push(floor);

        let mut left_wall = Shape::plane();
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
        w.objects.push(left_wall);

        let mut right_wall = Shape::plane();
        right_wall.transform = Matrix::identity(4)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0))
            .multiply(&Matrix::translate(0.0, 0.0, 5.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
        ;
        right_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        right_wall.material.specular = 0.0;
        w.objects.push(right_wall);

        let mut middle = Shape::sphere();
        middle.transform = Matrix::translate(-0.5, 1.0, 0.5);
        middle.material.pattern = Pattern::solid(Color::new(0.1, 1.0, 0.5), Matrix::identity(4));
        middle.material.diffuse = 0.7;
        middle.material.specular = 0.3;
        w.objects.push(middle);

        let mut right = Shape::sphere();
        right.transform = Matrix::translate(1.5, 0.5, -0.5).multiply(&Matrix::scale(0.5, 0.5, 0.5));
        right.material.pattern = Pattern::solid(Color::new(0.5, 1.0, 0.1), Matrix::identity(4));
        right.material.diffuse = 0.7;
        right.material.specular = 0.3;
        w.objects.push(right);

        let mut left = Shape::sphere();
        left.transform = Matrix::translate(-1.5, 0.33, -0.75).multiply(&Matrix::scale(0.33, 0.33, 0.33));
        left.material.pattern = Pattern::solid(Color::new(1.0, 0.8, 0.1), Matrix::identity(4));
        left.material.diffuse = 0.7;
        left.material.specular = 0.3;
        w.objects.push(left);

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file("canvas.png");
    }
}
use std::sync::{Arc, Mutex};
use crate::matrix::Matrix;
use crate::tuple::Tuple;
use crate::raytracer::canvas::Canvas;
use indicatif::ProgressBar;
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use crate::raytracer::ray::Ray;
use crate::raytracer::scene::Scene;

/// Represents a camera in the raytracer scene.
///
/// The camera is defined by its horizontal size (`hsize`), vertical size (`vsize`),
/// field of view (`field_of_view`), and a transformation matrix (`transform`) that
/// positions and orients the camera in the scene. The `pixel_size`, `half_width`,
/// and `half_height` are calculated based on the camera's field of view and aspect ratio.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    /// Constructs a new `Camera` with the given size and field of view.
    ///
    /// # Arguments
    ///
    /// * `hsize` - The horizontal size of the camera's view.
    /// * `vsize` - The vertical size of the camera's view.
    /// * `field_of_view` - The camera's field of view in radians.
    ///
    /// # Returns
    ///
    /// A new `Camera` instance.
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

    /// Calculates the ray from the camera to a specific pixel on the canvas.
    ///
    /// # Arguments
    ///
    /// * `px` - The x-coordinate of the pixel on the canvas.
    /// * `py` - The y-coordinate of the pixel on the canvas.
    ///
    /// # Returns
    ///
    /// A `Ray` instance representing the ray from the camera to the specified pixel.
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

    /// Renders the scene from the perspective of the camera.
    ///
    /// This method utilizes parallel processing to render the scene, improving performance
    /// for large images. It returns a `Canvas` that represents the rendered image.
    ///
    /// # Arguments
    ///
    /// * `scene` - A reference to the `Scene` that will be rendered.
    ///
    /// # Returns
    ///
    /// A `Canvas` instance representing the rendered image.
    pub fn render(&self, scene: &Scene) -> Canvas {
        let image = Arc::new(Mutex::new(Canvas::new(self.hsize, self.vsize)));
        let bar = ProgressBar::new((self.vsize * self.hsize) as u64);
        let iter = pixel_coordinates(self.vsize, self.hsize).par_bridge();
        iter.for_each(|(x, y)| {
            let ray = self.ray_for_pixel(x, y);
            let color = scene.color_at(&ray, 5);
            let mut image = image.lock().unwrap();
            image.write_pixel(x, y, color);
            drop(image); // unlock the mutex
            bar.inc(1);
        });
        bar.finish();
        Arc::try_unwrap(image).unwrap().into_inner().unwrap()
    }
}

/// Generates an iterator over the coordinates of each pixel in the canvas.
///
/// # Arguments
///
/// * `vsize` - The vertical size of the canvas.
/// * `hsize` - The horizontal size of the canvas.
///
/// # Returns
///
/// An iterator that yields tuples of (x, y) coordinates for each pixel.
pub fn pixel_coordinates(vsize: usize, hsize: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..vsize).flat_map(move |y| (0..hsize).map(move |x| (x, y)))
}

#[cfg(test)]
mod tests {
    use crate::EPSILON;
    use std::sync::Arc;
    use crate::matrix::Matrix;
    use crate::raytracer::light::Light;
    use crate::raytracer::material::pattern::Pattern;
    use crate::raytracer::object::plane::Plane;
    use crate::raytracer::object::sphere::Sphere;
    use crate::raytracer::scene::{Scene};
    use super::Camera;
    use crate::tuple::Tuple;

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
        use crate::color::Color;

        let mut c = Camera::new(256, 256, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 1.5, -5.0);
        let to = Tuple::point(0.0, 1.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut floor = Sphere::new();
        floor.transform = Matrix::scale(10.0, 0.01, 10.0);
        floor.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        floor.material.specular = 0.0;
        w.add_object(Arc::new(floor));

        let mut left_wall = Sphere::new();
        left_wall.transform = Matrix::translate(0.0, 0.0, 5.0)
            .multiply(&Matrix::rotate_y(-std::f64::consts::PI / 4.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
            .multiply(&Matrix::scale(10.0, 0.01, 10.0));
        left_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        left_wall.material.specular = 0.0;
        w.add_object(Arc::new(left_wall));

        let mut right_wall = Sphere::new();
        right_wall.transform = Matrix::translate(0.0, 0.0, 5.0)
            .multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / 2.0))
            .multiply(&Matrix::scale(10.0, 0.01, 10.0));
        right_wall.material.pattern = Pattern::solid(Color::new(1.0, 0.9, 0.9), Matrix::identity(4));
        right_wall.material.specular = 0.0;
        w.add_object(Arc::new(right_wall));

        let mut middle = Sphere::new();
        middle.transform = Matrix::translate(-0.5, 1.0, 0.5);
        middle.material.pattern = Pattern::solid(Color::new(0.1, 1.0, 0.5), Matrix::identity(4));
        middle.material.diffuse = 0.7;
        middle.material.specular = 0.3;
        w.add_object(Arc::new(middle));

        let mut right = Sphere::new();
        right.transform = Matrix::translate(1.5, 0.5, -0.5).multiply(&Matrix::scale(0.5, 0.5, 0.5));
        right.material.pattern = Pattern::solid(Color::new(0.5, 1.0, 0.1), Matrix::identity(4));
        right.material.diffuse = 0.7;
        right.material.specular = 0.3;
        w.add_object(Arc::new(right));

        let mut left = Sphere::new();
        left.transform = Matrix::translate(-1.5, 0.33, -0.75).multiply(&Matrix::scale(0.33, 0.33, 0.33));
        left.material.pattern = Pattern::solid(Color::new(1.0, 0.8, 0.1), Matrix::identity(4));
        left.material.diffuse = 0.7;
        left.material.specular = 0.3;
        w.add_object(Arc::new(left));

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file("canvas.png",1);
    }

    #[test]
    #[ignore]
    fn test_render_chap9() {
        use crate::color::Color;

        let mut c = Camera::new(400, 200, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 1.5, -5.0);
        let to = Tuple::point(0.0, 1.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = Scene::new();
        w.add_light(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));

        let mut floor = Plane::new();
        floor.transform = Matrix::translate(0.0, 0.0, 0.0);
        floor.material.pattern = Pattern::stripe(Pattern::solid(Color::new(1.0, 0.5, 0.5), Matrix::identity(4)),
                                                 Pattern::solid(Color::new(0.5, 1.0, 0.5), Matrix::identity(4)),
                                                 Matrix::scale(0.1, 0.1, 0.1).multiply(&Matrix::rotate_y(std::f64::consts::PI / 4.0)));
        floor.material.specular = 0.0;
        w.add_object(Arc::new(floor));

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

        let mut middle = Sphere::new();
        middle.transform = Matrix::translate(-0.5, 1.0, 0.5);
        middle.material.pattern = Pattern::solid(Color::new(0.1, 1.0, 0.5), Matrix::identity(4));
        middle.material.diffuse = 0.7;
        middle.material.specular = 0.3;
        w.add_object(Arc::new(middle));

        let mut right = Sphere::new();
        right.transform = Matrix::translate(1.5, 0.5, -0.5).multiply(&Matrix::scale(0.5, 0.5, 0.5));
        right.material.pattern = Pattern::solid(Color::new(0.5, 1.0, 0.1), Matrix::identity(4));
        right.material.diffuse = 0.7;
        right.material.specular = 0.3;
        w.add_object(Arc::new(right));

        let mut left = Sphere::new();
        left.transform = Matrix::translate(-1.5, 0.33, -0.75).multiply(&Matrix::scale(0.33, 0.33, 0.33));
        left.material.pattern = Pattern::solid(Color::new(1.0, 0.8, 0.1), Matrix::identity(4));
        left.material.diffuse = 0.7;
        left.material.specular = 0.3;
        w.add_object(Arc::new(left));

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file("canvas.png",1);
    }
}
/// The `raytracer` module serves as the foundation for a ray tracing engine.
///
/// This module orchestrates the core components of a ray tracing engine, including
/// objects, rays, intersections, materials, scenes, lighting, camera, and utilities
/// for loading object models and building scenes from YAML files. It also includes
/// a module for rendering scenes to a canvas.
///
/// Modules:
/// - `object`: Defines the geometric objects that can be rendered in the scene.
/// - `ray`: Represents rays that can intersect with objects in the scene.
/// - `intersection`: Handles the calculation and storage of intersections between rays and objects.
/// - `computations`: Provides utilities for calculating shading, lighting, and reflections.
/// - `material`: Defines the material properties of objects, such as color and reflectiveness.
/// - `scene`: Represents the collection of objects and lights that make up a scene to be rendered.
/// - `light`: Defines the light sources in the scene.
/// - `camera`: Manages the viewpoint from which the scene is rendered.
/// - `load_obj`: Utilities for loading object models from .obj files.
/// - `scene_builder_yaml`: Provides functionality for building scenes from YAML configuration files.
/// - `canvas`: A module for creating and manipulating the canvas on which scenes are rendered.

mod object;
mod ray;
mod intersection;
mod computations;
mod material;
mod scene;
mod light;
mod camera;
mod load_obj;
pub(crate) mod scene_builder_yaml;
mod canvas;
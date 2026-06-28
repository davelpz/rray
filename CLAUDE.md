# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build --release

# Run
./target/release/rray -W 800 -H 600 -s example1.yaml -o output.png -a 1

# Test all
cargo test

# Test a single test
cargo test test_intersect

# Test within a specific module
cargo test raytracer::scene::tests
```

CLI flags: `-W` (width), `-H` (height), `-s` (scene YAML), `-o` (output PNG), `-a` (anti-aliasing 1–5).

## Architecture

The raytracer follows "The Ray Tracer Challenge" by Jamis Buck. The top-level math primitives (`tuple.rs`, `color.rs`, `matrix.rs`) are used throughout `src/raytracer/`.

### Global Object Registry (`raytracer/object/db.rs`)

All objects live in a single global `GLOBAL_OBJECTS: Arc<Mutex<Vec<Arc<dyn Object>>>>`. An object's ID is its index in this vector. This design allows parent-child relationships in the scene graph without cyclic `Arc` references. The pattern for creating an object:
1. Call `get_next_id()` — reserves a slot with a `Sentinel` placeholder and returns the new ID.
2. Construct the object (storing the returned ID as `self.id`).
3. Call `add_object(arc)` — replaces the sentinel with the real object.

### Object Trait (`raytracer/object/mod.rs`)

All scene primitives implement `Object`. The trait splits intersection and normal calculation into two layers:
- `local_intersect` / `local_normal_at` — work in object-local space (each primitive implements these).
- `intersect` / `normal_at` — default implementations that apply the object's transform, call the local methods, then transform back to world space.

`world_to_object` and `normal_to_world` recursively walk the parent chain (via `get_parent_id`) to handle nested groups.

### Scene Graph and Primitives

- `object/group.rs` — groups children under a shared transform; stores child IDs.
- `object/csg.rs` — Constructive Solid Geometry (union, intersection, difference) over two child objects.
- `object/triangle.rs` + `smooth_triangle.rs` — used by `load_obj.rs` (tobj crate) for OBJ mesh import.
- `object/torus.rs` — uses the `roots` crate to solve the degree-4 polynomial for torus intersections.
- All other primitives: `sphere`, `plane`, `cube`, `cylinder`, `cone`.

### Rendering Pipeline

`scene_builder_yaml.rs` parses the YAML scene file and builds a `Scene` + `Camera`. The `Camera` fires rays through each pixel; `rayon` parallelizes this across pixels. For each ray, `Scene::color_at` finds intersections, picks the closest hit, calls `shade_hit` (Phong + Schlick approximation for reflective/transparent materials), and recursively traces reflected/refracted rays up to a fixed depth.

### Materials and Patterns (`raytracer/material/`)

`Material` holds Phong coefficients (`ambient`, `diffuse`, `specular`, `shininess`) plus `reflective`, `transparency`, and `refractive_index`. The `pattern` field is a `Pattern` enum covering: solid, stripe, gradient, ring, checker, blend, perturbed, noise (fastnoise-lite), and image (UV-mapped PNG via the `image` crate).

### Key Constants

`EPSILON = 0.00001` is defined in `src/main.rs` as `pub const` and used throughout for floating-point comparisons.

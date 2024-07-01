use std::sync::Arc;
use tobj::Mesh;
use crate::raytracer::material::Material;
use crate::raytracer::object::group::Group;
use crate::raytracer::object::triangle::Triangle;
use crate::tuple::Tuple;

fn get_faces(mesh: &Mesh) -> Vec<Vec<Tuple>> {
    let mut faces: Vec<Vec<Tuple>> = vec![];
    let mut next_face = 0;
    for face in 0..mesh.face_arities.len() {
        let end = next_face + mesh.face_arities[face] as usize;
        let face_indices = &mesh.indices[next_face..end];
        let mut face: Vec<Tuple> = vec![];
        //let mut vertex = 0;
        for f in face_indices {
            let x: f64 = mesh.positions[3 * *f as usize] as f64;
            let y: f64 = mesh.positions[3 * *f as usize + 1] as f64;
            let z: f64 = mesh.positions[3 * *f as usize + 2] as f64;
            //println!("face: {}, vertex: {},  x: {}, y: {}, z: {}", faces.len(), vertex, x, y, z);
            face.push(Tuple::point(x, y, z));
            //vertex += 1;
        }
        faces.push(face);

        next_face = end;
    }

    faces
}

fn convert_face_to_triangles(vertexes: &Vec<Tuple>) -> Vec<Triangle> {
    let mut triangles: Vec<Triangle> = vec![];

    for i in 1..vertexes.len() - 1 {
        triangles.push(Triangle::new(vertexes[0], vertexes[i], vertexes[i + 1]));
    }

    triangles
}

fn create_group(mesh: &Mesh, material: Material) -> Group {
    let mut group = Group::new();
    let faces: Vec<Vec<Tuple>> = get_faces(mesh);
    for f in faces.iter() {
        let triangles = convert_face_to_triangles(f);
        for mut t in triangles {
            t.material = material.clone();
            group.add_child(Arc::new(t));
        }
    }

    group
}

pub fn load_obj_file(file: &str, material: Material) -> Group {
    let (models, _materials) = tobj::load_obj(file, &tobj::LoadOptions::default())
       .expect(&format!("Failed to OBJ load file: {}", file));

    match models.len() {
        0 => panic!("No models found in file: {}", file),
        1 => create_group(&models[0].mesh, material),
        _ => {
            let mut master_group = Group::new();
            for m in models {
                master_group.add_child(Arc::new(create_group(&m.mesh, material.clone())));
            }
            master_group
        }
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
    use crate::raytracer::scene::Scene;
    use crate::tuple::Tuple;

    #[test]
    fn test_load_obj_file() {
        let obj_file = "examples/teapot-low.obj";
        let group = super::load_obj_file(obj_file, Material::default());
        assert_eq!(group.child_ids.len(), 240);
    }

    #[test]
    fn test_parse_vertex() {
        let obj_file = "examples/teapot-low.obj";
        let (models, materials) =
            tobj::load_obj(
                &obj_file,
                &tobj::LoadOptions::default()
            )
                .expect("Failed to OBJ load file");

        // Note: If you don't mind missing the materials, you can generate a default.
        let materials = materials.expect("Failed to load MTL file");

        println!("Number of models          = {}", models.len());
        println!("Number of materials       = {}", materials.len());

        for (i, m) in models.iter().enumerate() {
            let mesh = &m.mesh;
            println!();
            println!("model[{}].name             = \'{}\'", i, m.name);
            println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

            println!(
                "model[{}].face_count       = {}",
                i,
                mesh.face_arities.len()
            );

            let mut next_face = 0;
            for face in 0..mesh.face_arities.len() {
                let end = next_face + mesh.face_arities[face] as usize;

                let face_indices = &mesh.indices[next_face..end];
                println!(" face[{}].indices          = {:?}", face, face_indices);

                if !mesh.texcoord_indices.is_empty() {
                    let texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
                    println!(
                        " face[{}].texcoord_indices = {:?}",
                        face, texcoord_face_indices
                    );
                }
                if !mesh.normal_indices.is_empty() {
                    let normal_face_indices = &mesh.normal_indices[next_face..end];
                    println!(
                        " face[{}].normal_indices   = {:?}",
                        face, normal_face_indices
                    );
                }

                next_face = end;
            }

            // Normals and texture coordinates are also loaded, but not printed in
            // this example.
            println!(
                "model[{}].positions        = {}",
                i,
                mesh.positions.len() / 3
            );
            assert!(mesh.positions.len() % 3 == 0);

            for vtx in 0..mesh.positions.len() / 3 {
                println!(
                    "              position[{}] = ({}, {}, {})",
                    vtx,
                    mesh.positions[3 * vtx],
                    mesh.positions[3 * vtx + 1],
                    mesh.positions[3 * vtx + 2]
                );
            }
        }

        for (i, m) in materials.iter().enumerate() {
            println!("material[{}].name = \'{}\'", i, m.name);
            if let Some(ambient) = m.ambient {
                println!(
                    "    material.Ka = ({}, {}, {})",
                    ambient[0], ambient[1], ambient[2]
                );
            }
            if let Some(diffuse) = m.diffuse {
                println!(
                    "    material.Kd = ({}, {}, {})",
                    diffuse[0], diffuse[1], diffuse[2]
                );
            }
            if let Some(specular) = m.specular {
                println!(
                    "    material.Ks = ({}, {}, {})",
                    specular[0], specular[1], specular[2]
                );
            }
            if let Some(shininess) = m.shininess {
                println!("    material.Ns = {}", shininess);
            }
            if let Some(dissolve) = m.dissolve {
                println!("    material.d = {}", dissolve);
            }
            if let Some(ambient_texture) = &m.ambient_texture {
                println!("    material.map_Ka = {}", ambient_texture);
            }
            if let Some(diffuse_texture) = &m.diffuse_texture {
                println!("    material.map_Kd = {}", diffuse_texture);
            }
            if let Some(specular_texture) = &m.specular_texture {
                println!("    material.map_Ks = {}", specular_texture);
            }
            if let Some(shininess_texture) = &m.shininess_texture {
                println!("    material.map_Ns = {}", shininess_texture);
            }
            if let Some(normal_texture) = &m.normal_texture {
                println!("    material.map_Bump = {}", normal_texture);
            }
            if let Some(dissolve_texture) = &m.dissolve_texture {
                println!("    material.map_d = {}", dissolve_texture);
            }

            for (k, v) in &m.unknown_param {
                println!("    material.{} = {}", k, v);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_render_model() {
        use crate::color::Color;

        let mut c = Camera::new(800, 400, std::f64::consts::PI / 3.0);
        let from = Tuple::point(0.0, 1.5, -5.0);
        let to = Tuple::point(0.0, 1.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        c.transform = Matrix::view_transform(from, to, up);

        let mut w = Scene::new(Light::new_point_light(Tuple::point(-10.0, 10.0, -10.0), Color::new(1.0, 1.0, 1.0)));

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

        let mut material = Material::default();
        material.pattern = Pattern::solid(Color::new(0.302, 0.71, 0.98), Matrix::identity(4));
        let mut group = super::load_obj_file("examples/teapot-low.obj", material);
        group.transform = Matrix::identity(4)
            //.multiply(&Matrix::rotate_y(std::f64::consts::PI))
            .multiply(&Matrix::scale(0.10, 0.10, 0.10))
            .multiply(&Matrix::rotate_x(std::f64::consts::PI / -2.0))
        ;
        w.add_object(Arc::new(group));

        let image = c.render(&w);
        //let image = c.render_sequential(&w);
        //assert_eq!(image.pixel_at(5, 5), Color::new(0.38066, 0.47583, 0.2855));

        image.write_to_file("canvas.png");
    }
}
use std::borrow::BorrowMut;
use std::{collections::HashMap, hash::Hash, path::Path, rc::Rc};

use glam::Vec4Swizzles;
use glam::{Mat4, Vec2, Vec3, Vec4};
use gltf::Material;
use gltf::{buffer::Data, iter::Nodes, Document};

use crate::{structs::Vertex, texture::Texture};

pub struct Mesh {
    verts: Vec<Vertex>,
}

pub struct Model {
    meshes: HashMap<u32, Mesh> // Where the u32 is the material id
}

fn create_vertex_array(
    primitive: &gltf::Primitive,
    gltf_document: &Document,
    mesh_data: &Vec<Data>,
    local_matrix: Mat4,
) -> Mesh {
    let mut position_vec: &[Vec3] = &[];
    let mut normal_vec: &[Vec3] = &[];
    let mut tangent_vec: &[Vec4] = &[];
    let mut colour_vec: &[Vec4] = &[];
    let mut texcoord_vec: &[Vec2] = &[];
    let mut indices: &[u16] = &[];

    // Loop over all the primitive attributes
    for (name, accessor) in primitive.attributes() {
        // Get buffer view
        let bufferview = accessor.view().unwrap();

        // Find location in buffer
        let buffer_index = bufferview.buffer().index();
        let buffer_offset = bufferview.offset();
        let buffer_end = bufferview.offset() + bufferview.length();

        // Find location in buffer
        let buffer_base = &mesh_data[buffer_index].0;
        let buffer_slice = buffer_base.get(buffer_offset..buffer_end).unwrap();

        // Assign to the vectors
        println!(
            "name: {}, size: {}, count: {}",
            name.to_string(),
            accessor.size(),
            accessor.count()
        );
        unsafe {
            // todo: make this less hardcoded in terms of type
            match name.to_string().as_str() {
                "POSITION" => {
                    position_vec = std::mem::transmute(buffer_slice);
                    position_vec = &position_vec[0..accessor.count()];
                }
                "NORMAL" => {
                    normal_vec = std::mem::transmute(buffer_slice);
                    normal_vec = &normal_vec[0..accessor.count()];
                }
                "TANGENT" => {
                    tangent_vec = std::mem::transmute(buffer_slice);
                    tangent_vec = &tangent_vec[0..accessor.count()];
                }

                "TEXCOORD_0" => {
                    texcoord_vec = std::mem::transmute(buffer_slice);
                    texcoord_vec = &texcoord_vec[0..accessor.count()];
                }
                "COLOR_0" => {
                    colour_vec = std::mem::transmute(buffer_slice);
                    colour_vec = &colour_vec[0..accessor.count()];
                }
                _ => {}
            }
        }
    }

    // Find indices
    {
        // Get accessor
        let accessor = primitive.indices().unwrap();

        // Get buffer view
        let bufferview = accessor.view().unwrap();

        // Find location in buffer
        let buffer_index = bufferview.buffer().index();
        let buffer_offset = bufferview.offset();
        let buffer_end = bufferview.offset() + bufferview.length();

        // Find location in buffer
        let buffer_base = &mesh_data[buffer_index].0;
        let buffer_slice = buffer_base.get(buffer_offset..buffer_end).unwrap();
        unsafe {
            // todo: make this less hardcoded in terms of type
            indices = std::mem::transmute(buffer_slice);
            indices = &indices[0..accessor.count()];
        }
    }

    // Create vertex array
    let mut mesh_out = Mesh { verts: Vec::new() };
    for index in indices {
        let mut vertex = Vertex {
            position: Vec3::new(0., 0., 0.),
            normal: Vec3::new(0., 0., 0.),
            tangent: Vec3::new(0., 0., 0.),
            colour: Vec3::new(0., 0., 0.),
            uv: Vec2::new(0., 0.),
        };
        if !position_vec.is_empty() {
            let pos3 = position_vec[*index as usize];
            vertex.position = local_matrix.transform_vector3(pos3);
        }
        if !normal_vec.is_empty() {
            vertex.normal = local_matrix.transform_vector3(normal_vec[*index as usize]);
        }
        if !tangent_vec.is_empty() {
            vertex.tangent = local_matrix.transform_vector3(tangent_vec[*index as usize].xyz());
        }
        if !tangent_vec.is_empty() {
            vertex.uv = texcoord_vec[*index as usize];
        }
        if !colour_vec.is_empty() {
            vertex.colour.x = f32::powf(colour_vec[*index as usize].x as f32, 1.0 / 2.2);
            if vertex.colour.x > 1.0 {
                vertex.colour.x = 1.0
            }
        }
        mesh_out.verts.push(vertex);
    }
    mesh_out
}

fn traverse_nodes(
    node: &gltf::Node,
    gltf_document: &Document,
    mesh_data: &Vec<Data>,
    local_transform: Mat4,
    primitives_processed: &mut HashMap<u32, Mesh>,
) {
        println!("\t\t\t{}: {}", node.index(), node.name().unwrap());

        // Convert matrix in GLTF model to a Mat4. If it does not exist, set it to identity
        let local_matrix = Mat4::IDENTITY;
        for y in 0..node.transform().matrix().len() {
            for x in 0..node.transform().matrix()[y].len() {
                local_matrix.row(x)[y] = node.transform().matrix()[x][y];
            }
        }

        // If it has a mesh, process it
        if node.mesh().is_some() {
            // Get mesh
            let mesh = node.mesh().unwrap();
            let primitives = mesh.primitives();

            for primitive in primitives {
                println!("Creating vertex array for mesh {}", node.name().unwrap());
                let mesh_buffer_data =
                    create_vertex_array(&primitive, gltf_document, &mesh_data, local_matrix);
                let material = primitive.material().index().unwrap_or(usize::MAX) as u32;
                primitives_processed.insert(material, mesh_buffer_data);
            }
        }

        // If it has children, process those
        if node.children().len() == 0 {
            for child in node.children() {
                traverse_nodes(&child, gltf_document, mesh_data, local_transform, primitives_processed);
            }
    }
}

impl Model {
    pub(crate) fn from_gltf(&mut self, path: &Path) {
        // Load GLTF from file
        let gltf_file = gltf::import(path);
        let (gltf_document, mesh_data, image_data) = gltf_file.unwrap();

        // Loop over each scene
        println!("Scenes:");
        for scene in gltf_document.scenes() {
            // For each scene, get the nodes
            println!("\t{}: {}:", scene.index(), scene.name().unwrap());

            // Print node debug
            println!("\t\tNodes:");
            for node in scene.nodes() {
                traverse_nodes(
                    &node,
                    &gltf_document,
                    &mesh_data,
                    Mat4::IDENTITY,
                    &mut self.meshes,
                );
            }
            println!("test");
        }
        return;
    }

    pub(crate) fn new() -> Model {
        Model { meshes: HashMap::new() }
    }
}

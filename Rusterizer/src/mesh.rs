use std::{collections::HashMap, path::Path};

use glam::Vec4Swizzles;
use glam::{Mat4, Vec2, Vec3, Vec4};

use gltf::{buffer::Data, Document};

use crate::rendering::Renderer;
use crate::{structs::Vertex, texture::Texture};

pub struct Mesh {
    pub verts: Vec<Vertex>,
}

pub struct Model {
    pub meshes: HashMap<String, Mesh>, // Where the u32 is the material id
}

// So what this function needs to do: &[u8] -(reinterpret)> &[SrcCompType] -(convert)> &[DstCompType]
fn reinterpret_then_convert<SrcCompType, DstCompType>(input_buffer: &[u8]) -> Vec<DstCompType>
where
    DstCompType: From<SrcCompType>,
    SrcCompType: Copy,
{
    // &[u8] -> &[SrcCompType]
    let input_ptr = input_buffer.as_ptr();
    let src_comp_buffer: &[SrcCompType] = unsafe {
        std::slice::from_raw_parts(
            std::mem::transmute(input_ptr),
            input_buffer.len() / std::mem::size_of::<SrcCompType>(),
        )
    };

    // &[SrcCompType] -> Vec<DstCompType>
    let mut dst_comp_vec = Vec::<DstCompType>::new();
    for item in src_comp_buffer {
        dst_comp_vec.push(DstCompType::from(*item));
    }

    // Return
    dst_comp_vec
}

fn convert_gltf_buffer_to_f32(input_buffer: &[u8], accessor: &gltf::Accessor) -> Vec<f32> {
    // Convert based on data type
    // First we make a f64 vector (this way we can do fancy generics magic and still convert u32 to f32)
    let values64 = match accessor.data_type() {
        gltf::accessor::DataType::I8 => reinterpret_then_convert::<i8, f64>(input_buffer),
        gltf::accessor::DataType::U8 => reinterpret_then_convert::<u8, f64>(input_buffer),
        gltf::accessor::DataType::I16 => reinterpret_then_convert::<i16, f64>(input_buffer),
        gltf::accessor::DataType::U16 => reinterpret_then_convert::<u16, f64>(input_buffer),
        gltf::accessor::DataType::U32 => reinterpret_then_convert::<u32, f64>(input_buffer),
        gltf::accessor::DataType::F32 => reinterpret_then_convert::<f32, f64>(input_buffer),
    };

    // Then we convert that to a f32 vector - this feels cursed as heck but let's ignore that, it'll be fine!
    let mut values32 = Vec::<f32>::new();
    values32.resize(values64.len(), 0.0);
    for i in 0..values32.len() {
        values32[i] = values64[i] as f32;
    }

    // Return
    values32
}

fn create_vertex_array(
    primitive: &gltf::Primitive,
    gltf_document: &Document,
    mesh_data: &Vec<Data>,
    local_matrix: Mat4,
) -> Mesh {
    let mut position_vec = Vec::<Vec3>::new();
    let mut normal_vec = Vec::<Vec3>::new();
    let mut tangent_vec = Vec::<Vec4>::new();
    let mut colour_vec = Vec::<Vec4>::new();
    let mut texcoord_vec = Vec::<Vec2>::new();
    let mut indices = Vec::<u16>::new();

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
        // todo: make this less hardcoded in terms of type
        match name.to_string().as_str() {
            "POSITION" => {
                let values = convert_gltf_buffer_to_f32(buffer_slice, &accessor);
                for i in (0..accessor.count() * 3).step_by(3) {
                    let slice = &values[i..i + 3];
                    position_vec.push(Vec3::from_slice(slice));
                }
            }
            "NORMAL" => {
                let values = convert_gltf_buffer_to_f32(buffer_slice, &accessor);
                for i in (0..accessor.count() * 3).step_by(3) {
                    let slice = &values[i..i + 3];
                    normal_vec.push(Vec3::from_slice(slice));
                }
            }
            "TANGENT" => {
                let values = convert_gltf_buffer_to_f32(buffer_slice, &accessor);
                for i in (0..accessor.count() * 4).step_by(4) {
                    let slice = &values[i..i + 4];
                    tangent_vec.push(Vec4::from_slice(slice));
                }
            }
            "TEXCOORD_0" => {
                let values = convert_gltf_buffer_to_f32(buffer_slice, &accessor);
                for i in (0..accessor.count() * 2).step_by(2) {
                    let slice = &values[i..i + 2];
                    texcoord_vec.push(Vec2::from_slice(slice));
                }
            }
            "COLOR_0" => {
                let values = convert_gltf_buffer_to_f32(buffer_slice, &accessor);
                for i in (0..accessor.count() * 4).step_by(4) {
                    let slice = &values[i..i + 4];
                    colour_vec.push(Vec4::from_slice(slice));
                }
            }
            _ => {}
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

        // Convert from raw buffer to f32 vec - this is incredibly cursed but it'll have to do
        let indices_f32 = convert_gltf_buffer_to_f32(buffer_slice, &accessor);
        for index in indices_f32 {
            indices.push(index as u16);
        }
    }

    // Create vertex array
    let mut mesh_out = Mesh { verts: Vec::new() };
    for index in indices {
        let mut vertex = Vertex {
            position: Vec3::new(0., 0., 0.),
            normal: Vec3::new(0., 0., 0.),
            tangent: Vec3::new(0., 0., 0.),
            colour: Vec3::new(1., 1., 1.),
            uv: Vec2::new(0., 0.),
        };
        if !position_vec.is_empty() {
            let pos3 = position_vec[index as usize];
            vertex.position = local_matrix.transform_vector3(pos3);
        }
        if !normal_vec.is_empty() {
            vertex.normal = local_matrix.transform_vector3(normal_vec[index as usize]);
        }
        if !tangent_vec.is_empty() {
            vertex.tangent = local_matrix.transform_vector3(tangent_vec[index as usize].xyz());
        }
        if !texcoord_vec.is_empty() {
            vertex.uv = texcoord_vec[index as usize];
        }
        if !colour_vec.is_empty() {
            vertex.colour.x = f32::powf(colour_vec[index as usize].x, 1.0 / 2.2);
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
    primitives_processed: &mut HashMap<String, Mesh>,
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
            let mut mesh_buffer_data =
                create_vertex_array(&primitive, gltf_document, &mesh_data, local_matrix);
            let material = String::from(primitive.material().name().unwrap_or("None"));
            #[allow(clippy::map_entry)] // This was really annoying and made the code less readable
            if primitives_processed.contains_key(&material) {
                let mesh: &mut Mesh = primitives_processed.get_mut(&material).unwrap();
                mesh.verts.append(&mut mesh_buffer_data.verts);
            } else {
                primitives_processed.insert(material, mesh_buffer_data);
            }
        }
    }

    // If it has children, process those
    if node.children().len() == 0 {
        for child in node.children() {
            traverse_nodes(
                &child,
                gltf_document,
                mesh_data,
                local_transform,
                primitives_processed,
            );
        }
    }
}

impl Model {
    pub(crate) fn create_from_gltf(&mut self, path: &Path, renderer: &mut Renderer) {
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

        // Get all the textures from the GLTF
        for material in gltf_document.materials() {
            let gltf_tex = material
                .pbr_metallic_roughness()
                .base_color_texture()
                .unwrap()
                .texture()
                .source()
                .index();
            let image = &image_data[gltf_tex];
            let tex = Texture {
                width: image.width as usize,
                height: image.height as usize,
                depth: 4,
                data: {
                    let mut data = Vec::<u32>::new();
                    for i in (0..image.pixels.len()).step_by(4) {
                        data.push(
                            (image.pixels[i] as u32)
                                + ((image.pixels[i + 1] as u32) << 8)
                                + ((image.pixels[i + 2] as u32) << 16)
                                + ((image.pixels[i + 3] as u32) << 24),
                        );
                    }
                    data
                },
            };
            renderer
                .textures
                .insert(material.name().unwrap().to_string(), tex);
        }
    }

    pub(crate) fn new() -> Model {
        Model {
            meshes: HashMap::new(),
        }
    }
}

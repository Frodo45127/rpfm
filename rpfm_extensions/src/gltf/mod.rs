//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use base64::{Engine, engine::general_purpose::STANDARD};
pub use gltf::{Document, Gltf, json};
use gltf_json::{image::MimeType, validation::{Checked::Valid, USize64}};

use std::fs::File;
use std::io::{BufWriter, Write};
use std::mem;
use std::path::Path;

use rpfm_lib::{error::Result, files::RFileDecoded};
use rpfm_lib::files::rigidmodel::{*, materials::TextureType, vertices::Vertex};

use crate::dependencies::Dependencies;

#[cfg(test)] mod test;

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

pub fn gltf_from_rigid(value: &RigidModel, dependencies: &mut Dependencies) -> Result<Gltf> {
    let mut root = gltf_json::Root::default();

    // All the data that is total war-exclusive goes here.
    //root.extras = Some(Box::new(RawValue:: HashMap::new()));

    for lod in value.lods() {

        // As Gltf doesn't support lod levels natively, we do one scene per lod.
        let mut scene = json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: vec![],
        };

        for mesh_block in lod.mesh_blocks() {

            let vertices = mesh_block.mesh().vertices();
            let indices = mesh_block.mesh().indices();

            // Calculate mins and max for the values of the vertex.
            let (min_pos, max_pos) = bounding_coords_positions(vertices);

            // Encode the vertex and index data to binary.
            let vertex_bin = to_padded_byte_vector(vertices.clone());
            let index_bin = to_padded_byte_vector(indices.clone());

            // Buffers
            let vertex_buffer_length = vertices.len() * mem::size_of::<Vertex>();
            let vertex_buffer = root.push(json::Buffer {
                byte_length: USize64::from(vertex_buffer_length),
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                uri: Some(format!("data:application/octet-stream;base64,{}", STANDARD.encode(&vertex_bin))),
            });

            let index_buffer_length = indices.len() * mem::size_of::<u16>();
            let index_buffer = root.push(json::Buffer {
                byte_length: USize64::from(index_buffer_length),
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                uri: Some(format!("data:application/octet-stream;base64,{}", STANDARD.encode(&index_bin))),
            });

            // Buffer views.
            let vertex_buffer_view = root.push(json::buffer::View {
                buffer: vertex_buffer,
                byte_length: USize64::from(vertex_buffer_length),
                byte_offset: None,
                byte_stride: Some(json::buffer::Stride(mem::size_of::<Vertex>())),
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                target: Some(Valid(json::buffer::Target::ArrayBuffer)),
            });

            let index_buffer_view = root.push(json::buffer::View {
                buffer: index_buffer,
                byte_length: USize64::from(index_buffer_length),
                byte_offset: None,
                byte_stride: None,
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
            });

            // Accessors
            let indices = root.push(json::Accessor {
                buffer_view: Some(index_buffer_view),
                byte_offset: Some(USize64(0)),
                count: USize64::from(indices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::U16,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Scalar),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let positions = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(0)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec3),
                min: Some(json::Value::from(Vec::from(min_pos))),
                max: Some(json::Value::from(Vec::from(max_pos))),
                name: None,
                normalized: false,
                sparse: None,
            });

            let text_coords_1 = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(16)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec2),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let text_coords_2 = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(24)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec2),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let normals = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(32)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec3),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let tangents = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(48)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec4),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            // These are calculated by the client from tangents, not specified in the file.
            /*let _bitangents = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(64)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec4),
                min: Some(json::Value::from(Vec::from(min_bitan))),
                max: Some(json::Value::from(Vec::from(max_bitan))),
                name: None,
                normalized: false,
                sparse: None,
            });*/

            let _colors = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(80)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec4),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let joints = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(96)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::U8,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec4),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let weights = root.push(json::Accessor {
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(USize64(100)),
                count: USize64::from(vertices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(json::accessor::Type::Vec4),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            // After the mesh, add the material data.
            let mut material = json::Material {
                alpha_cutoff: Default::default(),
                alpha_mode: Default::default(),
                double_sided: Default::default(),
                name: Default::default(),
                pbr_metallic_roughness: Default::default(),
                normal_texture: Default::default(),
                occlusion_texture: Default::default(),
                emissive_texture: Default::default(),
                emissive_factor: Default::default(),
                extensions: Default::default(),
                extras: Default::default(),
            };

            // Add the textures used by the material block.
            for text in mesh_block.material().textures() {
                if let Ok(ref mut image) = dependencies.file_mut(text.path(), true, true) {
                    let image_data = image.decode(&None, false, true)?.unwrap();

                    if let RFileDecoded::Image(image) = image_data {
                        let image_buffer_length = image.data().len();

                        let image_buffer = root.push(json::Buffer {
                            byte_length: USize64::from(image_buffer_length),
                            extensions: Default::default(),
                            extras: Default::default(),
                            name: None,
                            uri: Some(format!("data:application/octet-stream;base64,{}", STANDARD.encode(&image.data()))),
                        });

                        let image_buffer_view = root.push(json::buffer::View {
                            buffer: image_buffer,
                            byte_length: USize64::from(image_buffer_length),
                            byte_offset: None,
                            byte_stride: None,
                            extensions: Default::default(),
                            extras: Default::default(),
                            name: None,
                            target: None,
                        });

                        let image = root.push(json::Image {
                            buffer_view: Some(image_buffer_view),
                            mime_type: Some(MimeType(String::from("image/png"))),
                            name: Default::default(),
                            uri: Default::default(),
                            extensions: Default::default(),
                            extras: Default::default(),
                        });

                        let texture = root.push(json::Texture {
                            name: None,
                            sampler: None,
                            source: image,
                            extensions: None,
                            extras: None,
                        });

                        match text.tex_type() {
                            TextureType::Diffuse => {
                                material.pbr_metallic_roughness.base_color_texture = Some(json::texture::Info {
                                    index: texture,
                                    tex_coord: 1,
                                    extensions: None,
                                    extras: Default::default(),
                                });
                            },
                            TextureType::Normal => {
                                material.normal_texture = Some(json::material::NormalTexture {
                                    index: texture,
                                    scale: 1.0,
                                    tex_coord: 0,
                                    extensions: None,
                                    extras: None,
                                });
                            },
                            _ => {}
                        }
                    }
                }
            }

            let material = root.push(material);

            // Build the primitive for the mesh.
            let primitive = json::mesh::Primitive {
                attributes: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert(Valid(json::mesh::Semantic::Positions), positions);
                    map.insert(Valid(json::mesh::Semantic::Normals), normals);
                    map.insert(Valid(json::mesh::Semantic::Tangents), tangents);
                    map.insert(Valid(json::mesh::Semantic::TexCoords(0)), text_coords_1);
                    map.insert(Valid(json::mesh::Semantic::TexCoords(1)), text_coords_2);
                    //map.insert(Valid(json::mesh::Semantic::Colors(0)), colors);
                    map.insert(Valid(json::mesh::Semantic::Joints(0)), joints);
                    map.insert(Valid(json::mesh::Semantic::Weights(0)), weights);
                    map
                },
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(indices),
                material: Some(material),
                mode: Valid(json::mesh::Mode::Triangles),
                targets: None,
            };

            let mesh = root.push(json::Mesh {
                extensions: Default::default(),
                extras: Default::default(),
                name: Some(mesh_block.mesh().name().to_owned()),
                primitives: vec![primitive],
                weights: None,
            });

            let node = root.push(json::Node {
                mesh: Some(mesh),
                ..Default::default()
            });

            scene.nodes.push(node);
        }

        root.push(scene);
    }

    // Build the gltf itself.
    let gltf = Gltf {
        document: Document::from_json(root).unwrap(),
        blob: None,
    };

    Ok(gltf)
}

/// NOT YET IMPLEMENTED.
pub fn rigid_from_gltf(_value: &Gltf) -> Result<RigidModel> {
    let rigid = RigidModel::default();

    Ok(rigid)
}

pub fn save_gltf_to_disk(value: &Gltf, path: &Path) -> Result<()> {
    let mut writer_gltf = BufWriter::new(File::create(path)?);
    writer_gltf.write_all(value.as_json().to_string_pretty()?.as_bytes())?;
    //let mut writer_bin = BufWriter::new(File::create(path).unwrap());
    //writer_bin.write_all(&value.blob.clone().unwrap()).unwrap();
    //
    Ok(())
}

/// Calculate bounding coordinates of a list of vertices, used for the clipping distance of the model
fn bounding_coords_positions(points: &[Vertex]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.position();
        for i in 0..3 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}
/*
fn bounding_coords_normals(points: &[Vertex]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.normal();
        for i in 0..3 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn bounding_coords_tangents(points: &[Vertex]) -> ([f32; 4], [f32; 4]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.tangent();
        for i in 0..4 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn bounding_coords_bitangents(points: &[Vertex]) -> ([f32; 4], [f32; 4]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.bitangent();
        for i in 0..4 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn bounding_coords_colours(points: &[Vertex]) -> ([f32; 4], [f32; 4]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.color();
        for i in 0..4 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn bounding_coords_joints(points: &[Vertex]) -> ([u8; 4], [u8; 4]) {
    let mut min = [u8::MAX, u8::MAX, u8::MAX, u8::MAX];
    let mut max = [u8::MIN, u8::MIN, u8::MIN, u8::MIN];

    for point in points {
        let p = point.bone_indices();
        for i in 0..4 {
            min[i] = u8::min(min[i], p[i]);
            max[i] = u8::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn bounding_coords_weight(points: &[Vertex]) -> ([f32; 4], [f32; 4]) {
    let mut min = [f32::MAX, f32::MAX, f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN, f32::MIN, f32::MIN];

    for point in points {
        let p = point.weights();
        for i in 0..4 {
            min[i] = f32::min(min[i], p[i]);
            max[i] = f32::max(max[i], p[i]);
        }
    }
    (min, max)
}

fn align_to_multiple_of_four(n: &mut usize) {
    *n = (*n + 3) & !3;
}*/

fn to_padded_byte_vector<T>(vec: Vec<T>) -> Vec<u8> {
    let byte_length = vec.len() * mem::size_of::<T>();
    let byte_capacity = vec.capacity() * mem::size_of::<T>();
    let alloc = vec.into_boxed_slice();
    let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
    let mut new_vec = unsafe { Vec::from_raw_parts(ptr, byte_length, byte_capacity) };
    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }
    new_vec
}

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This is a module to read/write binary RigidModel files, v6.
//!
//! For internal use only.

use crate::error::Result;
use crate::binary::{ReadBytes, WriteBytes};
use crate::files::rigidmodel::*;
use crate::files::rigidmodel::materials::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl RigidModel {

    pub fn read_v6<R: ReadBytes>(&mut self, data: &mut R) -> Result<()> {
        let lod_count = data.read_u16()?;
        self.uk_1 = data.read_u16()?;
        self.skeleton_id = data.read_string_u8_0padded(PADDED_SIZE_128)?;

        // First pass to get the lods headers.
        self.lods = Vec::with_capacity(lod_count as usize);
        for _ in 0..lod_count {
            let mut lod = Lod::default();

            let mesh_count = data.read_u32()?;
            lod.mesh_blocks = Vec::with_capacity(mesh_count as usize);

            // Unused by the parser.
            let _vertices_data_length = data.read_u32()?;
            let _indices_data_length = data.read_u32()?;
            let _start_offset = data.read_u32()?;

            lod.visibility_distance = data.read_f32()?;

            self.lods.push(lod);
        }

        // Second pass is to populate the lods data.
        for lod in &mut self.lods {
            for _ in 0..lod.mesh_blocks.capacity() {
                let mut mesh = MeshBlock::default();

                // Mesh data
                mesh.mesh.material_type = MaterialType::try_from(data.read_u16()?)?;

                // Possible mesh lenght.
                let _render_flags = data.read_u16()?;
                let _mesh_section_size = data.read_u32()?;
                let _vertex_offset = data.read_u32()?;
                let vertex_count = data.read_u32()?;
                let _index_offset = data.read_u32()?;
                let index_count = data.read_u32()?;

                mesh.mesh.min_bb = Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?);
                mesh.mesh.max_bb = Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?);

                mesh.mesh.shader_params.data = data.read_slice(PADDED_SIZE_32, false)?;
                //mesh.mesh.shader_params.name = data.read_string_u8_0terminated()?;
                //mesh.mesh.shader_params.uk_1 = data.read_slice(PADDED_SIZE_10, false)?;
                //mesh.mesh.shader_params.uk_2 = data.read_slice(8, false)?;
                //mesh.mesh.shader_params.uk_2 = data.read_slice(PADDED_SIZE_10, false)?;

                mesh.mesh.vertices = Vec::with_capacity(vertex_count as usize);
                mesh.mesh.indices = Vec::with_capacity(index_count as usize);

                mesh.material = Material::read(data, mesh.mesh.material_type)?;

                for _ in 0..mesh.mesh.vertices().capacity() {
                    mesh.mesh.vertices.push(Vertex::read(data, self.version, *mesh.material.vertex_format(), *mesh.mesh.material_type())?);
                }

                for _ in 0..mesh.mesh.indices.capacity() {
                    mesh.mesh.indices.push(data.read_u16()?);
                }

                lod.mesh_blocks.push(mesh);
            }
        }

        Ok(())
    }

    pub fn write_v6<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u16(self.lods.len() as u16)?;
        buffer.write_u16(self.uk_1)?;
        buffer.write_string_u8_0padded(self.skeleton_id(), PADDED_SIZE_128, true)?;

        let mut lod_headers = vec![];
        let mut lod_data = vec![];
        let mut offset_current_lod = 0;

        for lod in self.lods() {
            lod_headers.write_u32(lod.mesh_blocks.len() as u32)?;

            let mut total_vertices_size = 0;
            let mut total_indices_size = 0;

            for mesh in lod.mesh_blocks() {
                let mut mesh_data = vec![];

                mesh_data.write_f32(mesh.mesh.min_bb().x)?;
                mesh_data.write_f32(mesh.mesh.min_bb().y)?;
                mesh_data.write_f32(mesh.mesh.min_bb().z)?;

                mesh_data.write_f32(mesh.mesh.max_bb().x)?;
                mesh_data.write_f32(mesh.mesh.max_bb().y)?;
                mesh_data.write_f32(mesh.mesh.max_bb().z)?;

                mesh_data.write_all(mesh.mesh.shader_params().data())?;
                //mesh_data.write_string_u8_0terminated(mesh.mesh.shader_params().name())?;
                //mesh_data.write_all(mesh.mesh.shader_params().uk_1())?;
                //mesh_data.write_all(mesh.mesh.shader_params().uk_2())?;

                mesh.material.write(&mut mesh_data, mesh.mesh.material_type)?;

                let offset_start_vertices = mesh_data.len();
                for vertex in mesh.mesh.vertices() {
                    vertex.write(&mut mesh_data, self.version, *mesh.material.vertex_format(), *mesh.mesh.material_type())?;
                }

                total_vertices_size += mesh_data.len() - offset_start_vertices;

                let offset_start_indices = mesh_data.len();
                for index in mesh.mesh.indices() {
                    mesh_data.write_u16(*index)?;
                }

                total_indices_size += mesh_data.len() - offset_start_indices;

                // We do the header after so we can get the correct sizes for it.
                let mut mesh_header = vec![];
                mesh_header.write_u16(u16::from(mesh.mesh.material_type))?;

                mesh_header.write_u16(0)?;
                mesh_header.write_u32(24 + mesh_data.len() as u32)?;
                mesh_header.write_u32(24 + offset_start_vertices as u32)?;
                mesh_header.write_u32(mesh.mesh.vertices.len() as u32)?;
                mesh_header.write_u32(24 + offset_start_indices as u32)?;
                mesh_header.write_u32(mesh.mesh.indices.len() as u32)?;

                lod_data.append(&mut mesh_header);
                lod_data.append(&mut mesh_data);
            }

            lod_headers.write_u32(total_vertices_size as u32)?;
            lod_headers.write_u32(total_indices_size as u32)?;

            // Offset of the first mesh of this lod. We have to calculate it as such:
            // 5 items per lod header * 4 bytes per item * X num of lods + 134 (header lenght) + data until current lod.
            lod_headers.write_u32((5 * 4 * self.lods().len() as u32) + HEADER_LENGTH + offset_current_lod)?;
            offset_current_lod = lod_data.len() as u32;

            lod_headers.write_f32(lod.visibility_distance)?;
        }

        buffer.write_all(&lod_headers)?;
        buffer.write_all(&lod_data)?;

        Ok(())
    }
}

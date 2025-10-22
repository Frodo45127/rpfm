//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

//---------------------------------------------------------------------------//
//                            Implementation
//---------------------------------------------------------------------------//

impl Material {
    pub fn read_cloth<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let mut mat = Self::default();

        mat.vertex_format = VertexFormat::try_from(data.read_u16()?)?;
        mat.name = data.read_string_u8_0padded(PADDED_SIZE_32)?;

        mat.texture_directory = data.read_string_u8_0padded(PADDED_SIZE_256)?;
        mat.filters = data.read_string_u8_0padded(PADDED_SIZE_256)?;

        mat.padding_byte0 = data.read_u8()?;
        mat.padding_byte1 = data.read_u8()?;

        mat.v_pivot = Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?);

        mat.matrix1 = Matrix3x4::new(
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?
        );
        mat.matrix2 = Matrix3x4::new(
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?
        );
        mat.matrix3 = Matrix3x4::new(
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
            data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?
        );

        mat.i_matrix_index = data.read_i32()?;
        mat.i_parent_matrix_index = data.read_i32()?;

        let attachment_points_count = data.read_u32()?;
        mat.attachment_points = Vec::with_capacity(attachment_points_count as usize);

        let texture_count = data.read_u32()?;
        mat.textures = Vec::with_capacity(texture_count as usize);

        let param_string_count = data.read_u32()?;
        mat.params_string = Vec::with_capacity(param_string_count as usize);

        let param_f32_count = data.read_u32()?;
        mat.params_f32 = Vec::with_capacity(param_f32_count as usize);

        let param_i32_count = data.read_u32()?;
        mat.params_i32 = Vec::with_capacity(param_i32_count as usize);

        let param_vector4df32_count = data.read_u32()?;
        mat.params_vector4df32 = Vec::with_capacity(param_vector4df32_count as usize);

        mat.sz_padding = data.read_slice(124, false)?;

        // Attachment points.
        for _ in 0..mat.attachment_points.capacity() {
            let mut entry = AttachmentPointEntry::default();

            entry.name = data.read_string_u8_0padded(PADDED_SIZE_32)?;
            entry.matrix = Matrix3x4::new(
                data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
                data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?,
                data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?
            );

            entry.bone_id = data.read_u32()?;

            mat.attachment_points.push(entry);
        }

        // Textures
        for _ in 0..mat.textures.capacity() {
            let mut entry = Texture::default();

            entry.tex_type = TextureType::try_from(data.read_i32()?)?;
            entry.path = data.read_string_u8_0padded(PADDED_SIZE_256)?;

            mat.textures.push(entry)
        }

        // Extra material params.
        for _ in 0..mat.params_string.capacity() {
            mat.params_string.push((data.read_i32()?, data.read_sized_string_u8()?));
        }

        for _ in 0..mat.params_f32.capacity() {
            mat.params_f32.push((data.read_i32()?, data.read_f32()?));
        }

        for _ in 0..mat.params_i32.capacity() {
            mat.params_i32.push((data.read_i32()?, data.read_i32()?));
        }

        for _ in 0..mat.params_vector4df32.capacity() {
            mat.params_vector4df32.push((data.read_i32()?, Vector4::new(data.read_f32()?, data.read_f32()?, data.read_f32()?, data.read_f32()?)));
        }

        // These have a ton of extra data here.
        let uk_7_count = data.read_u32()?;
        mat.uk_7 = Vec::with_capacity(uk_7_count as usize);

        let uk_8_count = data.read_u32()?;
        mat.uk_8 = Vec::with_capacity(uk_8_count as usize);

        let uk_9_count = data.read_u32()?;
        mat.uk_9 = Vec::with_capacity(uk_9_count as usize);

        for _ in 0..mat.uk_7.capacity() {
            let mut uk = Uk7::default();

            uk.uk1 = data.read_i32()?;
            uk.uk2 = data.read_i32()?;
            uk.uk3 = data.read_f32()?;

            mat.uk_7.push(uk);
        }

        for _ in 0..mat.uk_8.capacity() {
            let mut uk = Uk8::default();

            uk.uk1 = data.read_i32()?;

            mat.uk_8.push(uk);
        }

        for _ in 0..mat.uk_9.capacity() {
            let mut uk = Uk9::default();

            uk.uk1 = data.read_i32()?;
            uk.uk2 = data.read_i32()?;
            uk.uk3 = data.read_i32()?;

            mat.uk_9.push(uk);
        }

        Ok(mat)
    }
    pub fn write_cloth<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u16(u16::from(self.vertex_format))?;
        buffer.write_string_u8_0padded(self.name(), PADDED_SIZE_32, true)?;

        buffer.write_string_u8_0padded(self.texture_directory(), PADDED_SIZE_256, true)?;
        buffer.write_string_u8_0padded(self.filters(), PADDED_SIZE_256, true)?;

        buffer.write_u8(self.padding_byte0)?;
        buffer.write_u8(self.padding_byte1)?;

        buffer.write_f32(self.v_pivot().x)?;
        buffer.write_f32(self.v_pivot().y)?;
        buffer.write_f32(self.v_pivot().z)?;

        buffer.write_f32(self.matrix1().m11)?;
        buffer.write_f32(self.matrix1().m12)?;
        buffer.write_f32(self.matrix1().m13)?;
        buffer.write_f32(self.matrix1().m14)?;
        buffer.write_f32(self.matrix1().m21)?;
        buffer.write_f32(self.matrix1().m22)?;
        buffer.write_f32(self.matrix1().m23)?;
        buffer.write_f32(self.matrix1().m24)?;
        buffer.write_f32(self.matrix1().m31)?;
        buffer.write_f32(self.matrix1().m32)?;
        buffer.write_f32(self.matrix1().m33)?;
        buffer.write_f32(self.matrix1().m34)?;

        buffer.write_f32(self.matrix2().m11)?;
        buffer.write_f32(self.matrix2().m12)?;
        buffer.write_f32(self.matrix2().m13)?;
        buffer.write_f32(self.matrix2().m14)?;
        buffer.write_f32(self.matrix2().m21)?;
        buffer.write_f32(self.matrix2().m22)?;
        buffer.write_f32(self.matrix2().m23)?;
        buffer.write_f32(self.matrix2().m24)?;
        buffer.write_f32(self.matrix2().m31)?;
        buffer.write_f32(self.matrix2().m32)?;
        buffer.write_f32(self.matrix2().m33)?;
        buffer.write_f32(self.matrix2().m34)?;

        buffer.write_f32(self.matrix3().m11)?;
        buffer.write_f32(self.matrix3().m12)?;
        buffer.write_f32(self.matrix3().m13)?;
        buffer.write_f32(self.matrix3().m14)?;
        buffer.write_f32(self.matrix3().m21)?;
        buffer.write_f32(self.matrix3().m22)?;
        buffer.write_f32(self.matrix3().m23)?;
        buffer.write_f32(self.matrix3().m24)?;
        buffer.write_f32(self.matrix3().m31)?;
        buffer.write_f32(self.matrix3().m32)?;
        buffer.write_f32(self.matrix3().m33)?;
        buffer.write_f32(self.matrix3().m34)?;

        buffer.write_i32(self.i_matrix_index)?;
        buffer.write_i32(self.i_parent_matrix_index)?;

        buffer.write_i32(self.attachment_points.len() as i32)?;
        buffer.write_i32(self.textures.len() as i32)?;
        buffer.write_i32(self.params_string.len() as i32)?;
        buffer.write_i32(self.params_f32.len() as i32)?;
        buffer.write_i32(self.params_i32.len() as i32)?;
        buffer.write_i32(self.params_vector4df32.len() as i32)?;

        buffer.write_all(self.sz_padding())?;

        for att_point in self.attachment_points() {
            buffer.write_string_u8_0padded(att_point.name(), PADDED_SIZE_32, true)?;

            buffer.write_f32(att_point.matrix().m11)?;
            buffer.write_f32(att_point.matrix().m12)?;
            buffer.write_f32(att_point.matrix().m13)?;
            buffer.write_f32(att_point.matrix().m14)?;
            buffer.write_f32(att_point.matrix().m21)?;
            buffer.write_f32(att_point.matrix().m22)?;
            buffer.write_f32(att_point.matrix().m23)?;
            buffer.write_f32(att_point.matrix().m24)?;
            buffer.write_f32(att_point.matrix().m31)?;
            buffer.write_f32(att_point.matrix().m32)?;
            buffer.write_f32(att_point.matrix().m33)?;
            buffer.write_f32(att_point.matrix().m34)?;

            buffer.write_u32(att_point.bone_id)?;
        }

        for texture in self.textures() {
            buffer.write_i32(i32::try_from(texture.tex_type)?)?;
            buffer.write_string_u8_0padded(texture.path(), PADDED_SIZE_256, true)?;
        }

        for (key, param) in self.params_string() {
            buffer.write_i32(*key)?;
            buffer.write_sized_string_u8(param)?;
        }

        for (key, param) in self.params_f32() {
            buffer.write_i32(*key)?;
            buffer.write_f32(*param)?;
        }

        for (key, param) in self.params_i32() {
            buffer.write_i32(*key)?;
            buffer.write_i32(*param)?;
        }

        for (key, param) in self.params_vector4df32() {
            buffer.write_i32(*key)?;
            buffer.write_f32(param.x)?;
            buffer.write_f32(param.y)?;
            buffer.write_f32(param.z)?;
            buffer.write_f32(param.w)?;
        }

        buffer.write_i32(self.uk_7.len() as i32)?;
        buffer.write_i32(self.uk_8.len() as i32)?;
        buffer.write_i32(self.uk_9.len() as i32)?;

        for uk in self.uk_7() {
            buffer.write_i32(uk.uk1)?;
            buffer.write_i32(uk.uk2)?;
            buffer.write_f32(uk.uk3)?;
        }

        for uk in self.uk_8() {
            buffer.write_i32(uk.uk1)?;
        }

        for uk in self.uk_9() {
            buffer.write_i32(uk.uk1)?;
            buffer.write_i32(uk.uk2)?;
            buffer.write_i32(uk.uk3)?;
        }

        Ok(())
    }
}

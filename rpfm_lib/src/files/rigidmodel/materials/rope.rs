//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
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

/// Rope materials of version 6 RigidModels: the default material, followed by the rope as a chain of nodes.
///
/// There's one link fewer than nodes, and the file doesn't store their count.
impl Material {
    pub fn read_rope<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        let mut material = Self::read_default(data, version)?;

        let node_count = data.read_u32()?;
        let rope = &mut material.rope;
        rope.attachments = Vector4::new(data.read_i32()?, data.read_i32()?, data.read_i32()?, data.read_i32()?);
        rope.uk_1 = data.read_f32()?;
        rope.uk_2 = data.read_u32()?;

        rope.nodes = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            rope.nodes.push(RopeNode {
                position: Vector3::new(data.read_f32()?, data.read_f32()?, data.read_f32()?),
                weight: data.read_f32()?,
                uk_1: data.read_f32()?,
            });
        }

        let link_count = node_count.saturating_sub(1);
        rope.links = Vec::with_capacity(link_count as usize);
        for _ in 0..link_count {
            rope.links.push(RopeLink {
                node_a: data.read_u32()?,
                node_b: data.read_u32()?,
                length: data.read_f32()?,
            });
        }

        Ok(material)
    }

    pub fn write_rope<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        let rope = &self.rope;
        if rope.links.len() != rope.nodes.len().saturating_sub(1) {
            return Err(RLibError::EncodingRigidModelRopeLinkCountMismatch(rope.nodes.len(), rope.links.len()));
        }

        self.write_default(buffer, version)?;

        buffer.write_u32(rope.nodes.len() as u32)?;
        for attachment in rope.attachments.iter() {
            buffer.write_i32(*attachment)?;
        }
        buffer.write_f32(rope.uk_1)?;
        buffer.write_u32(rope.uk_2)?;

        for node in &rope.nodes {
            buffer.write_f32(node.position.x)?;
            buffer.write_f32(node.position.y)?;
            buffer.write_f32(node.position.z)?;
            buffer.write_f32(node.weight)?;
            buffer.write_f32(node.uk_1)?;
        }

        for link in &rope.links {
            buffer.write_u32(link.node_a)?;
            buffer.write_u32(link.node_b)?;
            buffer.write_f32(link.length)?;
        }

        Ok(())
    }
}

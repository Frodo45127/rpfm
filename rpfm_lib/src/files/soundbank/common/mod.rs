//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! For more info about all this stuff, check https://github.com/bnnm/wwiser/.

use getset::*;
use serde_derive::{Serialize, Deserialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::Result;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct InitialRTPC {
    entries: Vec<RTPCEntry>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RTPCEntry {
    id: u32,
    rtpc_type: u8,
    rtpc_accum: u8,
    param_id: u32,
    rtpc_curve_id: u32,
    rtpc_curve: RTPCCurve,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RTPCCurve {
    scaling: u32,
    rptc_graph: RTPCGraph,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RTPCGraph {
    entries: Vec<RTPCGraphEntry>,
}

#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct RTPCGraphEntry {
    from: f32,
    to: f32,
    interp: u32,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl InitialRTPC {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        let mut rtpc = Self::default();

        let count = if version <= 36 {
            data.read_u32()?
        } else {
            data.read_u16()? as u32
        };

        for _ in 0..count {
            let mut entry = RTPCEntry::default();

            if version <= 36 {
                todo!("parse FXID");
            } else if version <= 48 {
                todo!("parse FXID and read bool is_rendered");
            }

            entry.id = data.read_u32()?;

            if version > 89 {
                entry.rtpc_type = data.read_u8()?;
                entry.rtpc_accum = data.read_u8()?;
            }

            if version <= 89 {
                entry.param_id = data.read_u32()?;
            } else if version <= 113 {
                entry.param_id = data.read_u8()? as u32;
            } else {

                // In this case, is one of those 0x7F padded things.
                /*
                elif type == TYPE_VAR:
                cur = r.u8()
                value = (cur & 0x7F)

                max = 0
                while (cur & 0x80) and max < 10:
                    cur = r.u8()
                    value = (value << 7) | (cur & 0x7F)
                    max += 1
                if max >= 10: #arbitary max
                    raise ValueError("unexpected variable loop count")
                */
                todo!("Implement 0x7F thingy");
            }

            entry.rtpc_curve_id = data.read_u32()?;
            entry.rtpc_curve = RTPCCurve::read(data, version)?;

            rtpc.entries.push(entry);
        }

        Ok(rtpc)
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {

        if version <= 36 {
            buffer.write_u32(self.entries.len() as u32)?;
        } else {
            buffer.write_u16(self.entries.len() as u16)?;
        }

        for entry in self.entries() {
            if version <= 36 {
                todo!("write FXID");
            } else if version <= 48 {
                todo!("write FXID and write bool is_rendered");
            }

            buffer.write_u32(entry.id)?;

            if version > 89 {
                buffer.write_u8(entry.rtpc_type)?;
                buffer.write_u8(entry.rtpc_accum)?;
            }

            if version <= 89 {
                buffer.write_u32(entry.param_id)?;
            } else if version <= 113 {
                buffer.write_u8(entry.param_id as u8)?;
            } else {

                // In this case, is one of those 0x7F padded things.
                /*
                elif type == TYPE_VAR:
                cur = r.u8()
                value = (cur & 0x7F)

                max = 0
                while (cur & 0x80) and max < 10:
                    cur = r.u8()
                    value = (value << 7) | (cur & 0x7F)
                    max += 1
                if max >= 10: #arbitary max
                    raise ValueError("unexpected variable loop count")
                */
                todo!("Implement 0x7F thingy");
            }

            buffer.write_u32(entry.rtpc_curve_id)?;
            entry.rtpc_curve.write(buffer, version)?;
        }

        Ok(())
    }
}

impl RTPCCurve {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        if version <= 36 {
            Ok(Self {
                scaling: data.read_u32()?,
                rptc_graph: RTPCGraph::read(data, version)?,
            })
        } else {
            Ok(Self {
                scaling: data.read_u8()? as u32,
                rptc_graph: RTPCGraph::read(data, version)?,
            })
        }
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        if version <= 36 {
            buffer.write_u32(self.scaling)?;
        } else {
            buffer.write_u8(self.scaling as u8)?;
        }

        self.rptc_graph.write(buffer, version)?;

        Ok(())
    }
}

impl RTPCGraph {

    pub(crate) fn read<R: ReadBytes>(data: &mut R, version: u32) -> Result<Self> {
        let mut graph = Self::default();
        let len = if version <= 36 {
            data.read_u32()?
        } else {
            data.read_u16()? as u32
        };

        for _ in 0..len {
            graph.entries.push(RTPCGraphEntry {
                from: data.read_f32()?,
                to: data.read_f32()?,
                interp: data.read_u32()?,
            });
        }

        Ok(graph)
    }

    pub(crate) fn write<W: WriteBytes>(&self, buffer: &mut W, version: u32) -> Result<()> {
        if version <= 36 {
            buffer.write_u32(self.entries.len() as u32)?;
        } else {
            buffer.write_u16(self.entries.len() as u16)?;
        }

        for entry in self.entries() {
            buffer.write_f32(entry.from)?;
            buffer.write_f32(entry.to)?;
            buffer.write_u32(entry.interp)?;
        }

        Ok(())
    }
}

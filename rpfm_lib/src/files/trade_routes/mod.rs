//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Trade routes file format support (`campaign_maps/<map>/trade_routes.ptd`).
//!
//! These files contain the precomputed trade network of a campaign map: the spline curves trade routes
//! follow, and the routes between regions and between ports. The game copies them into the trade manager of
//! the campaign startpos.
//!
//! # File Format
//!
//! All values are little endian:
//!
//! - Signature `\x89PTD\r\n\x1a\n`, `u32` version (1 in Attila).
//! - `u32` region count, then each region key as `u32` length + bytes.
//! - `u32` spline count, then per spline: 4 flag bytes, `u32` segment count, and per segment the start point
//!   (`u16`, `u16`), the first control point as an `i16` offset from the start, the second control point as an
//!   `i16` offset from the end, and the end point (`u16`, `u16`). Points are in raw map units; offsets are at
//!   100 times the raw scale.
//! - Region routes, then port routes: `u32` count, then per route `u32` from, `u32` to, `u32` edge count and
//!   the edges. `from`/`to` index the region table. A pair can appear more than once (alternative paths).
//! - A trailing `u32` (checksum?).

use getset::*;
use serde_derive::{Deserialize, Serialize};

use crate::binary::{ReadBytes, WriteBytes};
use crate::error::{Result, RLibError};
use crate::files::{DecodeableExtraData, Decodeable, EncodeableExtraData, Encodeable};
use crate::utils::check_size_mismatch;

/// File extension of trade routes files.
pub const EXTENSION: &str = ".ptd";

/// Signature of trade routes files.
pub const SIGNATURE: &[u8; 8] = b"\x89PTD\r\n\x1a\n";

/// Version of trade routes files supported.
const SUPPORTED_VERSION: u32 = 1;

#[cfg(test)] mod trade_routes_test;

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// Decoded trade routes (`.ptd`) file.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TradeRoutes {

    /// File version.
    version: u32,

    /// Region keys. Routes refer to regions by their index in this list.
    regions: Vec<String>,

    /// Spline curves the routes are made of.
    splines: Vec<TradeSpline>,

    /// Routes between regions. A region pair may have several alternatives.
    region_routes: Vec<TradeRoute>,

    /// Routes between ports.
    port_routes: Vec<TradeRoute>,

    /// Trailing value of the file (checksum?).
    trailer: u32,
}

/// A spline of the trade network: a chain of cubic Bezier segments.
#[derive(Default, PartialEq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TradeSpline {

    /// Flags. Meaning unknown; the game copies them into the startpos spline record.
    flags: [bool; 4],

    /// Bezier segments, in order.
    segments: Vec<TradeSplineSegment>,
}

/// A cubic Bezier segment, in raw map units.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TradeSplineSegment {

    /// Start point.
    start: (u16, u16),

    /// First control point, as an offset from the start point (at 100x the raw scale).
    start_control: (i16, i16),

    /// Second control point, as an offset from the end point (at 100x the raw scale).
    end_control: (i16, i16),

    /// End point.
    end: (u16, u16),
}

/// A route between two regions or ports: the spline edges to follow, in order.
#[derive(Default, PartialEq, Eq, Clone, Debug, Getters, MutGetters, Setters, Serialize, Deserialize)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct TradeRoute {

    /// Index of the origin in the region table.
    from: u32,

    /// Index of the destination in the region table.
    to: u32,

    /// Edges to follow.
    edges: Vec<TradeRouteEdge>,
}

/// A spline traversal in a route: `spline index * 2 + direction`, where an odd value means the spline is
/// traversed from end to start.
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TradeRouteEdge(u32);

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Decodeable for TradeRoutes {

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: &Option<DecodeableExtraData>) -> Result<Self> {
        let signature = data.read_slice(SIGNATURE.len(), false)?;
        if signature != SIGNATURE {
            return Err(RLibError::DecodingPTDUnsupportedSignature(signature));
        }

        let version = data.read_u32()?;
        if version != SUPPORTED_VERSION {
            return Err(RLibError::DecodingPTDUnsupportedVersion(version));
        }

        let region_count = data.read_u32()?;
        let regions = (0..region_count).map(|_| data.read_sized_string_u8_u32()).collect::<Result<Vec<_>>>()?;

        let spline_count = data.read_u32()?;
        let splines = (0..spline_count).map(|_| TradeSpline::decode(data)).collect::<Result<Vec<_>>>()?;

        let region_routes = TradeRoute::decode_list(data)?;
        let port_routes = TradeRoute::decode_list(data)?;
        let trailer = data.read_u32()?;

        check_size_mismatch(data.stream_position()? as usize, data.len()? as usize)?;

        Ok(Self {
            version,
            regions,
            splines,
            region_routes,
            port_routes,
            trailer,
        })
    }
}

impl Encodeable for TradeRoutes {

    fn encode<W: WriteBytes>(&mut self, buffer: &mut W, _extra_data: &Option<EncodeableExtraData>) -> Result<()> {
        buffer.write_all(SIGNATURE)?;
        buffer.write_u32(self.version)?;

        buffer.write_u32(self.regions.len() as u32)?;
        for region in &self.regions {
            buffer.write_sized_string_u8_u32(region)?;
        }

        buffer.write_u32(self.splines.len() as u32)?;
        for spline in &self.splines {
            spline.encode(buffer)?;
        }

        TradeRoute::encode_list(&self.region_routes, buffer)?;
        TradeRoute::encode_list(&self.port_routes, buffer)?;
        buffer.write_u32(self.trailer)?;
        Ok(())
    }
}

impl TradeSpline {

    fn decode<R: ReadBytes>(data: &mut R) -> Result<Self> {
        let mut flags = [false; 4];
        for flag in &mut flags {
            *flag = data.read_bool()?;
        }

        let segment_count = data.read_u32()?;
        let segments = (0..segment_count).map(|_| TradeSplineSegment::decode(data)).collect::<Result<Vec<_>>>()?;
        Ok(Self { flags, segments })
    }

    fn encode<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        for flag in self.flags {
            buffer.write_bool(flag)?;
        }

        buffer.write_u32(self.segments.len() as u32)?;
        for segment in &self.segments {
            segment.encode(buffer)?;
        }
        Ok(())
    }
}

impl TradeSplineSegment {

    fn decode<R: ReadBytes>(data: &mut R) -> Result<Self> {
        Ok(Self {
            start: (data.read_u16()?, data.read_u16()?),
            start_control: (data.read_i16()?, data.read_i16()?),
            end_control: (data.read_i16()?, data.read_i16()?),
            end: (data.read_u16()?, data.read_u16()?),
        })
    }

    fn encode<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u16(self.start.0)?;
        buffer.write_u16(self.start.1)?;
        buffer.write_i16(self.start_control.0)?;
        buffer.write_i16(self.start_control.1)?;
        buffer.write_i16(self.end_control.0)?;
        buffer.write_i16(self.end_control.1)?;
        buffer.write_u16(self.end.0)?;
        buffer.write_u16(self.end.1)
    }
}

impl TradeRoute {

    /// Creates a route.
    pub fn new(from: u32, to: u32, edges: Vec<TradeRouteEdge>) -> Self {
        Self { from, to, edges }
    }

    /// Decodes a `u32` count followed by that many routes.
    fn decode_list<R: ReadBytes>(data: &mut R) -> Result<Vec<Self>> {
        let count = data.read_u32()?;
        (0..count).map(|_| {
            let from = data.read_u32()?;
            let to = data.read_u32()?;
            let edge_count = data.read_u32()?;
            let edges = (0..edge_count).map(|_| data.read_u32().map(TradeRouteEdge)).collect::<Result<Vec<_>>>()?;
            Ok(Self { from, to, edges })
        }).collect()
    }

    /// Encodes a `u32` count followed by the routes.
    fn encode_list<W: WriteBytes>(routes: &[Self], buffer: &mut W) -> Result<()> {
        buffer.write_u32(routes.len() as u32)?;
        for route in routes {
            buffer.write_u32(route.from)?;
            buffer.write_u32(route.to)?;
            buffer.write_u32(route.edges.len() as u32)?;
            for edge in &route.edges {
                buffer.write_u32(edge.0)?;
            }
        }
        Ok(())
    }
}

impl TradeRouteEdge {

    /// Creates an edge from its stored value.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the stored value.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Returns the index of the spline this edge traverses.
    pub fn spline(&self) -> usize {
        (self.0 / 2) as usize
    }

    /// Returns `true` if the spline is traversed from end to start.
    pub fn is_reversed(&self) -> bool {
        self.0 % 2 == 1
    }
}

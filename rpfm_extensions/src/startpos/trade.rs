//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Trade network of a startpos (`CAMPAIGN_TRADE_MANAGER`), built from `trade_routes.ptd`.
//!
//! Verified against Attila's main campaign: every spline (flags, segments, points, lengths, speed tables) and every
//! region and port route matches the game-built startpos. Not covered: the order the game lists the routes in (hash
//! order).

use getset::Getters;
use serde_derive::{Deserialize, Serialize};

use std::collections::HashSet;

use rpfm_lib::files::trade_routes::{TradeRoute, TradeRouteEdge, TradeRoutes, TradeSplineSegment};

use super::coords::MapScale;

#[cfg(test)] #[path = "trade_test.rs"] mod trade_test;

/// Gauss-Legendre nodes (on [-1, 1]) used to integrate segment lengths.
const GAUSS_NODES: [f64; 5] = [-0.906_179_845_938_664, -0.538_469_310_105_683, 0.0, 0.538_469_310_105_683, 0.906_179_845_938_664];

/// Gauss-Legendre weights matching [`GAUSS_NODES`].
const GAUSS_WEIGHTS: [f64; 5] = [0.236_926_885_056_189, 0.478_628_670_499_366, 0.568_888_888_888_889, 0.478_628_670_499_366, 0.236_926_885_056_189];

/// Subintervals each segment is split into when integrating its length.
const LENGTH_SUBINTERVALS: usize = 64;

/// Samples of the speed table of each segment.
const SPEED_SAMPLES: usize = 100;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Trade network of a campaign, as stored in the startpos.
#[derive(Debug, Clone, PartialEq, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct TradeNetwork {

    /// Region keys. Routes refer to regions by their index in this list.
    regions: Vec<String>,

    /// Splines, in file order (`CAMPAIGN_TRADE_MANAGER/SPLINES`).
    splines: Vec<NetworkSpline>,

    /// One route per region pair (`REGION_ROUTES`).
    region_routes: Vec<NetworkRoute>,

    /// One route per port pair (`PORT_ROUTES`).
    port_routes: Vec<NetworkRoute>,
}

/// A trade spline in world coordinates.
#[derive(Debug, Clone, PartialEq, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct NetworkSpline {

    /// Flags copied from the map file.
    flags: [bool; 4],

    /// Bezier control points of each segment: start, first control, second control, end.
    segments: Vec<[(f32, f32); 4]>,

    /// Speed table of each segment: the running sum of the curve's speed at `t = k / 100`, for k from 0 to 99.
    speed_tables: Vec<Vec<f32>>,

    /// Length of each segment.
    lengths: Vec<f32>,

    /// Total length.
    total_length: f32,
}

/// A route between two regions or ports.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct NetworkRoute {

    /// Index of the origin in the map file's region table.
    from: u32,

    /// Index of the destination in the map file's region table.
    to: u32,

    /// Spline edges to follow.
    edges: Vec<TradeRouteEdge>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl TradeNetwork {

    /// Builds the trade network of a startpos from the campaign map's trade routes file.
    ///
    /// # Arguments
    ///
    /// * `trade_routes` - Decoded `trade_routes.ptd` of the campaign map.
    /// * `scale` - Scale between the campaign map's world and raw units.
    ///
    /// # Returns
    ///
    /// The splines with their lengths, and the first route listed for each region and port pair.
    pub fn from_trade_routes(trade_routes: &TradeRoutes, scale: &MapScale) -> Self {
        let splines = trade_routes.splines().iter().map(|spline| {
            let segments = spline.segments().iter().map(|segment| control_points_world(segment, scale)).collect::<Vec<_>>();
            let lengths = segments.iter().map(bezier_length).collect::<Vec<_>>();
            let speed_tables = segments.iter().map(speed_table).collect();
            let total_length = lengths.iter().sum();
            NetworkSpline {
                flags: *spline.flags(),
                segments,
                speed_tables,
                lengths,
                total_length,
            }
        }).collect();

        Self {
            regions: trade_routes.regions().clone(),
            splines,
            region_routes: first_route_per_pair(trade_routes.region_routes()),
            port_routes: first_route_per_pair(trade_routes.port_routes()),
        }
    }
}

/// Keeps the first route listed for each (from, to) pair, in order of first appearance.
fn first_route_per_pair(routes: &[TradeRoute]) -> Vec<NetworkRoute> {
    let mut seen = HashSet::new();
    routes.iter()
        .filter(|route| seen.insert((*route.from(), *route.to())))
        .map(|route| NetworkRoute {
            from: *route.from(),
            to: *route.to(),
            edges: route.edges().clone(),
        })
        .collect()
}

/// Returns the four Bezier control points of a segment in world coordinates: start, start + first control,
/// end + second control, end.
fn control_points_world(segment: &TradeSplineSegment, scale: &MapScale) -> [(f32, f32); 4] {
    let start = scale.raw_to_world(f32::from(segment.start().0), f32::from(segment.start().1));
    let end = scale.raw_to_world(f32::from(segment.end().0), f32::from(segment.end().1));
    let start_offset = scale.raw_offset_to_world(segment.start_control().0, segment.start_control().1);
    let end_offset = scale.raw_offset_to_world(segment.end_control().0, segment.end_control().1);
    [
        start,
        (start.0 + start_offset.0, start.1 + start_offset.1),
        (end.0 + end_offset.0, end.1 + end_offset.1),
        end,
    ]
}

/// Returns the speed of a cubic Bezier curve (the length of its derivative) at `t`.
fn bezier_speed(points: &[(f32, f32); 4], t: f64) -> f64 {
    let [p0, p1, p2, p3] = points.map(|(x, y)| (f64::from(x), f64::from(y)));
    let u = 1.0 - t;
    let dx = 3.0 * u * u * (p1.0 - p0.0) + 6.0 * u * t * (p2.0 - p1.0) + 3.0 * t * t * (p3.0 - p2.0);
    let dy = 3.0 * u * u * (p1.1 - p0.1) + 6.0 * u * t * (p2.1 - p1.1) + 3.0 * t * t * (p3.1 - p2.1);
    (dx * dx + dy * dy).sqrt()
}

/// Returns the speed table of a segment: the running sum of its speed at `t = k / 100`. The last entry is close to
/// 100 times its length.
fn speed_table(points: &[(f32, f32); 4]) -> Vec<f32> {
    let mut sum = 0.0;
    (0..SPEED_SAMPLES).map(|sample| {
        sum += bezier_speed(points, sample as f64 / SPEED_SAMPLES as f64);
        sum as f32
    }).collect()
}

/// Returns the length of a cubic Bezier curve, integrating its speed with Gauss-Legendre quadrature.
fn bezier_length(points: &[(f32, f32); 4]) -> f32 {
    let speed = |t: f64| bezier_speed(points, t);

    let step = 1.0 / LENGTH_SUBINTERVALS as f64;
    let length: f64 = (0..LENGTH_SUBINTERVALS).map(|interval| {
        let start = interval as f64 * step;
        GAUSS_NODES.iter().zip(GAUSS_WEIGHTS.iter())
            .map(|(node, weight)| weight * speed(start + (node + 1.0) * step / 2.0))
            .sum::<f64>() * step / 2.0
    }).sum();

    length as f32
}

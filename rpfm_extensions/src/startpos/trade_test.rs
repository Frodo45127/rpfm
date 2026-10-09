//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;

use rpfm_lib::files::Decodeable;
use rpfm_lib::files::esf::{NodeType, RecordNode, ESF};

use super::*;
use crate::startpos::esf_nav::{as_record, child_record};
use crate::startpos::startpos_test::{test_data_esf, test_data_path};

#[test]
fn test_bezier_length_straight_line() {
    let line = [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)];

    assert!((bezier_length(&line) - 3.0).abs() < 1e-5);
}

#[test]
fn test_bezier_length_first_attila_spline() {

    // First spline of Attila's main campaign: the game stores 13.893966 as its length.
    let segment = [(482.63748, 41.295902), (482.63748, 45.46409), (482.63748, 51.021664), (482.63748, 55.18985)];

    assert!((bezier_length(&segment) - 13.893966).abs() < 1e-3);
}

#[test]
fn test_control_points_world() {

    // First spline of Attila's main campaign: raw points from its trade_routes.ptd, world points from its startpos.
    let mut segment = TradeSplineSegment::default();
    segment.set_start((4336, 432));
    segment.set_start_control((0, 4320));
    segment.set_end_control((0, -4320));
    segment.set_end((4336, 576));

    let expected = [(482.63748, 41.295902), (482.63748, 45.46409), (482.63748, 51.021664), (482.63748, 55.18985)];
    for (point, expected) in control_points_world(&segment).iter().zip(expected.iter()) {
        assert!((point.0 - expected.0).abs() < 1e-3 && (point.1 - expected.1).abs() < 1e-3, "{point:?} != {expected:?}");
    }
}

#[test]
fn test_first_route_per_pair() {
    let route = |from, to, edge| TradeRoute::new(from, to, vec![TradeRouteEdge::new(edge)]);
    let routes = vec![route(1, 2, 0), route(1, 2, 4), route(2, 1, 1), route(1, 3, 6)];
    let kept = first_route_per_pair(&routes);

    assert_eq!(kept.iter().map(|route| (*route.from(), *route.to(), route.edges()[0].value())).collect::<Vec<_>>(), vec![(1, 2, 0), (2, 1, 1), (1, 3, 6)]);
}

/// Returns the game-built trade manager record.
fn startpos_trade_manager(startpos: &ESF) -> RecordNode {
    let root = as_record(startpos.root_node()).unwrap();
    let model = child_record(child_record(root, "CAMPAIGN_ENV").unwrap(), "CAMPAIGN_MODEL").unwrap();
    child_record(model, "CAMPAIGN_TRADE_MANAGER").unwrap().clone()
}

/// Reads the (from, to, edges) routes of a startpos route record.
fn startpos_routes(manager: &RecordNode, name: &str) -> BTreeSet<(u32, u32, Vec<u32>)> {
    child_record(manager, name).unwrap().children().iter().map(|group| {
        let number = |index: usize| match &group[index] { NodeType::U32(value) => *value.value(), other => panic!("{other:?}") };
        let edges = match &group[2] { NodeType::U32Array(value) => value.value().clone(), other => panic!("{other:?}") };
        (number(0), number(1), edges)
    }).collect()
}

#[test]
#[ignore = "needs RPFM_STARTPOS_TEST_DATA with trade_routes.ptd and startpos.esf"]
fn test_matches_game_startpos() {
    let trade_routes = TradeRoutes::decode(&mut BufReader::new(File::open(test_data_path("trade_routes.ptd")).unwrap()), &None).unwrap();
    let network = TradeNetwork::from_trade_routes(&trade_routes);

    assert_eq!(trade_routes.regions().len(), 187);
    assert_eq!(trade_routes.region_routes().len(), 1137);
    let manager = startpos_trade_manager(&test_data_esf("startpos.esf"));

    let game_splines = child_record(&manager, "SPLINES").unwrap().children();
    assert_eq!(network.splines().len(), game_splines.len());
    for (spline, group) in network.splines().iter().zip(game_splines.iter()) {
        let flags = group[..4].iter().map(|node| match node { NodeType::Bool(value) => *value.value(), other => panic!("{other:?}") }).collect::<Vec<_>>();
        assert_eq!(flags, spline.flags().to_vec());

        // Per segment: 4 points and a table. Then lengths, cumulative starts, total length and a constant.
        for (index, segment) in spline.segments().iter().enumerate() {
            for (point, node) in segment.iter().zip(group[5 + index * 5..].iter()) {
                let NodeType::Coord2d(game) = node else { panic!("{node:?}") };
                assert!((point.0 - game.x()).abs() < 1e-3 && (point.1 - game.y()).abs() < 1e-3);
            }
        }
        let NodeType::F32(total) = &group[5 + spline.segments().len() * 5 + 2] else { panic!() };
        assert!((spline.total_length() - total.value()).abs() < 1e-3 * total.value().max(1.0));
    }

    let to_set = |routes: &[NetworkRoute]| routes.iter().map(|route| (*route.from(), *route.to(), route.edges().iter().map(|edge| edge.value()).collect())).collect::<BTreeSet<_>>();
    assert_eq!(to_set(network.region_routes()), startpos_routes(&manager, "REGION_ROUTES"));
    assert_eq!(to_set(network.port_routes()), startpos_routes(&manager, "PORT_ROUTES"));
}

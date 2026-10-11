//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;

use rpfm_lib::files::Decodeable;
use rpfm_lib::files::trade_routes::TradeRoutes;

use super::*;
use crate::startpos::esf_nav::{as_record, as_u32, child_record};
use crate::startpos::startpos_test::{test_data_path, test_data_tables, test_startpos};
use crate::startpos::world::writer::writer_test::{game_plan, world_test_data};

/// Paths with as many steps as the game's but another tie-break.
const MAX_OTHER_PATHS: usize = 5;

/// Returns the routes of an `INTERNATIONAL_TRADE_ROUTES` record by (owner id, partner region at the path's end), with
/// their ids and groups.
fn routes_by_ends(record: &RecordNode) -> BTreeMap<(u32, u32), (u32, Vec<NodeType>)> {
    let mut routes = BTreeMap::new();
    for entry in record.children() {
        let owner = as_u32(&entry[0]).unwrap();
        let array = entry.iter().find_map(as_record).unwrap();
        for route in array.children().iter().flatten().filter_map(as_record) {
            let group = route.children()[0].clone();
            let NodeType::U32Array(path) = &group[1] else { panic!("a route without a path") };
            routes.insert((owner, *path.value().last().unwrap()), (as_u32(&group[0]).unwrap(), group));
        }
    }
    routes
}

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_international_routes_match_the_game_startpos() {
    let (index, layout, _, _, _) = world_test_data();
    let plan = game_plan(&index);
    let trade_routes = TradeRoutes::decode(&mut BufReader::new(File::open(test_data_path("trade_routes.ptd")).unwrap()), &None).unwrap();
    let network = TradeNetwork::from_trade_routes(&trade_routes);

    let written = write_international_routes(&layout, test_data_tables(), &index, &plan, &network).unwrap();
    let NodeType::Record(root) = test_startpos().root_node() else { panic!("the startpos root isn't a record") };
    let model = child_record(child_record(root, "CAMPAIGN_ENV").unwrap(), "CAMPAIGN_MODEL").unwrap();
    let game = child_record(child_record(model, "CAMPAIGN_TRADE_MANAGER").unwrap(), "INTERNATIONAL_TRADE_ROUTES").unwrap();

    // Every faction has its entry, in the same order, and the same routes (by owner and destination) exist on both sides.
    let entry_owners = |record: &RecordNode| record.children().iter().map(|entry| as_u32(&entry[0]).unwrap()).collect::<Vec<_>>();
    assert_eq!(entry_owners(&written), entry_owners(game));
    let (ours, theirs) = (routes_by_ends(&written), routes_by_ends(game));
    assert_eq!(ours.keys().collect::<Vec<_>>(), theirs.keys().collect::<Vec<_>>());

    // Ids aside, routes match but for the paths of tied lengths and the float of routes by sea.
    let mut other_paths = 0;
    for (key, (_, our_group)) in &ours {
        let their_group = &theirs[key].1;
        assert_eq!(our_group.len(), their_group.len(), "shape of the route {key:?}");
        if our_group[1] != their_group[1] {
            let (NodeType::U32Array(ours), NodeType::U32Array(theirs)) = (&our_group[1], &their_group[1]) else { unreachable!() };
            assert_eq!(ours.value().len(), theirs.value().len(), "steps of the route {key:?}");
            other_paths += 1;
            continue;
        }
        let is_sea = matches!(our_group[2], NodeType::Bool(ref value) if *value.value());
        for (position, (our_node, their_node)) in our_group.iter().zip(their_group).enumerate().skip(2) {
            if !(is_sea && position == 7) {
                assert_eq!(our_node, their_node, "field {position} of the route {key:?}");
            }
        }
    }
    assert!(other_paths <= MAX_OTHER_PATHS, "{other_paths} paths differ");

    // Ids number the routes from 1, the two routes of an agreement one after the other.
    let mut ids = ours.values().map(|(id, _)| *id).collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!(ids, (1..=theirs.len() as u32).collect::<Vec<_>>());
}

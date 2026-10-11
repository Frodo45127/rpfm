//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use rpfm_lib::files::esf::NodeType;

use super::*;
use crate::startpos::esf_nav::child_record;
use crate::startpos::startpos_test::test_startpos;

#[test]
#[ignore = "needs the game data, see startpos_test.rs"]
fn test_counters_match_the_game_startpos() {
    let layout = StartposLayout::from_esf(test_startpos()).unwrap();
    let root = test_startpos().root_node();
    let NodeType::Record(root) = root else { panic!("the startpos root isn't a record") };
    let model = child_record(child_record(root, "CAMPAIGN_ENV").unwrap(), "CAMPAIGN_MODEL").unwrap();
    let sections = ["WORLD", "CAMPAIGN_TRADE_MANAGER"].map(|name| child_record(model, name).unwrap());

    let written = write_index_tables(&layout, &sections).unwrap();
    assert_eq!(&written, child_record(model, "CAMPAIGN_COMMAND_QUEUE_INDEX_TABLES").unwrap());
}

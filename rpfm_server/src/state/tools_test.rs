//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the tools' options.

use super::*;

#[test]
fn optimizer_options_change_only_the_named_ones() {
    let base = OptimizerOptions::default();
    let base_values = optimizer_option_values(&base);
    assert!(base_values.contains_key("table_remove_itm_entries"));

    let flipped = !base_values["table_remove_itm_entries"];
    let changed = optimizer_options_with(&base, &BTreeMap::from([("table_remove_itm_entries".to_owned(), flipped)])).unwrap();
    let changed_values = optimizer_option_values(&changed);

    assert_eq!(changed_values["table_remove_itm_entries"], flipped);
    assert!(changed_values.iter().filter(|(name, _)| *name != "table_remove_itm_entries").all(|(name, value)| base_values[name] == *value));

    assert!(matches!(optimizer_options_with(&base, &BTreeMap::from([("nope".to_owned(), true)])), Err(ApiError::InvalidParams(_))));
}

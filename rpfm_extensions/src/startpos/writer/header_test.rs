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

#[test]
fn test_full_names_join_the_localised_names() {
    let texts = HashMap::from([
        ("names_name_1".to_owned(), "Alfred".to_owned()),
        ("names_name_2".to_owned(), "of Wessex".to_owned()),
        ("names_name_3".to_owned(), String::new()),
    ]);
    let names = |ids: &[&str]| ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();

    assert_eq!(full_name(&texts, &names(&["1", "", "", ""])), "Alfred");
    assert_eq!(full_name(&texts, &names(&["1", "2", "3", "4"])), "Alfred of Wessex");
}

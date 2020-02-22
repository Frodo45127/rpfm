//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `GlobalSearchUI`.
!*/

use crate::locale::qtr;
use super::GlobalSearchUI;

/// This function sets the status bar tip for all the actions in the provided `GlobalSearchUI`.
pub fn set_tips(global_search_ui: &mut GlobalSearchUI) {

    //---------------------------------------------------//
    // Global Search panel tips.
    //---------------------------------------------------//
    unsafe { global_search_ui.global_search_use_regex_checkbox.set_status_tip(&qtr("tt_global_search_use_regex_checkbox")); }
    unsafe { global_search_ui.global_search_case_sensitive_checkbox.set_status_tip(&qtr("tt_global_search_case_sensitive_checkbox")); }
    unsafe { global_search_ui.global_search_search_on_all_checkbox.set_status_tip(&qtr("tt_global_search_search_on_all_checkbox")); }
    unsafe { global_search_ui.global_search_search_on_dbs_checkbox.set_status_tip(&qtr("tt_global_search_search_on_dbs_checkbox")); }
    unsafe { global_search_ui.global_search_search_on_locs_checkbox.set_status_tip(&qtr("tt_global_search_search_on_locs_checkbox")); }
    unsafe { global_search_ui.global_search_search_on_texts_checkbox.set_status_tip(&qtr("tt_global_search_search_on_texts_checkbox")); }
    unsafe { global_search_ui.global_search_search_on_schemas_checkbox.set_status_tip(&qtr("tt_global_search_search_on_schemas_checkbox")); }
}

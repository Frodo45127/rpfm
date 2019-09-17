//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use crate::QString;
use super::GlobalSearchUI;

/// This function sets the status bar tip for all the actions in the provided `GlobalSearchUI`.
pub fn set_tips(global_search_ui: &GlobalSearchUI) {

    //---------------------------------------------------//
    // Global Search panel tips.
    //---------------------------------------------------//
    unsafe { global_search_ui.global_search_use_regex_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Enable search using Regex. Keep in mind that RPFM will fallback to a normal pattern search if the provided Regex is invalid.")); }
    unsafe { global_search_ui.global_search_case_sensitive_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Enable case sensitive search. Pretty self-explanatory.")); }
    unsafe { global_search_ui.global_search_search_on_all_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Include all searchable PackedFiles/Schemas on the search.")); }
    unsafe { global_search_ui.global_search_search_on_dbs_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Include DB Tables on the search.")); }
    unsafe { global_search_ui.global_search_search_on_locs_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Include LOC Tables on the search.")); }
    unsafe { global_search_ui.global_search_search_on_texts_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Include any kind of Text PackedFile on the search.")); }
    unsafe { global_search_ui.global_search_search_on_schemas_checkbox.as_mut().unwrap().set_status_tip(&QString::from_std_str("Include the currently loaded Schema on the search.")); }
}
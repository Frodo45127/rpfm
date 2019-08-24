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
Module with all the code to setup the tips (as tooltips) for the actions in `SettingsUI`.
!*/

use crate::QString;
use crate::settings_ui::SettingsUI;

/// This function sets the status bar tip for all the actions in the provided `SettingsUI`.
pub fn set_tips(settings_ui: &SettingsUI) {

    //-----------------------------------------------//
    // `UI` tips.
    //-----------------------------------------------//
    let ui_global_use_dark_theme_tip = QString::from_std_str("<i>Ash nazg durbatulûk, ash nazg gimbatul, ash nazg thrakatulûk, agh burzum-ishi krimpatul</i>");
    
    let ui_table_adjust_columns_to_content_tip = QString::from_std_str("If you enable this, when you open a DB Table or Loc File, all columns will be automatically resized depending on their content's size.\nOtherwise, columns will have a predefined size. Either way, you'll be able to resize them manually after the initial resize.\nNOTE: This can make very big tables take more time to load.");
    let ui_table_disable_combos_tip = QString::from_std_str("If you disable this, no more combos will be shown in referenced columns in tables. This means no combos nor autocompletion on DB Tables.\nNow shut up Baldy.");
    let ui_table_extend_last_column_tip = QString::from_std_str("If you enable this, the last column on DB Tables and Loc PackedFiles will extend itself to fill the empty space at his right, if there is any.");
    let ui_table_remember_column_sorting_tip = QString::from_std_str("Enable this to make RPFM remember for what column was a DB Table/LOC sorted when closing it and opening it again.");
    let ui_table_remember_column_visual_order_tip = QString::from_std_str("Enable this to make RPFM remember the visual order of the columns of a DB Table/LOC, when closing it and opening it again.");
    let ui_table_remember_table_state_permanently_tip = QString::from_std_str("If you enable this, RPFM will remember the state of a DB Table or Loc PackedFile (filter data, columns moved, what column was sorting the Table,...) even when you close RPFM and open it again. If you don't want this behavior, leave this disabled.");

    let ui_window_start_maximized_tip = QString::from_std_str("If you enable this, RPFM will start maximized.");

    unsafe { settings_ui.ui_global_use_dark_theme_label.as_mut().unwrap().set_tool_tip(&ui_global_use_dark_theme_tip); }
    unsafe { settings_ui.ui_global_use_dark_theme_checkbox.as_mut().unwrap().set_tool_tip(&ui_global_use_dark_theme_tip); }
    unsafe { settings_ui.ui_table_adjust_columns_to_content_label.as_mut().unwrap().set_tool_tip(&ui_table_adjust_columns_to_content_tip); }
    unsafe { settings_ui.ui_table_adjust_columns_to_content_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_adjust_columns_to_content_tip); }
    unsafe { settings_ui.ui_table_disable_combos_label.as_mut().unwrap().set_tool_tip(&ui_table_disable_combos_tip); }
    unsafe { settings_ui.ui_table_disable_combos_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_disable_combos_tip); }
    unsafe { settings_ui.ui_table_extend_last_column_label.as_mut().unwrap().set_tool_tip(&ui_table_extend_last_column_tip); }
    unsafe { settings_ui.ui_table_extend_last_column_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_extend_last_column_tip); }
    unsafe { settings_ui.ui_table_remember_column_sorting_label.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_sorting_tip); }
    unsafe { settings_ui.ui_table_remember_column_sorting_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_sorting_tip); }
    unsafe { settings_ui.ui_table_remember_column_visual_order_label.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_visual_order_tip); }
    unsafe { settings_ui.ui_table_remember_column_visual_order_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_remember_column_visual_order_tip); }
    unsafe { settings_ui.ui_table_remember_table_state_permanently_label.as_mut().unwrap().set_tool_tip(&ui_table_remember_table_state_permanently_tip); }
    unsafe { settings_ui.ui_table_remember_table_state_permanently_checkbox.as_mut().unwrap().set_tool_tip(&ui_table_remember_table_state_permanently_tip); }
    unsafe { settings_ui.ui_window_start_maximized_label.as_mut().unwrap().set_tool_tip(&ui_window_start_maximized_tip); }
    unsafe { settings_ui.ui_window_start_maximized_checkbox.as_mut().unwrap().set_tool_tip(&ui_window_start_maximized_tip); }

    //-----------------------------------------------//
    // `Extra` tips.
    //-----------------------------------------------//

    let extra_network_check_updates_on_start_tip = QString::from_std_str("If you enable this, RPFM will check for updates at the start of the program, and inform you if there is any update available.\nWhether download it or not is up to you.");
    let extra_network_check_schema_updates_on_start_tip = QString::from_std_str("If you enable this, RPFM will check for schema updates at the start of the program,\nand allow you to automatically download it if there is any update available.");
    let extra_packfile_allow_editing_of_ca_packfiles_tip = QString::from_std_str("By default, only PackFiles of Type 'Mod' and 'Movie' are editables, as those are the only ones used for modding.\nIf you enable this, you'll be able to edit 'Boot', 'Release' and 'Patch' PackFiles too. Just be careful of not writing over one of the game's original PackFiles!");
    let extra_packfile_optimize_not_renamed_packedfiles_tip = QString::from_std_str("If you enable this, when running the 'Optimize PackFile' feature RPFM will optimize Tables and Locs that have the same name as their vanilla counterparts.\nUsually, those files are intended to fully override their vanilla counterparts, so by default (this setting off) they are ignored by the optimizer. But it can be useful sometimes to optimize them too (AssKit including too many files), so that's why this setting exists.");
    let extra_packfile_use_dependency_checker_tip = QString::from_std_str("If you enable this, when opening a DB Table RPFM will try to get his dependencies and mark all cells with a reference to another table as 'Not Found In Table' (Red), 'Referenced Table Not Found' (Blue) or 'Correct Reference' (Black). It makes opening a big table a bit slower.");
    let extra_packfile_use_lazy_loading_tip = QString::from_std_str("If you enable this, PackFiles will load their data on-demand from the disk instead of loading the entire PackFile to Ram. This reduces Ram usage by a lot, but if something else changes/deletes the PackFile while it's open, the PackFile will likely be unrecoverable and you'll lose whatever is in it.\nIf you mainly mod in Warhammer 2's /data folder LEAVE THIS DISABLED, as a bug in the Assembly Kit causes PackFiles to become broken/be deleted when you have this enabled.");
    
    unsafe { settings_ui.extra_network_check_updates_on_start_label.as_mut().unwrap().set_tool_tip(&extra_network_check_updates_on_start_tip); }
    unsafe { settings_ui.extra_network_check_updates_on_start_checkbox.as_mut().unwrap().set_tool_tip(&extra_network_check_updates_on_start_tip); }
    unsafe { settings_ui.extra_network_check_schema_updates_on_start_label.as_mut().unwrap().set_tool_tip(&extra_network_check_schema_updates_on_start_tip); }
    unsafe { settings_ui.extra_network_check_schema_updates_on_start_checkbox.as_mut().unwrap().set_tool_tip(&extra_network_check_schema_updates_on_start_tip); }
    unsafe { settings_ui.extra_packfile_allow_editing_of_ca_packfiles_label.as_mut().unwrap().set_tool_tip(&extra_packfile_allow_editing_of_ca_packfiles_tip); }
    unsafe { settings_ui.extra_packfile_allow_editing_of_ca_packfiles_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_allow_editing_of_ca_packfiles_tip); }
    unsafe { settings_ui.extra_packfile_optimize_not_renamed_packedfiles_label.as_mut().unwrap().set_tool_tip(&extra_packfile_optimize_not_renamed_packedfiles_tip); }
    unsafe { settings_ui.extra_packfile_optimize_not_renamed_packedfiles_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_optimize_not_renamed_packedfiles_tip); }
    unsafe { settings_ui.extra_packfile_use_dependency_checker_label.as_mut().unwrap().set_tool_tip(&extra_packfile_use_dependency_checker_tip); }
    unsafe { settings_ui.extra_packfile_use_dependency_checker_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_use_dependency_checker_tip); }
    unsafe { settings_ui.extra_packfile_use_lazy_loading_label.as_mut().unwrap().set_tool_tip(&extra_packfile_use_lazy_loading_tip); }
    unsafe { settings_ui.extra_packfile_use_lazy_loading_checkbox.as_mut().unwrap().set_tool_tip(&extra_packfile_use_lazy_loading_tip); }

    //-----------------------------------------------//
    // `Debug` tips.
    //-----------------------------------------------//
    let debug_check_for_missing_table_definitions_tip = QString::from_std_str("If you enable this, RPFM will try to decode EVERY TABLE in the current PackFile when opening it or when changing the Game Selected, and it'll output all the tables without an schema to a \"missing_table_definitions.txt\" file.\nDEBUG FEATURE, VERY SLOW. DON'T ENABLE IT UNLESS YOU REALLY WANT TO USE IT.");
    
    unsafe { settings_ui.debug_check_for_missing_table_definitions_label.as_mut().unwrap().set_tool_tip(&debug_check_for_missing_table_definitions_tip); }
    unsafe { settings_ui.debug_check_for_missing_table_definitions_checkbox.as_mut().unwrap().set_tool_tip(&debug_check_for_missing_table_definitions_tip); }
}
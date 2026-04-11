//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to connect `PackFileContentsUI` signals with their corresponding slots.

This module is, and should stay, private, as it's only glue between the `PackFileContentsUI` and `PackFileContentsSlots` structs.
!*/

use rpfm_ipc::settings_keys::*;

use crate::ffi::draggable_file_tree_view_drop_signal;
use crate::settings_ui::backend::settings_bool;

use super::{PackFileContentsUI, slots::PackFileContentsSlots};

/// This function connects all the actions from the provided `PackFileContentsUI` with their slots in `PackFileContentsSlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not pollute the other modules with a ton of connections.
pub unsafe fn set_connections(ui: &PackFileContentsUI, slots: &PackFileContentsSlots) {
    if settings_bool(DISABLE_FILE_PREVIEWS) {
        ui.packfile_contents_tree_view.selection_model().selection_changed().connect(&slots.open_packedfile_full);
    } else {
        ui.packfile_contents_tree_view.selection_model().selection_changed().connect(&slots.open_packedfile_preview);
    }
    ui.packfile_contents_tree_view.double_clicked().connect(&slots.open_packedfile_full);

    if settings_bool(ENABLE_PACK_CONTENTS_DRAG_AND_DROP) {
        draggable_file_tree_view_drop_signal(ui.packfile_contents_tree_view.static_upcast()).connect(&slots.move_items);
    }

    // Trigger the filter whenever the "filtered" text or any of his settings changes.
    ui.filter_timer_delayed_updates.timeout().connect(&slots.filter_trigger);
    ui.filter_line_edit.text_changed().connect(&slots.filter_change_text);
    ui.filter_autoexpand_matches_button.toggled().connect(&slots.filter_change_autoexpand_matches);
    ui.filter_case_sensitive_button.toggled().connect(&slots.filter_change_case_sensitive);
    ui.filter_line_edit.text_changed().connect(&slots.filter_check_regex);

    ui.packfile_contents_tree_view.custom_context_menu_requested().connect(&slots.contextual_menu);
    ui.packfile_contents_tree_view.selection_model().selection_changed().connect(&slots.contextual_menu_enabler);
    ui.packfile_contents_tree_view_context_menu.about_to_show().connect(&slots.contextual_menu_enabler);

    ui.context_menu_add_file.triggered().connect(&slots.contextual_menu_add_file);
    ui.context_menu_add_folder.triggered().connect(&slots.contextual_menu_add_folder);
    ui.context_menu_copy_to_pack.about_to_show().connect(&slots.contextual_menu_copy_to_pack_about_to_show);
    ui.context_menu_copy_to_pack.triggered().connect(&slots.contextual_menu_copy_to_pack);
    ui.context_menu_delete.triggered().connect(&slots.contextual_menu_delete);
    ui.context_menu_extract.triggered().connect(&slots.contextual_menu_extract);
    ui.context_menu_rename.triggered().connect(&slots.contextual_menu_rename);
    ui.context_menu_copy_path.triggered().connect(&slots.contextual_menu_copy_path);
    ui.context_menu_copy.triggered().connect(&slots.contextual_menu_copy);
    ui.context_menu_cut.triggered().connect(&slots.contextual_menu_cut);
    ui.context_menu_paste.triggered().connect(&slots.contextual_menu_paste);
    ui.context_menu_duplicate.triggered().connect(&slots.contextual_menu_duplicate);

    ui.context_menu_new_folder.triggered().connect(&slots.contextual_menu_new_folder);
    ui.context_menu_new_packed_file_anim_pack.triggered().connect(&slots.contextual_menu_new_packed_file_anim_pack);
    ui.context_menu_new_packed_file_db.triggered().connect(&slots.contextual_menu_new_packed_file_db);
    ui.context_menu_new_packed_file_loc.triggered().connect(&slots.contextual_menu_new_packed_file_loc);
    ui.context_menu_new_packed_file_portrait_settings.triggered().connect(&slots.contextual_menu_new_packed_file_portrait_settings);
    ui.context_menu_new_packed_file_text.triggered().connect(&slots.contextual_menu_new_packed_file_text);
    ui.context_menu_new_queek_packed_file.triggered().connect(&slots.contextual_menu_new_queek_packed_file);

    ui.context_menu_open_decoder.triggered().connect(&slots.contextual_menu_open_decoder);
    ui.context_menu_open_dependency_manager.triggered().connect(&slots.contextual_menu_open_dependency_manager);
    ui.context_menu_open_containing_folder.triggered().connect(&slots.contextual_menu_open_containing_folder);
    ui.context_menu_open_with_external_program.triggered().connect(&slots.contextual_menu_open_in_external_program);
    ui.context_menu_open_packfile_settings.triggered().connect(&slots.contextual_menu_open_packfile_settings);
    ui.context_menu_open_notes.triggered().connect(&slots.contextual_menu_open_notes);

    ui.context_menu_merge_tables.triggered().connect(&slots.contextual_menu_tables_merge_tables);
    ui.context_menu_update_table.triggered().connect(&slots.contextual_menu_tables_update_table);
    ui.context_menu_generate_missing_loc_data.triggered().connect(&slots.contextual_menu_generate_missing_loc_data);

    ui.context_menu_install.triggered().connect(&slots.context_menu_install);
    ui.context_menu_uninstall.triggered().connect(&slots.context_menu_uninstall);
    ui.context_menu_packfile_type_boot.triggered().connect(&slots.context_menu_change_packfile_type);
    ui.context_menu_packfile_type_release.triggered().connect(&slots.context_menu_change_packfile_type);
    ui.context_menu_packfile_type_patch.triggered().connect(&slots.context_menu_change_packfile_type);
    ui.context_menu_packfile_type_mod.triggered().connect(&slots.context_menu_change_packfile_type);
    ui.context_menu_packfile_type_movie.triggered().connect(&slots.context_menu_change_packfile_type);
    ui.context_menu_index_includes_timestamp.triggered().connect(&slots.context_menu_index_includes_timestamp);
    ui.context_menu_compression_none.triggered().connect(&slots.context_menu_change_compression_format);
    ui.context_menu_compression_lzma1.triggered().connect(&slots.context_menu_change_compression_format);
    ui.context_menu_compression_lz4.triggered().connect(&slots.context_menu_change_compression_format);
    ui.context_menu_compression_zstd.triggered().connect(&slots.context_menu_change_compression_format);
    ui.context_menu_optimize_packfile.triggered().connect(&slots.context_menu_optimize_packfile);
    ui.context_menu_patch_siege_ai.triggered().connect(&slots.context_menu_patch_siege_ai);
    ui.context_menu_live_export.triggered().connect(&slots.context_menu_live_export);
    ui.context_menu_pack_map.triggered().connect(&slots.context_menu_pack_map);
    ui.context_menu_rescue_packfile.triggered().connect(&slots.context_menu_rescue_packfile);
    ui.context_menu_build_starpos.triggered().connect(&slots.context_menu_build_starpos);
    ui.context_menu_update_anim_ids.triggered().connect(&slots.context_menu_update_anim_ids);

    ui.context_menu_mymod_import.triggered().connect(&slots.context_menu_mymod_import);
    ui.context_menu_mymod_export.triggered().connect(&slots.context_menu_mymod_export);
    ui.context_menu_mymod_delete.triggered().connect(&slots.context_menu_mymod_delete);
    ui.context_menu_mymod_open_folder.triggered().connect(&slots.context_menu_mymod_open_folder);

    ui.packfile_contents_tree_view_expand_all.triggered().connect(&slots.packfile_contents_tree_view_expand_all);
    ui.packfile_contents_tree_view_collapse_all.triggered().connect(&slots.packfile_contents_tree_view_collapse_all);

    // Not yet working.
    //ui.packfile_contents_tree_view.expanded().connect(&slots.packfile_contents_resize);
}

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
Module with the slots for AnimFragment views.
!*/

use qt_widgets::SlotOfQPoint;
use qt_widgets::SlotOfIntSortOrder;
use qt_widgets::q_header_view::ResizeMode;

use qt_gui::QCursor;
use qt_gui::QStandardItemModel;
use qt_gui::SlotOfQStandardItem;

use qt_core::QModelIndex;
use qt_core::QItemSelection;
use qt_core::{Slot, SlotOfQItemSelectionQItemSelection};
use qt_core::QSortFilterProxyModel;
use qt_core::QFlags;
use qt_core::q_item_selection_model::SelectionFlag;

use cpp_core::{Ref, MutPtr};

use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::table::TableOperations;
use crate::packedfile_views::table::utils::*;
use crate::packedfile_views::utils::set_modified;
use crate::UI_STATE;
use crate::utils::*;

use super::PackedFileAnimFragmentViewRaw;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of the view of a AnimFragment PackedFile.
pub struct PackedFileAnimFragmentViewSlots {
    pub sort_order_column_changed_1: SlotOfIntSortOrder<'static>,
    pub sort_order_column_changed_2: SlotOfIntSortOrder<'static>,
    pub show_context_menu_1: SlotOfQPoint<'static>,
    pub show_context_menu_2: SlotOfQPoint<'static>,
    pub context_menu_enabler_1: SlotOfQItemSelectionQItemSelection<'static>,
    pub context_menu_enabler_2: SlotOfQItemSelectionQItemSelection<'static>,
    pub item_changed_1: SlotOfQStandardItem<'static>,
    pub item_changed_2: SlotOfQStandardItem<'static>,
    pub add_rows_1: Slot<'static>,
    pub add_rows_2: Slot<'static>,
    pub insert_rows_1: Slot<'static>,
    pub insert_rows_2: Slot<'static>,
    pub delete_rows_1: Slot<'static>,
    pub delete_rows_2: Slot<'static>,
    pub clone_and_append_1: Slot<'static>,
    pub clone_and_append_2: Slot<'static>,
    pub clone_and_insert_1: Slot<'static>,
    pub clone_and_insert_2: Slot<'static>,
    pub copy_1: Slot<'static>,
    pub copy_2: Slot<'static>,
    pub paste_1: Slot<'static>,
    pub paste_2: Slot<'static>,
    pub invert_selection_1: Slot<'static>,
    pub invert_selection_2: Slot<'static>,
    pub reset_selection_1: Slot<'static>,
    pub reset_selection_2: Slot<'static>,
    pub rewrite_selection_1: Slot<'static>,
    pub rewrite_selection_2: Slot<'static>,
    pub save_1: Slot<'static>,
    pub save_2: Slot<'static>,
    pub undo_1: Slot<'static>,
    pub undo_2: Slot<'static>,
    pub redo_1: Slot<'static>,
    pub redo_2: Slot<'static>,
    pub smart_delete_1: Slot<'static>,
    pub smart_delete_2: Slot<'static>,
    pub resize_columns_1: Slot<'static>,
    pub resize_columns_2: Slot<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Macro to generate all the slots for both tables at once.
macro_rules! slot_generator {
    (
        $view:ident,
        $app_ui:ident,
        $pack_file_contents_ui:ident,
        $global_search_ui:ident,

        $sort_order_column_changed:ident,
        $show_context_menu:ident,
        $context_menu_enabler:ident,
        $item_changed:ident,
        $add_rows:ident,
        $insert_rows:ident,
        $delete_rows:ident,
        $clone_and_append:ident,
        $clone_and_insert:ident,
        $copy:ident,
        $paste:ident,
        $invert_selection:ident,
        $reset_selection:ident,
        $rewrite_selection:ident,
        $save:ident,
        $undo:ident,
        $redo:ident,
        $smart_delete:ident,
        $resize_columns:ident,

        $table:ident,
        $column_sort_state:ident,
        $context_menu:ident,
        $context_menu_update:ident,
        $append_rows:ident,
        $history_undo:ident,
        $history_redo:ident,
        $undo_model:ident,
        $copy_selection:ident,
        $undo_redo:ident,
    ) => {


        let $sort_order_column_changed = SlotOfIntSortOrder::new(clone!(
            $view => move |column, _| {
                sort_column($view.$table, column, $view.$column_sort_state.clone());
            }
        ));

        // When we want to show the context menu.
        let $show_context_menu = SlotOfQPoint::new(clone!(
            mut $view => move |_| {
            $view.$context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        // When we want to trigger the context menu update function.
        let $context_menu_enabler = SlotOfQItemSelectionQItemSelection::new(clone!(
            mut $view => move |_,_| {
            $view.$context_menu_update();
        }));

        // When we want to respond to a change in one item in the model.
        let $item_changed = SlotOfQStandardItem::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move |item| {
                $view.$item_changed(item, &mut $app_ui, &mut $pack_file_contents_ui);
            }
        ));

        // When you want to append a row to the table...
        let $add_rows = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
                $view.$append_rows(false);
                set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
            }
        ));

        // When you want to insert a row in a specific position of the table...
        let $insert_rows = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
                $view.$insert_rows(false);
                set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
            }
        ));

        // When you want to delete one or more rows...
        let $delete_rows = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {

                let filter: MutPtr<QSortFilterProxyModel> = $view.$table.model().static_downcast_mut();
                let model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();

                // Get all the selected rows.
                let selection = $view.$table.selection_model().selection();
                let indexes = filter.map_selection_to_source(&selection).indexes();
                let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
                let mut rows_to_delete: Vec<i32> = indexes_sorted.iter().filter_map(|x| if x.is_valid() { Some(x.row()) } else { None }).collect();

                // Dedup the list and reverse it.
                rows_to_delete.sort();
                rows_to_delete.dedup();
                rows_to_delete.reverse();
                let rows_splitted = delete_rows(model, &rows_to_delete);

                // If we deleted something, try to save the PackedFile to the main PackFile.
                if !rows_to_delete.is_empty() {
                    $view.$history_undo.write().unwrap().push(TableOperations::RemoveRows(rows_splitted));
                    $view.$history_redo.write().unwrap().clear();
                    update_undo_model(model, $view.$undo_model);
                    set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
                }
            }
        ));

        // When you want to clone and insert one or more rows.
        let $clone_and_append = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
            $view.$append_rows(true);
            set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
        }));

        // When you want to clone and append one or more rows.
        let $clone_and_insert = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
            $view.$insert_rows(true);
            set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
        }));

        // When you want to copy one or more cells.
        let $copy = Slot::new(clone!(
            $view => move || {
            $view.$copy_selection();
        }));

        // When you want to copy one or more cells.
        let $paste = Slot::new(clone!(
            mut $view => move || {
            $view.$paste();
        }));

        // When we want to invert the selection of the table.
        let $invert_selection = Slot::new(clone!(
            mut $view => move || {
            let filter: MutPtr<QSortFilterProxyModel> = $view.$table.model().static_downcast_mut();
            let rows = filter.row_count_0a();
            let columns = filter.column_count_0a();
            if rows > 0 && columns > 0 {
                let mut selection_model = $view.$table.selection_model();
                let first_item = filter.index_2a(0, 0);
                let last_item = filter.index_2a(rows - 1, columns - 1);
                let selection = QItemSelection::new_2a(&first_item, &last_item);
                selection_model.select_q_item_selection_q_flags_selection_flag(&selection, QFlags::from(SelectionFlag::Toggle));
            }
        }));

        // When we want to reset the selected items of the table to their original value.
        let $reset_selection = Slot::new(clone!(
            mut $view => move || {
            $view.$reset_selection();
        }));

        // When we want to rewrite the selected items using a formula.
        let $rewrite_selection = Slot::new(clone!(
            mut $view => move || {
            $view.$rewrite_selection();
        }));

        // When we want to save the contents of the UI to the backend...
        //
        // NOTE: in-edition saves to backend are only triggered when the GlobalSearch has search data, to keep it updated.
        let $save = Slot::new(clone!(
            $view => move || {
            if !UI_STATE.get_global_search_no_lock().pattern.is_empty() {
                if let Some(packed_file) = UI_STATE.get_open_packedfiles().iter().find(|x| *x.get_ref_path() == *$view.packed_file_path.read().unwrap()) {
                    if let Err(error) = packed_file.save(&mut $app_ui, $global_search_ui, &mut $pack_file_contents_ui) {
                        show_dialog($view.$table, error, false);
                    }
                }
            }
        }));

        // When we want to undo the last action.
        let $undo = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
                let filter: MutPtr<QSortFilterProxyModel> = $view.$table.model().static_downcast_mut();
                let model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
                $view.$undo_redo(true, 0);
                update_undo_model(model, $view.$undo_model);
                $view.$context_menu_update();
                if $view.$history_undo.read().unwrap().is_empty() {
                    set_modified(false, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
                }
            }
        ));

        // When we want to redo the last undone action.
        let $redo = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
                let filter: MutPtr<QSortFilterProxyModel> = $view.$table.model().static_downcast_mut();
                let model: MutPtr<QStandardItemModel> = filter.source_model().static_downcast_mut();
                $view.$undo_redo(false, 0);
                update_undo_model(model, $view.$undo_model);
                $view.$context_menu_update();
                set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
            }
        ));

        // When we want to resize the columns depending on their contents...
        let $resize_columns = Slot::new(clone!($view => move || {
            $view.$table.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
            if SETTINGS.read().unwrap().settings_bool["extend_last_column_on_tables"] {
                $view.$table.horizontal_header().set_stretch_last_section(false);
                $view.$table.horizontal_header().set_stretch_last_section(true);
            }
        }));

        // When you want to use the "Smart Delete" feature...
        let $smart_delete = Slot::new(clone!(
            mut $pack_file_contents_ui,
            mut $view => move || {
                $view.$smart_delete();
                set_modified(true, &$view.packed_file_path.read().unwrap(), &mut $app_ui, &mut $pack_file_contents_ui);
            }
        ));
    }
}

/// Implementation for `PackedFileAnimFragmentViewSlots`.
impl PackedFileAnimFragmentViewSlots {

    /// This function creates the entire slot pack for CaVp8 PackedFile views.
    pub unsafe fn new(
        view: PackedFileAnimFragmentViewRaw,
        mut app_ui: AppUI,
        mut pack_file_contents_ui: PackFileContentsUI,
        global_search_ui: GlobalSearchUI
    )  -> Self {
        slot_generator!(
            view,
            app_ui,
            pack_file_contents_ui,
            global_search_ui,

            sort_order_column_changed_1,
            show_context_menu_1,
            context_menu_enabler_1,
            item_changed_1,
            add_rows_1,
            insert_rows_1,
            delete_rows_1,
            clone_and_append_1,
            clone_and_insert_1,
            copy_1,
            paste_1,
            invert_selection_1,
            reset_selection_1,
            rewrite_selection_1,
            save_1,
            undo_1,
            redo_1,
            smart_delete_1,
            resize_columns_1,

            table_1,
            column_sort_state_1,
            context_menu_1,
            context_menu_update_1,
            append_rows_1,
            history_undo_1,
            history_redo_1,
            undo_model_1,
            copy_selection_1,
            undo_redo_1,
        );

        slot_generator!(
            view,
            app_ui,
            pack_file_contents_ui,
            global_search_ui,

            sort_order_column_changed_2,
            show_context_menu_2,
            context_menu_enabler_2,
            item_changed_2,
            add_rows_2,
            insert_rows_2,
            delete_rows_2,
            clone_and_append_2,
            clone_and_insert_2,
            copy_2,
            paste_2,
            invert_selection_2,
            reset_selection_2,
            rewrite_selection_2,
            save_2,
            undo_2,
            redo_2,
            smart_delete_2,
            resize_columns_2,

            table_2,
            column_sort_state_2,
            context_menu_2,
            context_menu_update_2,
            append_rows_2,
            history_undo_2,
            history_redo_2,
            undo_model_2,
            copy_selection_2,
            undo_redo_2,
        );

        // Return the slots, so we can keep them alive for the duration of the view.
        Self {
            sort_order_column_changed_1,
            sort_order_column_changed_2,
            show_context_menu_1,
            show_context_menu_2,
            context_menu_enabler_1,
            context_menu_enabler_2,
            item_changed_1,
            item_changed_2,
            add_rows_1,
            add_rows_2,
            insert_rows_1,
            insert_rows_2,
            delete_rows_1,
            delete_rows_2,
            clone_and_append_1,
            clone_and_append_2,
            clone_and_insert_1,
            clone_and_insert_2,
            copy_1,
            copy_2,
            paste_1,
            paste_2,
            invert_selection_1,
            invert_selection_2,
            reset_selection_1,
            reset_selection_2,
            rewrite_selection_1,
            rewrite_selection_2,
            save_1,
            save_2,
            undo_1,
            undo_2,
            redo_1,
            redo_2,
            smart_delete_1,
            smart_delete_2,
            resize_columns_1,
            resize_columns_2,
        }
    }
}


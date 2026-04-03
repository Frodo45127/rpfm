//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutierrez Gonzalez. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the command palette UI logic.
//!
//! Provides a VS Code-style command palette with two modes:
//! - File mode (Ctrl+P): lists all files from open packs, filterable by typing.
//! - Command mode (Ctrl+Shift+P): lists all available actions, filterable by typing.

use qt_widgets::QAction;
use qt_widgets::QMenu;

use qt_gui::QStandardItem;

use qt_core::QString;
use qt_core::QPtr;

use rpfm_ipc::helpers::DataSource;
use rpfm_ui_common::icons::IconType;

use std::rc::Rc;

use crate::app_ui::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::ffi::*;
use crate::global_search_ui::GlobalSearchUI;
use crate::pack_tree::PackTree;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::references_ui::ReferencesUI;
use crate::TREEVIEW_ICONS;

/// Shows the command palette in file mode, listing all files from open packs.
pub unsafe fn show_file_palette(
    app_ui: &Rc<AppUI>,
    pack_file_contents_ui: &Rc<PackFileContentsUI>,
    global_search_ui: &Rc<GlobalSearchUI>,
    diagnostics_ui: &Rc<DiagnosticsUI>,
    dependencies_ui: &Rc<DependenciesUI>,
    references_ui: &Rc<ReferencesUI>,
) {
    let parent: QPtr<qt_widgets::QWidget> = app_ui.main_window().static_upcast();
    let palette = new_command_palette_safe(&parent.as_ptr());
    let palette_ptr: QPtr<qt_widgets::QWidget> = QPtr::from_raw(palette.as_mut_raw_ptr());
    command_palette_clear_safe(&palette_ptr);

    // Collect all files from all open editable packs.
    let mut file_paths: Vec<String> = Vec::new();
    let model = pack_file_contents_ui.packfile_contents_tree_model();

    for root_row in 0..model.row_count_0a() {
        let root_item = model.item_1a(root_row);
        let root_type = root_item.data_1a(rpfm_ui_common::ROOT_NODE_TYPE).to_int_0a();

        if root_type != rpfm_ui_common::ROOT_NODE_TYPE_EDITABLE_PACKFILE &&
           root_type != rpfm_ui_common::ROOT_NODE_TYPE_MYMOD_PACKFILE {
            continue;
        }

        collect_files_recursive(root_item, &mut file_paths);
    }

    for path in &file_paths {
        let display = QString::from_std_str(path);
        let empty = QString::new();
        let icon = TREEVIEW_ICONS.icon(IconType::File(path.clone()));
        command_palette_add_item_with_icon_safe(&palette_ptr, &display, &empty, &icon);
    }

    command_palette_show_safe(&palette_ptr);

    let selected = command_palette_selected_index_safe(&palette_ptr);
    if selected >= 0 {
        if let Some(file_path) = file_paths.get(selected as usize) {

            // Expand the tree to the file and select it.
            let tree_view = pack_file_contents_ui.packfile_contents_tree_view();
            let tree_index = tree_view.expand_treeview_to_item(file_path, DataSource::PackFile, "");
            if let Some(ref tree_index) = tree_index {
                if tree_index.is_valid() {
                    tree_view.scroll_to_1a(tree_index.as_ref().unwrap());
                    tree_view.selection_model().select_q_model_index_q_flags_selection_flag(
                        tree_index.as_ref().unwrap(),
                        qt_core::QFlags::from(qt_core::q_item_selection_model::SelectionFlag::ClearAndSelect),
                    );
                }
            }

            AppUI::open_packedfile(
                app_ui,
                pack_file_contents_ui,
                global_search_ui,
                diagnostics_ui,
                dependencies_ui,
                references_ui,
                Some(file_path.clone()),
                false,
                false,
                DataSource::PackFile,
            );
        }
    }
}

/// Shows the command palette in command mode, listing all available actions.
pub unsafe fn show_command_palette(
    app_ui: &Rc<AppUI>,
    pack_file_contents_ui: &Rc<PackFileContentsUI>,
) {
    let parent: QPtr<qt_widgets::QWidget> = app_ui.main_window().static_upcast();
    let palette = new_command_palette_safe(&parent.as_ptr());
    let palette_ptr: QPtr<qt_widgets::QWidget> = QPtr::from_raw(palette.as_mut_raw_ptr());
    command_palette_clear_safe(&palette_ptr);

    let mut actions: Vec<QPtr<QAction>> = Vec::new();

    // Collect actions from the main menu bar.
    let menu_bar = app_ui.main_window().menu_bar();
    let menu_bar_actions = menu_bar.actions();
    for i in 0..menu_bar_actions.count_0a() {
        let menu_action = menu_bar_actions.value_1a(i);
        if menu_action.is_null() { continue; }
        let menu = menu_action.menu();
        if menu.is_null() { continue; }
        let category = menu.title().to_std_string().replace("&", "");
        collect_actions_from_menu(&QPtr::from_raw(menu.as_mut_raw_ptr()), &category, &mut actions, &palette_ptr);
    }

    // Collect actions from the pack tree context menu.
    let tree_context_menu: QPtr<QMenu> = QPtr::from_raw(pack_file_contents_ui.packfile_contents_tree_view_context_menu().as_mut_raw_ptr());
    collect_actions_from_menu(&tree_context_menu, "Pack Tree", &mut actions, &palette_ptr);

    // Collect actions from the open file tab bar context menu.
    let tab_context_menu: QPtr<QMenu> = QPtr::from_raw(app_ui.tab_bar_packed_file_context_menu().as_mut_raw_ptr());
    collect_actions_from_menu(&tab_context_menu, "File Tab", &mut actions, &palette_ptr);

    // Collect actions from the currently active file view's context menu (if any).
    let current_widget = app_ui.tab_bar_packed_file().current_widget();
    if !current_widget.is_null() {
        for file_view in crate::UI_STATE.get_open_packedfiles().iter() {
            if file_view.main_widget().as_mut_raw_ptr() == current_widget.as_mut_raw_ptr() {
                if let crate::packedfile_views::ViewType::Internal(ref view) = *file_view.view_type() {
                    use crate::packedfile_views::View;
                    match view {
                        View::Table(table_view) => {
                            let menu: QPtr<QMenu> = QPtr::from_raw(table_view.get_ref_table().context_menu().as_mut_raw_ptr());
                            collect_actions_from_menu(&menu, "Table", &mut actions, &palette_ptr);
                        },
                        View::Decoder(decoder_view) => {
                            let menu: QPtr<QMenu> = QPtr::from_raw(decoder_view.table_view_context_menu().as_mut_raw_ptr());
                            collect_actions_from_menu(&menu, "Decoder", &mut actions, &palette_ptr);
                        },
                        View::UnitVariant(unit_variant_view) => {
                            let menu: QPtr<QMenu> = QPtr::from_raw(unit_variant_view.main_list_context_menu().as_mut_raw_ptr());
                            collect_actions_from_menu(&menu, "Unit Variant", &mut actions, &palette_ptr);
                        },
                        View::PortraitSettings(portrait_settings_view) => {
                            let menu: QPtr<QMenu> = QPtr::from_raw(portrait_settings_view.main_list_context_menu().as_mut_raw_ptr());
                            collect_actions_from_menu(&menu, "Portrait Settings", &mut actions, &palette_ptr);
                        },
                        _ => {}
                    }
                }
                break;
            }
        }
    }

    command_palette_show_safe(&palette_ptr);

    let selected = command_palette_selected_index_safe(&palette_ptr);
    if selected >= 0 {
        if let Some(action) = actions.get(selected as usize) {
            action.trigger();
        }
    }
}

/// Recursively collect all file paths from a tree item.
unsafe fn collect_files_recursive(
    parent: cpp_core::Ptr<QStandardItem>,
    paths: &mut Vec<String>,
) {
    if parent.is_null() { return; }

    for row in 0..parent.row_count() {
        let child = parent.child_1a(row);
        if child.is_null() { continue; }

        let item_type = child.data_1a(20).to_int_0a(); // ITEM_TYPE = 20

        if item_type == 1 {
            // ITEM_TYPE_FILE = 1 - build path by walking up.
            let mut path_parts: Vec<String> = Vec::new();
            let mut current: cpp_core::Ptr<QStandardItem> = child;
            loop {
                let parent_item = current.parent();
                if parent_item.is_null() { break; }
                path_parts.push(current.text().to_std_string());
                current = parent_item;
                if current.parent().is_null() { break; } // Stop before the root (pack name).
            }
            path_parts.reverse();
            paths.push(path_parts.join("/"));
        } else {
            collect_files_recursive(child, paths);
        }
    }
}

/// Collect all enabled actions from a menu recursively, adding them to the palette widget.
unsafe fn collect_actions_from_menu(
    menu: &QPtr<QMenu>,
    category: &str,
    actions: &mut Vec<QPtr<QAction>>,
    palette_ptr: &QPtr<qt_widgets::QWidget>,
) {
    let menu_actions = menu.actions();
    for i in 0..menu_actions.count_0a() {
        let action = menu_actions.value_1a(i);
        if action.is_null() || action.is_separator() { continue; }

        let submenu = action.menu();
        if !submenu.is_null() {
            let submenu_title = submenu.title().to_std_string().replace("&", "");
            let sub_category = if submenu_title.is_empty() {
                category.to_string()
            } else {
                format!("{} > {}", category, submenu_title)
            };
            collect_actions_from_menu(&QPtr::from_raw(submenu.as_mut_raw_ptr()), &sub_category, actions, palette_ptr);
            continue;
        }

        let action_text = action.text().to_std_string().replace("&", "");
        if action_text.is_empty() { continue; }

        // For actions in submenus (dynamic menus like "Open Recent", "MyMod > Game")
        // or actions that belong to an action group (like game selection radio buttons),
        // prepend the category so the user can identify which menu they belong to.
        let is_grouped = !action.action_group().is_null();
        let text = if category.contains('>') || is_grouped {
            format!("{}: {}", category, action_text)
        } else {
            action_text
        };

        // Build the detail line: status tip (description) if available, otherwise category + shortcut.
        let status_tip = action.status_tip().to_std_string();
        let shortcut = action.shortcut().to_string_0a().to_std_string();
        let detail = if !status_tip.is_empty() {
            if shortcut.is_empty() {
                status_tip
            } else {
                format!("{} [{}]", status_tip, shortcut)
            }
        } else if shortcut.is_empty() {
            category.to_string()
        } else {
            format!("{} [{}]", category, shortcut)
        };

        let display = QString::from_std_str(&text);
        let detail_q = QString::from_std_str(&detail);
        command_palette_add_item_safe(palette_ptr, &display, &detail_q);
        actions.push(QPtr::from_raw(action.as_mut_raw_ptr()));
    }
}

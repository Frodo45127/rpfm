//! CEO Builder tool module.
//!
//! This module contains the CEO Builder dialog (for creating CEO entries in DB tables)
//! and the BOB CEO runner dialog (for building ceo_data.ccd via the Assembly Kit).

use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::ButtonRole;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QListWidgetItem;
use qt_widgets::QListWidget;
use qt_widgets::QPushButton;
use qt_widgets::QTableWidget;
use qt_widgets::QTableWidgetItem;

use qt_core::ItemFlag;
use qt_core::QPtr;
use qt_core::SlotNoArgs;
use qt_core::QString;

use anyhow::Result;

use std::path::PathBuf;
use std::rc::Rc;

use rpfm_ipc::messages::CeoEntryData;
use rpfm_lib::files::ContainerPath;

use rpfm_ui_common::utils::{find_widget, load_template};

use crate::app_ui::AppUI;
use crate::communications::{Command, Response, send_ipc_command_result, send_ipc_command_result_async};
use crate::GAME_SELECTED;
use crate::pack_tree::{PackTree, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::settings_ui::backend::settings_path_buf;
use crate::utils::{qtr, show_dialog};
use crate::UI_STATE;

use rpfm_ipc::helpers::DataSource;

const BUILD_CEO_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/build_ceo_view.ui";
const BUILD_CEO_VIEW_RELEASE: &str = "ui/build_ceo_view.ui";

const BUILD_CEO_BUILDER_VIEW_DEBUG: &str = "rpfm_ui/ui_templates/build_ceo_builder_view.ui";
const BUILD_CEO_BUILDER_VIEW_RELEASE: &str = "ui/build_ceo_builder_view.ui";

/// This function builds CEO data into the open pack via BOB.
pub unsafe fn build_ceo(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {

    // CEO tools are only supported for Three Kingdoms.
    let game_key = GAME_SELECTED.read().unwrap().key().to_owned();
    if game_key != rpfm_lib::games::supported_games::KEY_THREE_KINGDOMS {
        return Err(anyhow::anyhow!("The CEO Builder is only available for Total War: Three Kingdoms."));
    }

    let template_path = if cfg!(debug_assertions) { BUILD_CEO_VIEW_DEBUG } else { BUILD_CEO_VIEW_RELEASE };
    let main_widget = load_template(app_ui.main_window(), template_path)?;
    let dialog = main_widget.static_downcast::<QDialog>();

    let instructions_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "instructions_label")?;
    let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
    let build_ceo_button = button_box.add_button_q_string_button_role(&qtr("build_ceo"), ButtonRole::ActionRole);
    let ceo_done_button = button_box.add_button_q_string_button_role(&qtr("build_ceo_done"), ButtonRole::YesRole);
    ceo_done_button.set_enabled(false);

    dialog.set_window_title(&qtr("build_ceo"));
    instructions_label.set_text(&qtr("build_ceo_instructions"));

    let game = GAME_SELECTED.read().unwrap();
    let akit_path = settings_path_buf(&(game.key().to_owned() + "_assembly_kit"))
        .to_string_lossy().to_string();
    let bob_exe = PathBuf::from(&akit_path).join("binaries").join("bob.modder.x64.exe");
    drop(game);

    if akit_path.is_empty() || !bob_exe.exists() {
        build_ceo_button.set_enabled(false);
    }

    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();

    // Actions — mirror build_starpos exactly:
    // build_ceo_button runs BOB, then enables ceo_done_button.
    // ceo_done_button closes the dialog (exec returns 1).
    // Import happens after exec() returns, outside any slot.
    let dialog_ptr = dialog.as_ptr();
    let build_ceo_button_ptr = build_ceo_button.as_ptr();
    let ceo_done_button_ptr = ceo_done_button.as_ptr();
    let pack_key_closure = pack_key.clone();
    let akit_path_closure = akit_path.clone();
    let bob_exe_str = bob_exe.to_string_lossy().to_string();

    let start_build = SlotNoArgs::new(&dialog, move || {
        build_ceo_button_ptr.set_enabled(false);
        match send_ipc_command_result_async(
            Command::BuildCeo(pack_key_closure.clone(), akit_path_closure.clone(), bob_exe_str.clone()),
            response_extractor!()
        ) {
            Ok(_) => ceo_done_button_ptr.set_enabled(true),
            Err(error) => {
                build_ceo_button_ptr.set_enabled(true);
                show_dialog(dialog_ptr, error, false);
            }
        }
    });

    build_ceo_button.released().connect(&start_build);
    ceo_done_button.released().connect(dialog_ptr.slot_accept());

    // After dialog closes via ceo_done_button, import ceo_data.ccd into the pack.
    if dialog.exec() == 1 {
        let paths = send_ipc_command_result_async(
            Command::BuildCeoPost(pack_key.clone(), akit_path.clone()),
            response_extractor!(Response::VecContainerPath)
        )?;
        if !paths.is_empty() {
            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Add(paths), DataSource::PackFile, &pack_key);
            UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
        }
        Ok(())
    } else if ceo_done_button.is_enabled() {
        Ok(())
    } else {
        Ok(())
    }
}

/// Opens the CEO Builder dialog, letting the user add CEO entries that are
/// inserted directly into the open pack's DB tables and loc file.

/// Opens the CEO Builder dialog, letting the user add CEO entries that are
/// inserted directly into the open pack's DB tables and loc file.
pub unsafe fn build_ceo_builder(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {

    // CEO Builder is only supported for Three Kingdoms.
    let game_key = GAME_SELECTED.read().unwrap().key().to_owned();
    if game_key != rpfm_lib::games::supported_games::KEY_THREE_KINGDOMS {
        return Err(anyhow::anyhow!("The CEO Builder is only available for Total War: Three Kingdoms."));
    }

    let template_path = if cfg!(debug_assertions) { BUILD_CEO_BUILDER_VIEW_DEBUG } else { BUILD_CEO_BUILDER_VIEW_RELEASE };
    let main_widget = load_template(app_ui.main_window(), template_path)?;
    let dialog = main_widget.static_downcast::<QDialog>();

    // ── Find widgets ──────────────────────────────────────────────────────
    let name_line_edit: QPtr<QLineEdit>         = find_widget(&main_widget.static_upcast(), "name_line_edit")?;
    let type_combo_box: QPtr<QComboBox>         = find_widget(&main_widget.static_upcast(), "type_combo_box")?;
    let element_combo_box: QPtr<QComboBox>      = find_widget(&main_widget.static_upcast(), "element_combo_box")?;
    let gender_combo_box: QPtr<QComboBox>       = find_widget(&main_widget.static_upcast(), "gender_combo_box")?;
    let expanded_check_box: QPtr<QCheckBox>     = find_widget(&main_widget.static_upcast(), "expanded_check_box")?;
    let trait_filter_line_edit: QPtr<QLineEdit> = find_widget(&main_widget.static_upcast(), "trait_filter_line_edit")?;
    let trait_count_label: QPtr<QLabel>         = find_widget(&main_widget.static_upcast(), "trait_count_label")?;
    let trait_list_widget: QPtr<QListWidget>    = find_widget(&main_widget.static_upcast(), "trait_list_widget")?;
    let add_character_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "add_character_button")?;
    let clear_all_button: QPtr<QPushButton>     = find_widget(&main_widget.static_upcast(), "clear_all_button")?;
    let delete_selected_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "delete_selected_button")?;
    let status_label: QPtr<QLabel>              = find_widget(&main_widget.static_upcast(), "status_label")?;
    let queue_table_widget: QPtr<QTableWidget>  = find_widget(&main_widget.static_upcast(), "queue_table_widget")?;
    let button_box: QPtr<QDialogButtonBox>      = find_widget(&main_widget.static_upcast(), "button_box")?;

    dialog.set_window_title(&QString::from_std_str("CEO Builder"));

    // ── Populate dropdowns ────────────────────────────────────────────────
    for opt in &["title", "unique"] {
        type_combo_box.add_item_q_string(&QString::from_std_str(opt));
    }
    for el in &["metal", "wood", "earth", "fire", "water"] {
        element_combo_box.add_item_q_string(&QString::from_std_str(el));
    }
    for g in &["male", "female"] {
        gender_combo_box.add_item_q_string(&QString::from_std_str(g));
    }

    // ── Trait data (fetched from dependencies at runtime) ─────────────────
    let trait_ceos: Vec<(String, String)> = match send_ipc_command_result(
        Command::GetTraitCeos,
        response_extractor!(Response::VecStringTuples)
    ) {
        Ok(traits) => traits,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "Failed to load trait CEOs from the Assembly Kit dependencies. \
                 Make sure the dependencies cache has been generated with the Assembly Kit path configured.\n\nError: {}", error
            ));
        }
    };

    if trait_ceos.is_empty() {
        return Err(anyhow::anyhow!(
            "No trait CEOs found in the Assembly Kit data. \
             Make sure Three Kingdoms is selected, the Assembly Kit path is configured, \
             and the dependencies cache has been regenerated."
        ));
    }

    // Populate list widget — store "uuid|key" in tooltip for retrieval
    for (ceo_key, display_name) in &trait_ceos {
        let display = if display_name.is_empty() {
            ceo_key.replace("3k_main_ceo_trait_", "").replace("3k_dlc", "")
                .replace("3k_ytr_ceo_trait_", "").replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                })
                .collect::<Vec<_>>().join(" ")
        } else {
            display_name.clone()
        };

        // Generate a stable UUID from the key via FNV-1a
        let stable_id = format!("{:032x}", {
            let mut hash: u64 = 0xcbf29ce484222325;
            for b in ceo_key.bytes() {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            hash as u128
        });
        let uuid = format!("{}-{}-{}-{}-{}",
            &stable_id[0..8], &stable_id[8..12], &stable_id[12..16],
            &stable_id[16..20], &stable_id[20..32]
        );

        let item = QListWidgetItem::from_q_string(&QString::from_std_str(&display));
        item.set_tool_tip(&QString::from_std_str(&format!("{}|{}", uuid, ceo_key)));
        trait_list_widget.add_item_q_list_widget_item(item.into_ptr());
    }

    // ── Trait filter ──────────────────────────────────────────────────────
    let trait_list_ptr = trait_list_widget.as_ptr();
    let trait_count_ptr = trait_count_label.as_ptr();
    let filter_ptr = trait_filter_line_edit.as_ptr();

    let update_filter = SlotNoArgs::new(&dialog, move || {
        let filter = filter_ptr.text().to_std_string().to_lowercase();
        for i in 0..trait_list_ptr.count() {
            let item = trait_list_ptr.item(i);
            let text = item.text().to_std_string().to_lowercase();
            item.set_hidden(!filter.is_empty() && !text.contains(&filter));
        }
    });
    trait_filter_line_edit.text_changed().connect(&update_filter);

    // Update selected count; disable unselected items when 3 are chosen
    let update_count = SlotNoArgs::new(&dialog, move || {
        let count = trait_list_ptr.selected_items().count();
        trait_count_ptr.set_text(&QString::from_std_str(&format!("Selected: {}/3", count)));
        let enabled_flags = (ItemFlag::ItemIsEnabled | ItemFlag::ItemIsSelectable).into();
        let disabled_flags = ItemFlag::ItemIsSelectable.into(); // no ItemIsEnabled
        for i in 0..trait_list_ptr.count() {
            let item = trait_list_ptr.item(i);
            if count >= 3 && !item.is_selected() {
                item.set_flags(disabled_flags);
            } else {
                item.set_flags(enabled_flags);
            }
        }
    });
    trait_list_widget.item_selection_changed().connect(&update_count);

    // ── Queue table setup ─────────────────────────────────────────────────
    queue_table_widget.set_column_count(8);
    queue_table_widget.horizontal_header().set_stretch_last_section(true);
    queue_table_widget.horizontal_header()
        .resize_sections(qt_widgets::q_header_view::ResizeMode::ResizeToContents);

    // ── Shared pointers for slots ─────────────────────────────────────────
    let queue_ptr = queue_table_widget.as_ptr();
    let name_ptr = name_line_edit.as_ptr();
    let type_ptr = type_combo_box.as_ptr();
    let element_ptr = element_combo_box.as_ptr();
    let gender_ptr = gender_combo_box.as_ptr();
    let expanded_ptr = expanded_check_box.as_ptr();
    let status_ptr = status_label.as_ptr();
    let trait_list_ptr2 = trait_list_widget.as_ptr();

    // ── Add Character ─────────────────────────────────────────────────────
    let add_character = SlotNoArgs::new(&dialog, move || {
        let name = name_ptr.text().to_std_string();
        let name = name.trim();
        if name.is_empty() {
            status_ptr.set_text(&QString::from_std_str("ERR: Name cannot be empty."));
            return;
        }
        let selected = trait_list_ptr2.selected_items();
        if selected.count() != 3 {
            status_ptr.set_text(&QString::from_std_str("ERR: Select exactly 3 traits."));
            return;
        }
        let row = queue_ptr.row_count();
        queue_ptr.insert_row(row);

        let make_item = |text: &str| QTableWidgetItem::from_q_string(&QString::from_std_str(text)).into_ptr();

        queue_ptr.set_item(row, 0, make_item(name));
        queue_ptr.set_item(row, 1, make_item(&type_ptr.current_text().to_std_string()));
        queue_ptr.set_item(row, 2, make_item(&element_ptr.current_text().to_std_string()));
        queue_ptr.set_item(row, 3, make_item(&gender_ptr.current_text().to_std_string()));
        queue_ptr.set_item(row, 4, make_item(if expanded_ptr.is_checked() { "true" } else { "false" }));

        for i in 0..3i64 {
            let trait_item = &**selected.at(i);
            let display = trait_item.text().to_std_string();
            let data = trait_item.tool_tip().to_std_string();
            let cell = QTableWidgetItem::from_q_string(&QString::from_std_str(&display));
            cell.set_tool_tip(&QString::from_std_str(&data));
            queue_ptr.set_item(row, 5 + i as i32, cell.into_ptr());
        }

        name_ptr.clear();
        trait_list_ptr2.clear_selection();
        status_ptr.set_text(&QString::from_std_str(&format!("OK: {} character(s) in queue.", queue_ptr.row_count())));
    });
    add_character_button.released().connect(&add_character);

    // ── Clear All ─────────────────────────────────────────────────────────
    let clear_all = SlotNoArgs::new(&dialog, move || {
        queue_ptr.set_row_count(0);
        status_ptr.set_text(&QString::from_std_str("Queue cleared."));
    });
    clear_all_button.released().connect(&clear_all);

    // ── Delete Selected ───────────────────────────────────────────────────
    let delete_selected = SlotNoArgs::new(&dialog, move || {
        let selected = queue_ptr.selected_items();
        if selected.is_empty() { return; }
        let mut rows: Vec<i32> = (0..selected.count())
            .map(|i| queue_ptr.row(*selected.at(i)))
            .collect();
        rows.sort_unstable();
        rows.dedup();
        for r in rows.into_iter().rev() {
            queue_ptr.remove_row(r);
        }
        status_ptr.set_text(&QString::from_std_str(
            &format!("{} character(s) in queue.", queue_ptr.row_count())
        ));
    });
    delete_selected_button.released().connect(&delete_selected);

    // ── Run button ────────────────────────────────────────────────────────
    let run_button = button_box.add_button_q_string_button_role(
        &QString::from_std_str("Run"), ButtonRole::AcceptRole
    );
    let run_button_ptr = run_button.as_ptr();
    let dialog_ptr = dialog.as_ptr();
    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();
    let pack_key_closure = pack_key.clone();

    // Store paths returned by BuildCeoEntries so the exec() block can use them.
    let added_paths = std::rc::Rc::new(std::cell::RefCell::new(Vec::<ContainerPath>::new()));
    let added_paths_closure = added_paths.clone();

    let run_slot = SlotNoArgs::new(&dialog, move || {
        if queue_ptr.row_count() == 0 {
            status_ptr.set_text(&QString::from_std_str("ERR: Queue is empty."));
            return;
        }

        let mut entries: Vec<CeoEntryData> = Vec::new();
        for row in 0..queue_ptr.row_count() {
            let get = |col: i32| -> String { (*queue_ptr.item(row, col)).text().to_std_string() };
            let get_trait = |col: i32| -> (String, String) {
                let data = (*queue_ptr.item(row, col)).tool_tip().to_std_string();
                let mut parts = data.splitn(2, '|');
                let uuid = parts.next().unwrap_or("").to_string();
                let key  = parts.next().unwrap_or("").to_string();
                (uuid, key)
            };
            entries.push(CeoEntryData {
                name:     get(0),
                option:   get(1),
                element:  get(2),
                gender:   get(3),
                expanded: get(4) == "true",
                traits:   vec![get_trait(5), get_trait(6), get_trait(7)],
            });
        }

        run_button_ptr.set_enabled(false);
        status_ptr.set_text(&QString::from_std_str("Running..."));

        match send_ipc_command_result_async(
            Command::BuildCeoEntries(pack_key_closure.clone(), entries),
            response_extractor!(Response::VecContainerPath)
        ) {
            Ok(paths) => {
                status_ptr.set_text(&QString::from_std_str(
                    &format!("OK: {} file(s) updated.", paths.len())
                ));
                *added_paths_closure.borrow_mut() = paths;
                dialog_ptr.done(1);
            }
            Err(error) => {
                run_button_ptr.set_enabled(true);
                status_ptr.set_text(&QString::from_std_str(&format!("ERR: {}", error)));
            }
        }
    });
    run_button.released().connect(&run_slot);

    button_box.button(StandardButton::Cancel).released().connect(dialog.slot_reject());

    // ── Execute ───────────────────────────────────────────────────────────
    if dialog.exec() == 1 {
        let paths = added_paths.borrow().clone();
        if !paths.is_empty() {
            pack_file_contents_ui.packfile_contents_tree_view()
                .update_treeview(true, TreeViewOperation::Add(paths), DataSource::PackFile, &pack_key);
            UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
        }
    }
    Ok(())
}

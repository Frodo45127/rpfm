//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the code to run Lua test files from the open packs, and the dialog showing their results.

use qt_widgets::q_abstract_item_view::EditTrigger;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::QLabel;
use qt_widgets::QProgressBar;
use qt_widgets::QTreeView;
use qt_widgets::QVBoxLayout;
use qt_widgets::QWidget;

use qt_gui::QBrush;
use qt_gui::QColor;
use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::Orientation;
use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::WindowModality;

use cpp_core::{CastInto, CppBox, Ptr};

use anyhow::Result;

use std::rc::Rc;

use rpfm_extensions::lua::harness::{LuaTestReport, LuaTestResult};

use rpfm_ipc::helpers::DataSource;

use rpfm_lib::files::ContainerPath;

use crate::app_ui::AppUI;
use crate::communications::*;
use crate::pack_tree::{get_color_correct, get_color_wrong, PackTree};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::{qtr, qtre, show_dialog, tr, tre};

/// This function runs the Lua test files selected in the pack tree, and shows their results.
///
/// # Arguments
///
/// * `app_ui` - Main window of the program.
/// * `pack_file_contents_ui` - Pack tree the test files are selected in.
pub unsafe fn run_selected_lua_tests(app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
    let test_paths = <QPtr<QTreeView> as PackTree>::get_item_types_from_main_treeview_selection(pack_file_contents_ui)
        .into_iter()
        .filter_map(|path| match path {
            ContainerPath::File(path) if path.ends_with(".lua") => Some(path),
            _ => None,
        })
        .collect::<Vec<_>>();

    if test_paths.is_empty() {
        return;
    }

    // Tests run against the scripts in the backend, so edits in open views must be there first.
    if let Err(error) = PackFileContentsUI::save_open_files(app_ui, pack_file_contents_ui) {
        return show_dialog(app_ui.main_window(), error, false);
    }

    let pack_key = pack_file_contents_ui.pack_key_from_selection_or_first().unwrap_or_default();

    let (progress_dialog, progress_label) = progress_dialog(app_ui.main_window());
    let results = test_paths.into_iter()
        .map(|path| {
            progress_label.set_text(&qtre("lua_tests_running", &[&path]));
            let report = run_test_file(&pack_key, &path);
            (path, report)
        })
        .collect::<Vec<_>>();
    progress_dialog.hide();
    progress_dialog.delete_later();

    show_results(app_ui.main_window(), &results);
}

/// This function shows a dialog with a busy progress bar, blocking the rest of the program while tests run.
///
/// The bar keeps moving because waiting for the backend keeps processing UI events.
///
/// # Arguments
///
/// * `parent` - Window the dialog belongs to.
///
/// # Returns
///
/// The dialog, and the label to show what's running in.
unsafe fn progress_dialog(parent: impl CastInto<Ptr<QWidget>>) -> (QBox<QDialog>, QBox<QLabel>) {
    let dialog = QDialog::new_1a(parent);
    dialog.set_window_title(&qtr("lua_tests_running_title"));
    dialog.set_window_modality(WindowModality::ApplicationModal);
    dialog.resize_2a(450, 0);

    let layout = QVBoxLayout::new_1a(&dialog);
    let label = QLabel::new();
    let progress_bar = QProgressBar::new_1a(&dialog);

    // A range of 0 to 0 makes the bar show activity instead of progress, as runs don't report how far they are.
    progress_bar.set_range(0, 0);
    progress_bar.set_text_visible(false);

    layout.add_widget(&label);
    layout.add_widget(&progress_bar);
    dialog.show();

    (dialog, label)
}

/// This function runs the tests of a test file of a pack.
fn run_test_file(pack_key: &str, path: &str) -> Result<LuaTestReport> {
    let (text, _) = send_ipc_command_result_async(Command::DecodePackedFile(pack_key.to_owned(), path.to_owned(), DataSource::PackFile), response_extractor!(Response::TextRFileInfo, text, info))?;
    send_ipc_command_result_async(Command::LuaRunTests(text.contents().to_owned(), None), response_extractor!(Response::LuaTestReport))
}

/// This function shows the results of running test files.
unsafe fn show_results(parent: impl CastInto<Ptr<QWidget>>, results: &[(String, Result<LuaTestReport>)]) {
    let dialog = QDialog::new_1a(parent);
    dialog.set_window_title(&qtr("lua_tests_title"));
    dialog.set_modal(true);
    dialog.resize_2a(1000, 650);

    let layout = QVBoxLayout::new_1a(&dialog);

    let tests = results.iter().filter_map(|(_, report)| report.as_ref().ok()).flat_map(|report| report.tests()).collect::<Vec<_>>();
    let failed = tests.iter().filter(|test| !test.passed()).count();
    let summary = QLabel::from_q_string(&qtre("lua_tests_summary", &[&(tests.len() - failed).to_string(), &failed.to_string()]));
    layout.add_widget(&summary);

    let model = QStandardItemModel::new_1a(&dialog);
    model.set_column_count(2);
    model.set_header_data_3a(0, Orientation::Horizontal, &QVariant::from_q_string(&qtr("lua_tests_column_test")));
    model.set_header_data_3a(1, Orientation::Horizontal, &QVariant::from_q_string(&qtr("lua_tests_column_result")));

    let tree_view = QTreeView::new_1a(&dialog);
    tree_view.set_model(&model);
    tree_view.set_edit_triggers(EditTrigger::NoEditTriggers.into());
    tree_view.set_uniform_row_heights(false);
    tree_view.header().resize_section(0, 600);
    layout.add_widget(&tree_view);

    for (path, report) in results {
        let file_item = QStandardItem::from_q_string(&QString::from_std_str(path));
        let file_result = match report {
            Ok(report) => {
                for test in report.tests() {
                    add_test_row(&file_item, test);
                }

                add_lines(&file_item, &tr("lua_tests_boot_errors"), report.boot_errors());

                let failed = report.tests().iter().filter(|test| !test.passed()).count();
                result_item(tre("lua_tests_file_summary", &[&(report.tests().len() - failed).to_string(), &failed.to_string()]), failed == 0)
            },
            Err(error) => result_item(error.to_string(), false),
        };

        let row = QListOfQStandardItem::new_0a();
        row.append_q_standard_item(&file_item.into_ptr().as_mut_raw_ptr());
        row.append_q_standard_item(&file_result.into_ptr().as_mut_raw_ptr());
        model.append_row_q_list_of_q_standard_item(&row);

        let index = model.index_2a(model.row_count_0a() - 1, 0);
        tree_view.expand(&index);

        // Failed tests are expanded, so their errors are visible without searching for them. Tests are the first children.
        if let Ok(report) = report {
            for (test_row, test) in report.tests().iter().enumerate() {
                if !test.passed() {
                    tree_view.expand(&model.index_3a(test_row as i32, 0, &index));
                }
            }
        }
    }

    let button_box = QDialogButtonBox::new();
    let close_button = button_box.add_button_standard_button(StandardButton::Close);
    close_button.released().connect(dialog.slot_accept());
    layout.add_widget(&button_box);

    dialog.exec();
}

/// This function adds a test, with its errors, undocumented calls and output, as a child of a file.
unsafe fn add_test_row(file_item: &CppBox<QStandardItem>, test: &LuaTestResult) {
    let test_item = QStandardItem::from_q_string(&QString::from_std_str(test.name()));
    let result = if *test.passed() { tr("lua_tests_passed") } else { tr("lua_tests_failed") };

    add_lines(&test_item, &tr("lua_tests_errors"), test.errors());
    add_lines(&test_item, &tr("lua_tests_unmocked_calls"), test.unmocked_calls());
    add_lines(&test_item, &tr("lua_tests_output"), test.log());

    let row = QListOfQStandardItem::new_0a();
    row.append_q_standard_item(&test_item.into_ptr().as_mut_raw_ptr());
    row.append_q_standard_item(&result_item(result, *test.passed()).into_ptr().as_mut_raw_ptr());
    file_item.append_row_q_list_of_q_standard_item(&row);
}

/// This function adds a group of lines as a child of an item, if there are any.
///
/// Only the first line of each entry is shown, with the full text, like error tracebacks, as its tooltip.
unsafe fn add_lines(parent: &CppBox<QStandardItem>, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }

    let group = QStandardItem::from_q_string(&QString::from_std_str(format!("{title} ({})", lines.len())));
    for line in lines {
        let item = QStandardItem::from_q_string(&QString::from_std_str(line.lines().next().unwrap_or_default()));
        item.set_tool_tip(&QString::from_std_str(line));
        group.append_row_q_standard_item(item.into_ptr());
    }

    parent.append_row_q_standard_item(group.into_ptr());
}

/// This function creates the item showing a result, colored by whether it's a success.
unsafe fn result_item(text: String, success: bool) -> CppBox<QStandardItem> {
    let item = QStandardItem::from_q_string(&QString::from_std_str(text));
    let color = if success { get_color_correct() } else { get_color_wrong() };
    item.set_foreground(&QBrush::from_q_color(&QColor::from_q_string(&QString::from_std_str(color))));
    item
}

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the utility functions, to make our programming lives easier.
!*/

use qt_widgets::QAction;
use qt_widgets::QApplication;
use qt_widgets::QDialog;
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::QMenu;
use qt_widgets::{QMessageBox, q_message_box::{Icon, StandardButton}};
use qt_widgets::QPushButton;
use qt_widgets::QWidget;
use qt_widgets::QMainWindow;

use qt_ui_tools::QUiLoader;

use qt_core::QBox;
use qt_core::QCoreApplication;
use qt_core::QFlags;
use qt_core::QListOfQObject;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QObject;
use qt_core::SlotNoArgs;
use qt_core::WidgetAttribute;

use cpp_core::CastInto;
use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::DynamicCast;
use cpp_core::Ptr;
use cpp_core::Ref;
use cpp_core::StaticUpcast;

use anyhow::{anyhow, Result};
use regex::Regex;

use rpfm_lib::files::pack::PackSettings;
use rpfm_lib::integrations::log::*;

use std::convert::AsRef;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{ASSETS_PATH, DARK_PALETTE, GAME_SELECTED, LIGHT_PALETTE, LIGHT_STYLE_SHEET, SENTRY_GUARD};
use crate::ffi::*;
use crate::locale::{qtr, qtre};
use crate::setting_bool;
use crate::STATUS_BAR;
use crate::pack_tree::{get_color_correct, get_color_wrong, get_color_clean};

// Colors used all over the program for theming and stuff.
pub const MEDIUM_DARKER_GREY: &str = "#262626";          // Medium-Darker Grey.
pub const GREEN_BRIGHT: &str = "#D0FDCC";
pub const GREEN_DARK: &str = "#708F6E";
pub const RED_BRIGHT: &str = "#FFCCCC";
pub const RED_DARK: &str = "#8F6E6E";
pub const TRANSPARENT_BRIGHT: &str = "#00000000";
pub const ERROR_UNPRESSED_DARK: &str = "#b30000";
pub const ERROR_UNPRESSED_LIGHT: &str = "#ffcccc";
pub const ERROR_PRESSED_DARK: &str = "#e60000";
pub const ERROR_PRESSED_LIGHT: &str = "#ff9999";
pub const WARNING_UNPRESSED_DARK: &str = "#4d4d00";
pub const WARNING_UNPRESSED_LIGHT: &str = "#ffffcc";
pub const WARNING_PRESSED_DARK: &str = "#808000";
pub const WARNING_PRESSED_LIGHT: &str = "#ffff99";
pub const INFO_UNPRESSED_DARK: &str = "#0059b3";
pub const INFO_UNPRESSED_LIGHT: &str = "#cce6ff";
pub const INFO_PRESSED_DARK: &str = "#0073e6";
pub const INFO_PRESSED_LIGHT: &str = "#99ccff";

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

pub(crate) fn atomic_from_cpp_box<T: CppDeletable>(cpp_box: CppBox<T>) -> AtomicPtr<T> {
    AtomicPtr::new(cpp_box.into_raw_ptr())
}

pub(crate) fn atomic_from_q_box<T: StaticUpcast<QObject> + CppDeletable>(q_box: QBox<T>) -> AtomicPtr<T> {
    unsafe { AtomicPtr::new(q_box.as_mut_raw_ptr()) }
}

pub(crate) fn atomic_from_ptr<T: Sized>(ptr: Ptr<T>) -> AtomicPtr<T> {
    AtomicPtr::new(ptr.as_mut_raw_ptr())
}

pub(crate) fn q_ptr_from_atomic<T: Sized + StaticUpcast<QObject>>(ptr: &AtomicPtr<T>) -> QPtr<T> {
    unsafe { QPtr::from_raw(ptr.load(Ordering::SeqCst)) }
}

pub(crate) fn ptr_from_atomic<T: Sized>(ptr: &AtomicPtr<T>) -> Ptr<T> {
    unsafe { Ptr::from_raw(ptr.load(Ordering::SeqCst)) }
}

pub(crate) fn ref_from_atomic<T: Sized>(ptr: &AtomicPtr<T>) -> Ref<T> {
    unsafe { Ref::from_raw(ptr.load(Ordering::SeqCst)).unwrap() }
}

pub(crate) fn ref_from_atomic_ref<T: Sized>(ptr: &AtomicPtr<T>) -> Ref<T> {
    unsafe { Ref::from_raw(ptr.load(Ordering::SeqCst)).unwrap() }
}

/// This functions logs the provided message to the status bar, so it can be seen by the user.
pub(crate) fn log_to_status_bar(text: &str) {
    unsafe { q_ptr_from_atomic(&STATUS_BAR).show_message_2a(&QString::from_std_str(text), 2500); }
    info!("{}", text);
}

/// This function takes the received KMessageWidget, and pushes a message onto it, making it visible in the process as an Error.
///
/// It requires:
/// - widget: a pointer to the KMessageWidget.
/// - text: something that implements the trait `Display`, to put in the KMessageWidget.
#[allow(dead_code)]
pub unsafe fn show_message_error<T: Display>(widget: &QPtr<QWidget>, text: T) {
    let message = QString::from_std_str(text.to_string());
    kmessage_widget_set_error_safe(&widget.as_ptr(), message.into_ptr())
}

/// This function takes the received KMessageWidget, and pushes a message onto it, making it visible in the process as a Warning.
///
/// It requires:
/// - widget: a pointer to the KMessageWidget.
/// - text: something that implements the trait `Display`, to put in the KMessageWidget.
#[allow(dead_code)]
pub unsafe fn show_message_warning<T: Display>(widget: &QPtr<QWidget>, text: T) {
    let message = QString::from_std_str(text.to_string());
    kmessage_widget_set_warning_safe(&widget.as_ptr(), message.into_ptr())
}

/// This function takes the received KMessageWidget, and pushes a message onto it, making it visible in the process as an Info Message.
///
/// It requires:
/// - widget: a pointer to the KMessageWidget.
/// - text: something that implements the trait `Display`, to put in the KMessageWidget.
#[allow(dead_code)]
pub unsafe fn show_message_info<T: Display>(widget: &QPtr<QWidget>, text: T) {
    let message = QString::from_std_str(text.to_string());
    kmessage_widget_set_info_safe(&widget.as_ptr(), message.into_ptr())
}

/// This function creates a modal dialog, for showing successes or errors.
///
/// It requires:
/// - parent: a pointer to the widget that'll be the parent of the dialog.
/// - text: something that implements the trait `Display`, to put in the dialog window.
/// - is_success: true for `Success` Dialog, false for `Error` Dialog.
pub unsafe fn show_dialog<T: Display>(parent: impl cpp_core::CastInto<Ptr<QWidget>>, text: T, is_success: bool) {

    // Depending on the type of the dialog, set everything specific here.
    let title = if is_success { qtr("title_success") } else { qtr("title_error") };
    let icon = if is_success { Icon::Information } else { Icon::Critical };

    // Create and run the dialog.
    let message_box = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
        icon,
        &title,
        &QString::from_std_str(text.to_string()),
        QFlags::from(StandardButton::Ok),
        parent,
    );

    message_box.set_attribute_1a(WidgetAttribute::WADeleteOnClose);
    message_box.exec();
}

/// This function creates a non-modal dialog, for debugging purpouses.
///
/// It requires:
/// - text: something that dereferences to `str`, to put in the window.
pub unsafe fn show_debug_dialog<T: AsRef<str>>(parent: impl cpp_core::CastInto<Ptr<QWidget>>, text: T) {
    let window = QMainWindow::new_1a(parent);
    let widget = QWidget::new_1a(window.static_upcast::<QMainWindow>());
    window.set_central_widget(&widget);
    let layout = create_grid_layout(widget.static_upcast());
    let editor = new_text_editor_safe(&widget.static_upcast());

    layout.add_widget_5a(&editor, 0, 0, 1, 1);
    set_text_safe(&editor.static_upcast(), &QString::from_std_str(text).as_ptr(), &QString::from_std_str("plain").as_ptr());

    // Center it on screen.
    window.resize_2a(1000, 600);
    let pos_x = QApplication::desktop().screen_geometry().center().as_ref() - window.rect().center().as_ref();
    window.move_1a(&pos_x);

    // And show it.
    let window_ptr = window.into_ptr();
    window_ptr.show();
}

/// This function creates a modal dialog with an extra "Send for decoding" button, for showing
/// errors and sending tables to be decoded.
///
/// It requires:
/// - parent: a pointer to the widget that'll be the parent of the dialog.
/// - text: something that implements the trait `Display`, to put in the dialog window.
/// - table_name: name/type of the table to decode.
/// - table_data: data of the table to decode.
pub unsafe fn show_dialog_decode_button<T: Display>(parent: Ptr<QWidget>, text: T, table_name: &str, table_data: &[u8]) {
    let table_name = table_name.to_owned();
    let table_data = table_data.to_owned();

    // Create and run the dialog.
    let dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
        Icon::Critical,
        &qtr("title_error"),
        &QString::from_std_str(text.to_string()),
        QFlags::from(0),
        parent,
    );

    let send_table_button = dialog.add_button_q_string_button_role(&qtr("send_table_for_decoding"), qt_widgets::q_message_box::ButtonRole::AcceptRole);
    dialog.add_button_standard_button(StandardButton::Ok);

    let send_table_slot = SlotNoArgs::new(&dialog, move || {
        show_undecoded_table_report_dialog(parent, &table_name, &table_data);
    });
    send_table_button.released().connect(&send_table_slot);

    // Disable sending tables until I implement a more robust way to stop the spam.
    send_table_button.set_enabled(false);

    dialog.exec();
}

/// This function creates a modal dialog, for sending tables to be decoded.
///
/// It requires:
/// - parent: a pointer to the widget that'll be the parent of the dialog.
/// - table_name: the name of the table to decode.
/// - table_data: data of the table to decode.
pub unsafe fn show_undecoded_table_report_dialog(parent: Ptr<QWidget>, table_name: &str, table_data: &[u8]) {
    let table_name = table_name.to_owned();
    let table_data = table_data.to_owned();

    // Create and configure the dialog.
    let dialog = QDialog::new_1a(parent);
    dialog.set_window_title(&qtr("send_table_for_decoding"));
    dialog.set_modal(true);
    dialog.resize_2a(400, 50);

    let main_grid = create_grid_layout(dialog.static_upcast());
    let explanation_label = QLabel::from_q_string_q_widget(&qtre("send_table_for_decoding_explanation", &[(GAME_SELECTED.read().unwrap().game_key_name()), &table_name]), &dialog);
    let cancel_button = QPushButton::from_q_string(&qtr("cancel"));
    let accept_button = QPushButton::from_q_string(&qtr("send"));

    main_grid.add_widget_5a(&explanation_label, 0, 0, 1, 2);
    main_grid.add_widget_5a(&cancel_button, 6, 0, 1, 1);
    main_grid.add_widget_5a(&accept_button, 6, 1, 1, 1);

    let send_table_slot = SlotNoArgs::new(&dialog, move || {
        let message = format!("{} - Request for table decoding: {}", GAME_SELECTED.read().unwrap().display_name(), table_name);
        if let Err(error) = Logger::send_event(&SENTRY_GUARD.read().unwrap(), Level::Info, &message, Some((&table_name, &table_data))) {
            show_dialog(parent, error, false)
        }
    });

    accept_button.released().connect(&send_table_slot);
    accept_button.released().connect(dialog.slot_accept());
    cancel_button.released().connect(dialog.slot_close());
    dialog.exec();
}

pub unsafe fn add_action_to_menu(menu: &QPtr<QMenu>, shortcuts: Ref<QListOfQObject>, action_group: &str, action_name: &str, action_translation_key: &str, associated_widget: Option<QPtr<QWidget>>) -> QPtr<QAction> {
    let action = shortcut_action_safe(shortcuts.as_ptr(), QString::from_std_str(action_group).into_ptr(), QString::from_std_str(action_name).into_ptr());
    action.set_text(&qtr(action_translation_key));
    menu.add_action(action.as_ptr());

    if let Some(associated_widget) = associated_widget {
        associated_widget.add_action(action.as_ptr());
    }

    action
}

pub unsafe fn add_action_to_widget(shortcuts: Ref<QListOfQObject>, action_group: &str, action_name: &str, associated_widget: Option<QPtr<QWidget>>) -> QPtr<QAction> {
    let action = shortcut_action_safe(shortcuts.as_ptr(), QString::from_std_str(action_group).into_ptr(), QString::from_std_str(action_name).into_ptr());

    if let Some(associated_widget) = associated_widget {
        associated_widget.add_action(action.as_ptr());
    }

    action
}

/// This function deletes all widgets from a widget's layout.
#[cfg(feature = "enable_tools")] pub unsafe fn clear_layout(widget: &QPtr<QWidget>) {
    let layout = widget.layout();
    while !layout.is_empty() {
        let item = layout.take_at(0);
        item.widget().delete();
        item.delete();
    }
}

/// This function creates a `GridLayout` for the provided widget with the settings we want.
pub unsafe fn create_grid_layout(widget: QPtr<QWidget>) -> QBox<QGridLayout> {
    let widget_layout = QGridLayout::new_1a(&widget);
    widget.set_layout(&widget_layout);

    // Due to how Qt works, if we want a decent look on windows, we have to do some specific tweaks there.
    if cfg!(target_os = "windows") {
        widget_layout.set_contents_margins_4a(2, 2, 2, 2);
        widget_layout.set_spacing(1);
    }
    else {
        widget_layout.set_contents_margins_4a(0, 0, 0, 0);
        widget_layout.set_spacing(0);
    }

    widget_layout
}

pub unsafe fn check_regex(pattern: &str, widget: QPtr<QWidget>) {
    let style_sheet = if !pattern.is_empty() {
        if Regex::new(pattern).is_ok() {
            get_color_correct()
        } else {
            get_color_wrong()
        }
    }
    else {
        get_color_clean()
    };

    widget.set_style_sheet(&QString::from_std_str(format!("background-color: {style_sheet}")));
}

/// This functin returns the feature flags enabled for RPFM.
pub fn get_feature_flags() -> String {
    let mut feature_flags = String::new();

    #[cfg(feature = "strict_subclasses_compilation")] {
        feature_flags.push_str("strict_subclasses_compilation");
    }

    #[cfg(feature = "support_rigidmodel")] {
        if !feature_flags.is_empty() {
            feature_flags.push_str(", ");
        }
        feature_flags.push_str("support_rigidmodel");
    }

    #[cfg(feature = "support_modern_dds")] {
        if !feature_flags.is_empty() {
            feature_flags.push_str(", ");
        }
        feature_flags.push_str("support_modern_dds");
    }

    #[cfg(feature = "support_uic")] {
        if !feature_flags.is_empty() {
            feature_flags.push_str(", ");
        }
        feature_flags.push_str("support_uic");
    }

    #[cfg(feature = "enable_tools")] {
        if !feature_flags.is_empty() {
            feature_flags.push_str(", ");
        }
        feature_flags.push_str("enable_tools");
    }

    #[cfg(feature = "only_for_the_brave")] {
        if !feature_flags.is_empty() {
            feature_flags.push_str(", ");
        }
        feature_flags.push_str("only_for_the_brave");
    }

    if feature_flags.is_empty() {
        feature_flags.push_str("None");
    }

    feature_flags
}

/// This function creates the stylesheet used for the dark theme in windows.
pub fn dark_stylesheet() -> Result<String> {
    let mut file = File::open(ASSETS_PATH.join("dark-theme.qss"))?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string.replace("{assets_path}", &ASSETS_PATH.to_string_lossy()))
}

/// This function is used to load/reload a theme live.
pub unsafe fn reload_theme() {
    let app = QCoreApplication::instance();
    let qapp = app.static_downcast::<QApplication>();
    let use_dark_theme = setting_bool("use_dark_theme");

    // Initialize the globals before applying anything.
    let light_style_sheet = ref_from_atomic(&*LIGHT_STYLE_SHEET);
    let light_palette = ref_from_atomic(&*LIGHT_PALETTE);
    let dark_palette = ref_from_atomic(&*DARK_PALETTE);

    // On Windows, we use the dark theme switch to control the Style, StyleSheet and Palette.
    if cfg!(target_os = "windows") {
        if use_dark_theme {
            QApplication::set_style_q_string(&QString::from_std_str("fusion"));
            QApplication::set_palette_1a(dark_palette);
            if let Ok(dark_stylesheet) = dark_stylesheet() {
                qapp.set_style_sheet(&QString::from_std_str(dark_stylesheet));
            }
        } else {
            QApplication::set_style_q_string(&QString::from_std_str("windowsvista"));
            QApplication::set_palette_1a(light_palette);
            qapp.set_style_sheet(light_style_sheet);
        }
    }

    // On MacOS, we use the dark theme switch to control the StyleSheet and Palette.
    else if cfg!(target_os = "macos") {
        if use_dark_theme {
            QApplication::set_palette_1a(dark_palette);
            if let Ok(dark_stylesheet) = dark_stylesheet() {
                qapp.set_style_sheet(&QString::from_std_str(dark_stylesheet));
            }
        } else {
            QApplication::set_palette_1a(light_palette);
            qapp.set_style_sheet(light_style_sheet);
        }
    }

    // Linux and company.
    else if use_dark_theme {
        qt_widgets::QApplication::set_palette_1a(dark_palette);
        if let Ok(dark_stylesheet) = dark_stylesheet() {
            qapp.set_style_sheet(&QString::from_std_str(dark_stylesheet));
        }
    } else {
        qt_widgets::QApplication::set_palette_1a(light_palette);
        qapp.set_style_sheet(light_style_sheet);
    }
}

/// This function returns the a widget from the view if it exits, and an error if it doesn't.
pub unsafe fn find_widget<T: StaticUpcast<qt_core::QObject>>(main_widget: &QPtr<QWidget>, widget_name: &str) -> Result<QPtr<T>>
    where QObject: DynamicCast<T> {
    main_widget.find_child(widget_name)
        .map_err(|_|
            anyhow!("One of the widgets of this view has not been found in the UI Template. This means either the code is wrong, or the template is incomplete/outdated.

            The missing widgets are: {}", widget_name))
}

/// This function load the template file in the provided path to memory, and returns it as a QBox<QWidget>.
pub unsafe fn load_template(parent: impl CastInto<Ptr<QWidget>>, path: &str) -> Result<QBox<QWidget>> {
    let path = format!("{}/{}", ASSETS_PATH.to_string_lossy(), path);
    let mut data = vec!();
    let mut file = BufReader::new(File::open(path)?);
    file.read_to_end(&mut data)?;

    let ui_loader = QUiLoader::new_0a();
    let main_widget = ui_loader.load_bytes_with_parent(&data, parent);

    Ok(main_widget)
}

pub fn initialize_pack_settings() -> PackSettings {
    let mut pack_settings = PackSettings::default();
    pack_settings.settings_text_mut().insert("diagnostics_files_to_ignore".to_owned(), "".to_owned());
    pack_settings.settings_text_mut().insert("import_files_to_ignore".to_owned(), "".to_owned());
    pack_settings.settings_bool_mut().insert("disable_autosaves".to_owned(), false);
    pack_settings
}

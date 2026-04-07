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
Module with all the utility functions, to make our programming lives easier.
!*/

use qt_widgets::QApplication;
use qt_widgets::QMenu;
use qt_widgets::{QMessageBox, q_message_box::{Icon, StandardButton}};
use qt_widgets::QMainWindow;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QAction;
use qt_gui::QGuiApplication;
use qt_gui::QIcon;
use qt_gui::q_palette::ColorRole;

use qt_core::QCoreApplication;
use qt_core::QFlags;
use qt_core::QListOfQObject;
use qt_core::QPtr;
use qt_core::QString;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;

use regex::Regex;

use std::convert::AsRef;
use std::fmt::Display;

use rpfm_log::*;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::utils::*;

use crate::LOCALE;
use crate::LOCALE_FALLBACK;
use crate::app_ui::AppUI;
use crate::ffi::*;
use crate::STATUS_BAR;
use crate::pack_tree::*;

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

/// This functions logs the provided message to the status bar, so it can be seen by the user.
pub(crate) fn log_to_status_bar(text: &str) {
    unsafe { q_ptr_from_atomic(&STATUS_BAR).show_message_2a(&QString::from_std_str(text), 2500); }
    info!("{text}");
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

pub unsafe fn show_dialog<T: Display>(parent: impl cpp_core::CastInto<Ptr<QWidget>>, text: T, is_success: bool) {
    let title = if is_success { tr("title_success")} else { tr("title_error") };
    rpfm_ui_common::utils::show_dialog(parent, title, text, is_success);
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
    let pos_x = QGuiApplication::primary_screen().geometry().center();
    pos_x.sub_assign(window.rect().center().as_ref());
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
pub unsafe fn show_dialog_decode_button<T: Display>(parent: Ptr<QWidget>, text: T) {
    // let table_name = table_name.to_owned();
    // let table_data = table_data.to_owned();

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

    // let send_table_slot = SlotNoArgs::new(&dialog, move || {
    //     show_undecoded_table_report_dialog(parent, &table_name, &table_data);
    // });
    // send_table_button.released().connect(&send_table_slot);

    // Disable sending tables until I implement a more robust way to stop the spam.
    send_table_button.set_enabled(false);

    dialog.exec();
}
/*
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
    let explanation_label = QLabel::from_q_string_q_widget(&qtre("send_table_for_decoding_explanation", &[(GAME_SELECTED.read().unwrap().key()), &table_name]), &dialog);
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
}*/

pub unsafe fn add_action_to_menu(menu: &QPtr<QMenu>, shortcuts: Ref<QListOfQObject>, action_group: &str, action_name: &str, action_translation_key: &str, associated_widget: Option<QPtr<QWidget>>) -> QPtr<QAction> {
    let action = shortcut_action_safe(shortcuts.as_ptr(), QString::from_std_str(action_group).into_ptr(), QString::from_std_str(action_name).into_ptr());
    action.set_text(&qtr(action_translation_key));
    menu.add_action_q_action(action.as_ptr());

    if let Some(associated_widget) = associated_widget {
        associated_widget.add_action_q_action(action.as_ptr());
    }

    action
}

pub unsafe fn add_action_to_widget(shortcuts: Ref<QListOfQObject>, action_group: &str, action_name: &str, associated_widget: Option<QPtr<QWidget>>) -> QPtr<QAction> {
    let action = shortcut_action_safe(shortcuts.as_ptr(), QString::from_std_str(action_group).into_ptr(), QString::from_std_str(action_name).into_ptr());

    if let Some(associated_widget) = associated_widget {
        associated_widget.add_action_q_action(action.as_ptr());
    }

    action
}

pub unsafe fn check_regex(pattern: &str, widget: QPtr<QWidget>, use_regex: bool) {
    let style_sheet = if !pattern.is_empty() && use_regex {
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

/// Detects whether the current system theme is dark by checking the application palette.
///
/// Returns `true` if the system is using a dark color scheme (Window background lightness < 128).
/// This replaces the old `use_dark_theme` setting — the system now controls light/dark mode natively.
pub unsafe fn is_dark_theme() -> bool {
    let palette = QGuiApplication::palette();
    let window_color = palette.color_1a(ColorRole::Window);
    window_color.lightness() < 128
}

/// This function refreshes theme-dependent UI elements to match the current native theme.
///
/// It does NOT override the native palette or stylesheet — Qt/KDE handles those.
/// It only updates elements that need manual intervention: icons with light/dark variants,
/// diagnostic filter button colors, and forces a full repaint.
pub unsafe fn reload_theme(app_ui: &AppUI) {
    let app = QCoreApplication::instance();
    let qapp = app.static_downcast::<QApplication>();

    // Clear any leftover custom stylesheet so native theming takes full effect.
    qapp.set_style_sheet(&QString::from_std_str(""));

    // Re-apply the current native palette to force all widgets to refresh.
    let native_palette = QGuiApplication::palette();
    QApplication::set_palette_1a(&native_palette);

    // Select the appropriate GitHub icon based on the native theme.
    if is_dark_theme() {
        app_ui.github_button().set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy()))));
    } else {
        app_ui.github_button().set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github-dark.svg", ASSETS_PATH.to_string_lossy()))));
    }

    // Re-apply diagnostic filter button colors for the current theme.
    reload_diagnostic_button_styles(app_ui);

    // Force the menu bar to fully recalculate its style. The KDE style caches palette
    // colors internally, so just calling update()/repaint() is not enough — we need to
    // unpolish (clear cache) then polish (recompute) via the style engine.
    let menu_bar = app_ui.main_window().menu_bar();
    let style = menu_bar.style();
    style.unpolish_q_widget(menu_bar.static_upcast::<QWidget>());
    style.polish_q_widget(menu_bar.static_upcast::<QWidget>());
    menu_bar.update();

    // Force the main window and all children to repaint.
    app_ui.main_window().repaint();
}

/// Re-applies theme-aware stylesheets to the diagnostic filter buttons.
///
/// These buttons use per-widget stylesheets for their colored backgrounds, so they
/// need to be re-applied whenever the theme changes.
unsafe fn reload_diagnostic_button_styles(app_ui: &AppUI) {
    let button_style = |unpressed: &str, pressed: &str| -> String {
        format!(
            "QToolButton {{ background-color: {} }} QToolButton:checked {{ background-color: {} }}",
            unpressed, pressed
        )
    };

    // Find the diagnostic buttons by object name. They may not exist if the dock was never created.
    let main_widget = app_ui.main_window().static_upcast::<QWidget>();
    if let Ok(info_btn) = find_widget::<QToolButton>(&main_widget, "info_button") {
        info_btn.set_style_sheet(&QString::from_std_str(button_style(&get_color_info(), &get_color_info_pressed())));
    }
    if let Ok(warn_btn) = find_widget::<QToolButton>(&main_widget, "warning_button") {
        warn_btn.set_style_sheet(&QString::from_std_str(button_style(&get_color_warning(), &get_color_warning_pressed())));
    }
    if let Ok(err_btn) = find_widget::<QToolButton>(&main_widget, "error_button") {
        err_btn.set_style_sheet(&QString::from_std_str(button_style(&get_color_error(), &get_color_error_pressed())));
    }
}

/// This function returns the translation for the key provided in the current language.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn tr(key: &str) -> String {
    LOCALE.tr(&LOCALE_FALLBACK, key)
}

/// This function returns the translation for the key provided in the current language,
/// replacing certain parts of the translation with the replacements provided.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn tre(key: &str, replacements: &[&str]) -> String {
    LOCALE.tre(&LOCALE_FALLBACK, key, replacements)
}

/// This function returns the translation as a `QString` for the key provided in the current language.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn qtr(key: &str) -> CppBox<QString> {
    QString::from_std_str(tr(key))
}

/// This function returns the translation as a `QString` for the key provided in the current language,
/// replacing certain parts of the translation with the replacements provided.
///
/// If the key doesn't exists, it returns the equivalent from the english localisation. If it fails to find it there too, returns a warning.
pub fn qtre(key: &str, replacements: &[&str]) -> CppBox<QString> {
    QString::from_std_str(tre(key, replacements))
}

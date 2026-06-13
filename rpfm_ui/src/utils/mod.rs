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
use qt_widgets::QDialog;
use qt_widgets::QLabel;
use qt_widgets::QMenu;
use qt_widgets::{QMessageBox, q_message_box::{Icon, StandardButton}};
use qt_widgets::QMainWindow;
use qt_widgets::QPushButton;
use qt_widgets::QTextEdit;
use qt_widgets::QToolButton;
use qt_widgets::QWidget;

use qt_gui::QAction;
use qt_gui::QGuiApplication;
use qt_gui::QIcon;
use qt_gui::q_palette::ColorRole;
#[cfg(target_os = "windows")] use qt_gui::QColor;
#[cfg(target_os = "windows")] use qt_gui::QPalette;
#[cfg(target_os = "windows")] use qt_gui::q_palette::ColorGroup;

#[cfg(target_os = "windows")] use qt_core::ColorScheme;
#[cfg(target_os = "windows")] use qt_core::QCoreApplication;
use qt_core::QFlags;
use qt_core::QListOfQObject;
use qt_core::QPtr;
use qt_core::SlotNoArgs;
use qt_core::QString;

use cpp_core::CppBox;
use cpp_core::Ptr;
use cpp_core::Ref;

use regex::Regex;

use std::cell::Cell;
use std::convert::AsRef;
use std::fmt::Display;

use rpfm_telemetry::*;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
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
pub const GREEN_DARK: &str = "#264d26";
pub const RED_BRIGHT: &str = "#FFCCCC";
pub const RED_DARK: &str = "#4d2626";
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

/// Modal dialog that lets the user type a free-form message and ship it to
/// PostHog via [`rpfm_telemetry::send_user_feedback`].
///
/// # Arguments
///
/// * `parent` - Widget that owns the dialog; the dialog is centered on it.
pub unsafe fn show_feedback_dialog(parent: impl cpp_core::CastInto<Ptr<QWidget>>) {
    let parent = parent.cast_into();
    let icon = QIcon::from_theme_q_string(&QString::from_std_str("mail-send"));

    let dialog = QDialog::new_1a(parent);
    dialog.set_window_title(&qtr("feedback_dialog_title"));
    dialog.set_window_icon(&icon);
    dialog.set_modal(true);
    dialog.resize_2a(560, 380);

    let main_grid = create_grid_layout(dialog.static_upcast());
    main_grid.set_contents_margins_4a(16, 16, 16, 16);
    main_grid.set_spacing(12);

    // Header row: icon + bold title.
    let header_widget = QWidget::new_1a(&dialog);
    let header_layout = create_grid_layout(header_widget.static_upcast());
    header_layout.set_spacing(10);

    let header_icon_label = QLabel::from_q_widget(&header_widget);
    header_icon_label.set_pixmap(&icon.pixmap_2_int(32, 32));
    header_layout.add_widget_5a(&header_icon_label, 0, 0, 1, 1);

    let title_label = QLabel::from_q_string_q_widget(
        &QString::from_std_str(format!("<h3 style='margin:0;'>{}</h3>", tr("feedback_dialog_title"))),
        &header_widget,
    );
    header_layout.add_widget_5a(&title_label, 0, 1, 1, 1);
    header_layout.set_column_stretch(1, 1);

    main_grid.add_widget_5a(&header_widget, 0, 0, 1, 1);

    // Explanation paragraph.
    let explanation_label = QLabel::from_q_string_q_widget(&qtr("feedback_dialog_explanation"), &dialog);
    explanation_label.set_word_wrap(true);
    main_grid.add_widget_5a(&explanation_label, 1, 0, 1, 1);

    // Multiline text input.
    let text_edit = QTextEdit::from_q_widget(&dialog);
    text_edit.set_placeholder_text(&qtr("feedback_dialog_placeholder"));
    text_edit.set_minimum_height(140);
    main_grid.add_widget_5a(&text_edit, 2, 0, 1, 1);
    main_grid.set_row_stretch(2, 1);

    // Right-aligned button row. Send is the default action (Enter triggers it).
    let buttons_widget = QWidget::new_1a(&dialog);
    let buttons_layout = create_grid_layout(buttons_widget.static_upcast());
    buttons_layout.set_spacing(8);
    buttons_layout.set_column_stretch(0, 1);

    let cancel_button = QPushButton::from_q_string_q_widget(&qtr("cancel"), &buttons_widget);
    let send_button = QPushButton::from_q_string_q_widget(&qtr("send"), &buttons_widget);
    send_button.set_icon(&icon);
    send_button.set_default(true);

    buttons_layout.add_widget_5a(&cancel_button, 0, 1, 1, 1);
    buttons_layout.add_widget_5a(&send_button, 0, 2, 1, 1);

    main_grid.add_widget_5a(&buttons_widget, 3, 0, 1, 1);

    let dialog_ptr = dialog.static_upcast::<QDialog>();
    let text_edit_ptr = text_edit.static_upcast::<QTextEdit>();

    let send_slot = SlotNoArgs::new(&dialog, clone!(
        text_edit_ptr,
        dialog_ptr => move || {
            let text = text_edit_ptr.to_plain_text().to_std_string();
            if text.trim().is_empty() {
                show_dialog(parent, tr("feedback_empty"), false);
                return;
            }

            send_user_feedback(&text);
            show_dialog(parent, tr("feedback_sent"), true);
            dialog_ptr.accept();
        }
    ));

    send_button.released().connect(&send_slot);
    cancel_button.released().connect(dialog.slot_close());
    dialog.exec();
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

/// Builds a softened gruvbox-flavored dark QPalette.
///
/// Used on Windows in place of Qt Fusion's stock dark palette, whose ~#191919 backgrounds
/// against pure-white text produce harsh contrast. The gruvbox tones lift backgrounds to
/// ~#282828 and dim text to a warm cream (~#ebdbb2) for a softer reading experience.
#[cfg(target_os = "windows")]
unsafe fn build_gruvbox_dark_palette() -> CppBox<QPalette> {
    let make = |hex: &str| QColor::from_q_string(&QString::from_std_str(hex));

    let bg0   = make("#282828");
    let bg0_h = make("#1d2021");
    let bg0_s = make("#32302f");
    let bg1   = make("#3c3836");
    let bg2   = make("#504945");
    let bg3   = make("#665c54");
    let bg4   = make("#7c6f64");

    let fg0 = make("#fbf1c7");
    let fg1 = make("#ebdbb2");
    let fg4 = make("#a89984");

    let red_bright    = make("#fb4934");
    let blue          = make("#458588");
    let blue_bright   = make("#83a598");
    let purple_bright = make("#d3869b");
    let orange        = make("#d65d0e");

    let palette = QPalette::new();

    palette.set_color_2a(ColorRole::Window, &bg0);
    palette.set_color_2a(ColorRole::WindowText, &fg1);
    palette.set_color_2a(ColorRole::Base, &bg0_h);
    palette.set_color_2a(ColorRole::AlternateBase, &bg0_s);
    palette.set_color_2a(ColorRole::Text, &fg1);
    palette.set_color_2a(ColorRole::ToolTipBase, &bg1);
    palette.set_color_2a(ColorRole::ToolTipText, &fg1);
    // Fusion paints buttons, tool buttons, scrollbar handles, tabs and table headers
    // as a gradient derived from Button, with the gradient top reaching lighter(115).
    // Setting Button equal to Window keeps the gradient envelope flush with the panel
    // instead of producing a visibly brighter band; the widgets still read as buttons
    // through the gradient shading, the Mid/Dark border, and hover via Highlight.
    palette.set_color_2a(ColorRole::Button, &bg0);
    palette.set_color_2a(ColorRole::ButtonText, &fg1);
    palette.set_color_2a(ColorRole::BrightText, &red_bright);

    // Light/Midlight/Mid/Dark/Shadow must follow Qt's lightness ordering
    // (Light > Midlight > Button > Mid > Dark > Shadow) because Fusion uses these
    // for borders, separators, and gradient shading on toolbars/menus. A too-light
    // Mid (e.g. bg3) reads as a misplaced warm strip against the neutral Window;
    // a Midlight far above Window exaggerates toolbar gradients. Keep the range
    // tight so Fusion's auto-shading blends cleanly with the gruvbox panels.
    palette.set_color_2a(ColorRole::Light, &bg1);
    palette.set_color_2a(ColorRole::Midlight, &bg0_s);
    palette.set_color_2a(ColorRole::Mid, &bg0_h);
    palette.set_color_2a(ColorRole::Dark, &bg0_h);
    palette.set_color_2a(ColorRole::Shadow, &bg0_h);
    palette.set_color_2a(ColorRole::Highlight, &blue);
    palette.set_color_2a(ColorRole::HighlightedText, &fg0);
    palette.set_color_2a(ColorRole::Link, &blue_bright);
    palette.set_color_2a(ColorRole::LinkVisited, &purple_bright);
    palette.set_color_2a(ColorRole::PlaceholderText, &fg4);
    palette.set_color_2a(ColorRole::Accent, &orange);

    palette.set_color_3a(ColorGroup::Disabled, ColorRole::WindowText, &bg4);
    palette.set_color_3a(ColorGroup::Disabled, ColorRole::Text, &bg4);
    palette.set_color_3a(ColorGroup::Disabled, ColorRole::ButtonText, &bg4);
    palette.set_color_3a(ColorGroup::Disabled, ColorRole::Highlight, &bg2);
    palette.set_color_3a(ColorGroup::Disabled, ColorRole::HighlightedText, &fg4);
    palette.set_color_3a(ColorGroup::Disabled, ColorRole::PlaceholderText, &bg3);

    palette
}

thread_local! {
    /// Guards `reload_theme` against re-entrancy.
    static RELOADING_THEME: Cell<bool> = const { Cell::new(false) };
}

/// RAII guard that clears [`RELOADING_THEME`] when it leaves scope, even on early return/panic.
struct ReloadThemeGuard;
impl Drop for ReloadThemeGuard {
    fn drop(&mut self) {
        RELOADING_THEME.with(|flag| flag.set(false));
    }
}

/// This function refreshes theme-dependent UI elements to match the current native theme.
///
/// On Windows the dark variant is replaced with a softened gruvbox-flavored palette;
/// other platforms keep the native palette unchanged.
/// It updates elements that need manual intervention: icons with light/dark variants,
/// diagnostic filter button colors, and forces a full repaint.
pub unsafe fn reload_theme(app_ui: &AppUI) {

    // Avoid re-entering this function via `themeChanged()` signal during palette updates.
    if RELOADING_THEME.with(|flag| flag.replace(true)) {
        return;
    }
    let _reentry_guard = ReloadThemeGuard;

    // On dark themes, QDockWidget titlebar close/float buttons use QStyle standard pixmaps
    // that are dark-colored and invisible on dark backgrounds. Override them with breeze
    // theme icons that get palette-recolored by KIconEngine.
    #[cfg(target_os = "windows")] {
        let app = QCoreApplication::instance();
        let qapp = app.static_downcast::<QApplication>();

        // Detect the system color scheme via QStyleHints, not the application palette —
        // once we override with gruvbox below, the application palette no longer reflects
        // future system light/dark switches.
        let system_dark = QGuiApplication::style_hints().color_scheme() == ColorScheme::Dark;

        if system_dark {
            qapp.set_style_sheet(&QString::from_std_str(
                "QDockWidget {\
                    titlebar-close-icon: url(:/icons/breeze/actions/22/window-close.svg);\
                    titlebar-normal-icon: url(:/icons/breeze/actions/22/window-restore.svg);\
                }"
            ));
            let palette = build_gruvbox_dark_palette();
            QApplication::set_palette_1a(&palette);
        } else {
            qapp.set_style_sheet(&QString::from_std_str(""));
            // Restore the style's default palette for the current (light) scheme.
            let palette = QApplication::style().standard_palette();
            QApplication::set_palette_1a(&palette);
        }
    }

    // Re-apply the current native palette to force all widgets to refresh.
    #[cfg(not(target_os = "windows"))] {
        let native_palette = QGuiApplication::palette();
        QApplication::set_palette_1a(&native_palette);
    }

    // Select the appropriate GitHub icon based on the native theme.
    let github_icon = if is_dark_theme() {
        QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy())))
    } else {
        QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github-dark.svg", ASSETS_PATH.to_string_lossy())))
    };
    app_ui.welcome_page_ui().github_button().set_icon(&github_icon);

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

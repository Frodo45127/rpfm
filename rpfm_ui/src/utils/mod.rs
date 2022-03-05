//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::QGridLayout;
use qt_widgets::QLabel;
use qt_widgets::{QMessageBox, q_message_box::{Icon, StandardButton}};
use qt_widgets::QPushButton;
use qt_widgets::QWidget;
use qt_widgets::QMainWindow;

use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QObject;
use qt_core::SlotNoArgs;

use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::DynamicCast;
use cpp_core::Ptr;
use cpp_core::Ref;
use cpp_core::StaticUpcast;

use log::info;

use regex::Regex;
use sentry::Envelope;
use sentry::Level;
use sentry::protocol::{Attachment, EnvelopeItem, Event};

use std::convert::AsRef;
use std::fmt::Display;
use std::sync::atomic::{AtomicPtr, Ordering};

use rpfm_error::{ErrorKind, Result};
use rpfm_lib::{GAME_SELECTED, packedfile::PackedFileType, SENTRY_GUARD};

use crate::ASSETS_PATH;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::*;
use crate::locale::{qtr, qtre};
use crate::ORANGE;
use crate::SLIGHTLY_DARKER_GREY;
use crate::MEDIUM_DARKER_GREY;
use crate::DARK_GREY;
use crate::KINDA_WHITY_GREY;
use crate::EVEN_MORE_WHITY_GREY;
use crate::STATUS_BAR;
use crate::pack_tree::{get_color_correct, get_color_wrong, get_color_clean};

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
    let message = QString::from_std_str(&text.to_string());
    kmessage_widget_set_error_safe(&widget.as_ptr(), message.into_ptr())
}

/// This function takes the received KMessageWidget, and pushes a message onto it, making it visible in the process as a Warning.
///
/// It requires:
/// - widget: a pointer to the KMessageWidget.
/// - text: something that implements the trait `Display`, to put in the KMessageWidget.
#[allow(dead_code)]
pub unsafe fn show_message_warning<T: Display>(widget: &QPtr<QWidget>, text: T) {
    let message = QString::from_std_str(&text.to_string());
    kmessage_widget_set_warning_safe(&widget.as_ptr(), message.into_ptr())
}

/// This function takes the received KMessageWidget, and pushes a message onto it, making it visible in the process as an Info Message.
///
/// It requires:
/// - widget: a pointer to the KMessageWidget.
/// - text: something that implements the trait `Display`, to put in the KMessageWidget.
#[allow(dead_code)]
pub unsafe fn show_message_info<T: Display>(widget: &QPtr<QWidget>, text: T) {
    let message = QString::from_std_str(&text.to_string());
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
    QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
        icon,
        &title,
        &QString::from_std_str(&text.to_string()),
        QFlags::from(StandardButton::Ok),
        parent,
    ).exec();
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
        &QString::from_std_str(&text.to_string()),
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
    let explanation_label = QLabel::from_q_string_q_widget(&qtre("send_table_for_decoding_explanation", &[&GAME_SELECTED.read().unwrap().get_game_key_name(), &table_name]), &dialog);
    let cancel_button = QPushButton::from_q_string(&qtr("cancel"));
    let accept_button = QPushButton::from_q_string(&qtr("send"));

    main_grid.add_widget_5a(&explanation_label, 0, 0, 1, 2);
    main_grid.add_widget_5a(&cancel_button, 6, 0, 1, 1);
    main_grid.add_widget_5a(&accept_button, 6, 1, 1, 1);

    let send_table_slot = SlotNoArgs::new(&dialog, move || {
        if SENTRY_GUARD.read().unwrap().is_enabled() {
            let mut event = Event::new();
            event.level = Level::Info;
            event.message = Some(format!("{} - Request for table decoding: {}", GAME_SELECTED.read().unwrap().get_display_name(), table_name));

            let mut envelope = Envelope::from(event);
            let attatchment = Attachment {
                buffer: table_data.to_owned(),
                filename: table_name.to_owned(),
                ty: None
            };

            envelope.add_item(EnvelopeItem::Attachment(attatchment));
            SENTRY_GUARD.read().unwrap().send_envelope(envelope);
        }
    });

    accept_button.released().connect(&send_table_slot);
    accept_button.released().connect(dialog.slot_accept());
    cancel_button.released().connect(dialog.slot_close());
    dialog.exec();
}

/// This function deletes all widgets from a widget's layout.
pub unsafe fn clear_layout(widget: &QPtr<QWidget>) {
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

    widget.set_style_sheet(&QString::from_std_str(&format!("background-color: {}", style_sheet)));
}

/// Util function to get the PackedFileType of a PackedFile in a reliable way.
pub fn get_packed_file_type(path: &[String]) -> PackedFileType {
    let receiver = CENTRAL_COMMAND.send_background(Command::GetPackedFileType(path.to_vec()));
    let response = CentralCommand::recv(&receiver);
    match response {
        Response::PackedFileType(packed_file_type) => packed_file_type,
        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
    }
}

/// This functin returns the feature flags enabled for RPFM.
pub fn get_feature_flags() -> String {
    let mut feature_flags = String::new();

    #[cfg(feature = "support_modern_dds")] {
        feature_flags.push_str("support_modern_dds");
    }

    #[cfg(feature = "support_rigidmodel")] {
        if !feature_flags.is_empty() {
            feature_flags.push_str(", ");
        }
        feature_flags.push_str("support_rigidmodel");
    }

    if feature_flags.is_empty() {
        feature_flags.push_str("None");
    }

    feature_flags
}

/// This function creates the stylesheet used for the dark theme in windows.
pub fn create_dark_theme_stylesheet() -> String {
    format!("
        /* Normal buttons, with no rounded corners, dark background (darker when enabled), and colored borders. */

        QPushButton {{
            border-style: solid;
            border-width: 1px;
            padding-top: 5px;
            padding-bottom: 4px;
            padding-left: 10px;
            padding-right: 10px;
            border-color: {button_bd_off};
            color: {text_normal};
            background-color: {button_bg_off};
        }}
        QPushButton:hover {{
            border-color: {button_bd_hover};
            color: {text_highlighted};
            background-color: {button_bg_hover};
        }}
        QPushButton:pressed {{
            border-color: {button_bd_hover};
            color: {text_highlighted};
            background-color: {button_bg_on};
        }}
        QPushButton:checked {{
            border-color: {button_bd_hover};
            background-color: {button_bg_on};
        }}
        QPushButton:disabled {{
            color: #808086;
            background-color: {button_bg_off};
        }}

        /* Normal checkboxes */
        QCheckBox::indicator:unchecked {{
            border-style: solid;
            border-width: 1px;
            border-color: {checkbox_bd_off};
        }}
        /* Disabled due to the evanesce check bug.
        QCheckBox::indicator:checked {{
            height: 12px;
            width: 12px;
            border-style: solid;
            border-width: 1px;
            border-color: {checkbox_bd_off};
            image:url({assets_path}/icons/checkbox_check.png);
        }}
        QCheckBox::indicator:hover {{
            border-style: solid;
            border-width: 1px;
            border-color: {checkbox_bd_hover};
        }}
        */

        /* Tweaked TableView, so the Checkboxes are white and easy to see. */

        /* Checkboxes */
        QTableView::indicator:unchecked {{
            border-style: solid;
            border-width: 1px;
            border-color: {checkbox_bd_off};
        }}

        /* Disabled due to the evanesce check bug.
        QTableView::indicator:hover {{
            border-style: solid;
            border-width: 1px;
            border-color: {checkbox_bd_hover};
        }}
        QTableView::indicator:checked {{
            border-style: solid;
            border-width: 1px;
            border-color: {checkbox_bd_off};
            image:url({assets_path}/icons/checkbox_check.png);
        }}
        */
        /* Normal LineEdits, with no rounded corners, dark background (darker when enabled), and colored borders. */

        QLineEdit {{
            border-style: solid;
            border-width: 1px;
            padding-top: 3px;
            padding-bottom: 3px;
            padding-left: 3px;
            padding-right: 3px;
            border-color: {button_bd_off};
            color: {text_normal};
            background-color: {button_bg_off};
        }}
        QLineEdit:hover {{
            border-color: {button_bd_hover};
            color: {text_highlighted};
            background-color: {button_bg_hover};
        }}

        QLineEdit:disabled {{
            color: #808086;
            background-color: {button_bg_off};
        }}

        /* Combos, similar to buttons. */
        /* Disabled due to the unlimited items bug.
        QComboBox {{
            border-style: solid;
            border-width: 1px;
            padding-top: 3px;
            padding-bottom: 3px;
            padding-left: 10px;
            padding-right: 10px;
            border-color: {button_bd_off};
            color: {text_normal};
            background-color: {button_bg_off};
        }}*/

        /* TreeView, with no rounded corners and darker. */
        QTreeView {{
            border-style: solid;
            border-width: 1px;
            border-color: {button_bd_off};
        }}

        ",
        assets_path = ASSETS_PATH.to_string_lossy(),
        button_bd_hover = *ORANGE,
        button_bd_off = *SLIGHTLY_DARKER_GREY,
        button_bg_on = *SLIGHTLY_DARKER_GREY,
        button_bg_off = *MEDIUM_DARKER_GREY,
        button_bg_hover = *DARK_GREY,
        text_normal = *KINDA_WHITY_GREY,
        text_highlighted = *EVEN_MORE_WHITY_GREY,

        checkbox_bd_off = *KINDA_WHITY_GREY,
        checkbox_bd_hover = *ORANGE
    )
}

/// This function returns the a widget from the view if it exits, and an error if it doesn't.
pub unsafe fn find_widget<T: StaticUpcast<qt_core::QObject>>(main_widget: &QPtr<QWidget>, widget_name: &str) -> Result<QPtr<T>>
    where QObject: DynamicCast<T> {
    main_widget.find_child(widget_name).map_err(|_| ErrorKind::TemplateUIWidgetNotFound(widget_name.to_owned()).into())
}

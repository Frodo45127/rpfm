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
Module with all the utility functions, to make our programming lives easier.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::{QMessageBox, q_message_box::{Icon, StandardButton}};
use qt_widgets::QWidget;

use qt_core::QFlags;
use qt_core::QString;

use cpp_core::CastInto;
use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::MutPtr;
use cpp_core::Ref;

use std::fmt::Display;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::ORANGE;
use crate::SLIGHTLY_DARKER_GREY;
use crate::MEDIUM_DARKER_GREY;
use crate::DARK_GREY;
use crate::KINDA_WHITY_GREY;
use crate::EVEN_MORE_WHITY_GREY;
use crate::STATUS_BAR;

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

pub(crate) fn atomic_from_cpp_box<T: CppDeletable>(cpp_box: CppBox<T>) -> AtomicPtr<T> {
    AtomicPtr::new(cpp_box.into_raw_ptr())
}

pub(crate) fn atomic_from_mut_ptr<T: Sized>(ptr: MutPtr<T>) -> AtomicPtr<T> {
    AtomicPtr::new(ptr.as_mut_raw_ptr())
}

pub(crate) fn mut_ptr_from_atomic<T: Sized>(ptr: &AtomicPtr<T>) -> MutPtr<T> {
    unsafe { MutPtr::from_raw(ptr.load(Ordering::SeqCst)) }
}

pub(crate) fn ref_from_atomic<T: Sized>(ptr: &AtomicPtr<T>) -> Ref<T> {
    unsafe { Ref::from_raw(ptr.load(Ordering::SeqCst)).unwrap() }
}

pub(crate) fn ref_from_atomic_ref<T: Sized>(ptr: &AtomicPtr<T>) -> Ref<T> {
    unsafe { Ref::from_raw(ptr.load(Ordering::SeqCst)).unwrap() }
}

/// This functions logs the provided message to the status bar, so it can be seen by the user.
pub(crate) fn log_to_status_bar(text: &str) {
    unsafe { mut_ptr_from_atomic(&STATUS_BAR).show_message_2a(&QString::from_std_str(text), 2500); }
}

/// This function creates a modal dialog, for showing successes or errors.
///
/// It requires:
/// - parent: a pointer to the widget that'll be the parent of the dialog.
/// - text: something that implements the trait `Display`, to put in the dialog window.
/// - is_success: true for `Success` Dialog, false for `Error` Dialog.
pub unsafe fn show_dialog<T: Display>(parent: impl CastInto<MutPtr<QWidget>>, text: T, is_success: bool) {

    // Depending on the type of the dialog, set everything specific here.
    let title = if is_success { "Success!" } else { "Error!" };
    let icon = if is_success { Icon::Information } else { Icon::Critical };

    // Create and run the dialog.
    QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
        icon,
        &QString::from_std_str(title),
        &QString::from_std_str(&text.to_string()),
        QFlags::from(StandardButton::Ok),
        parent,
    ).exec();
}


/*
/// This function shows the tips in the PackedFile View. Remember to call "purge_them_all" before this!
pub fn display_help_tips(app_ui: &AppUI) {

    // Create the widget that'll act as a container for the view.
    let widget = Widget::new().into_raw();
    let widget_layout = create_grid_layout_unsafe(widget);
    unsafe { app_ui.packed_file_splitter.as_mut().unwrap().insert_widget(0, widget); }

    let label = Label::new(&QString::from_std_str("Welcome to Rusted PackFile Manager! Here you have some tips on how to use it:
    - Read the manual. It's in 'About/Open Manual'. It explains how to configure RPFM and how to use it.
    - To know what each option in 'Preferences' do, left the mouse over the option for one second and a tooltip will pop up.
    - In the 'About' Menu, in 'About RPFM' you can find links to the Source Code and the Patreon of the Project.")).into_raw();

    unsafe { widget_layout.as_mut().unwrap().add_widget((label as *mut Widget, 0, 0, 1, 1)); }
}
*/

/// This function creates a `GridLayout` for the provided widget with the settings we want.
pub unsafe fn create_grid_layout(mut widget: MutPtr<QWidget>) -> MutPtr<QGridLayout> {
    let mut widget_layout = QGridLayout::new_0a();
    widget.set_layout(&mut widget_layout);

    // Due to how Qt works, if we want a decent look on windows, we have to do some specific tweaks there.
    if cfg!(target_os = "windows") {
        widget_layout.set_contents_margins_4a(2, 2, 2, 2);
        widget_layout.set_spacing(1);
    }
    else {
        widget_layout.set_contents_margins_4a(0, 0, 0, 0);
        widget_layout.set_spacing(0);
    }

    widget_layout.into_ptr()
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
            border-color: #{button_bd_off};
            color: #{text_normal};
            background-color: #{button_bg_off};
        }}
        QPushButton:hover {{
            border-color: #{button_bd_hover};
            color: #{text_highlighted};
            background-color: #{button_bg_hover};
        }}
        QPushButton:pressed {{
            border-color: #{button_bd_hover};
            color: #{text_highlighted};
            background-color: #{button_bg_on};
        }}
        QPushButton:checked {{
            border-color: #{button_bd_hover};
            background-color: #{button_bg_on};
        }}
        QPushButton:disabled {{
            color: #808086;
            background-color: #{button_bg_off};
        }}

        /* Normal checkboxes */
        QCheckBox::indicator:unchecked {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
        }}
        /* Disabled due to the evanesce check bug.
        QCheckBox::indicator:checked {{
            height: 12px;
            width: 12px;
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
            image:url(img/checkbox_check.png);
        }}
        QCheckBox::indicator:hover {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_hover};
        }}
        */

        /* Tweaked TableView, so the Checkboxes are white and easy to see. */

        /* Checkboxes */
        QTableView::indicator:unchecked {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
        }}

        /* Disabled due to the evanesce check bug.
        QTableView::indicator:hover {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_hover};
        }}
        QTableView::indicator:checked {{
            border-style: solid;
            border-width: 1px;
            border-color: #{checkbox_bd_off};
            image:url(img/checkbox_check.png);
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
            border-color: #{button_bd_off};
            color: #{text_normal};
            background-color: #{button_bg_off};
        }}
        QLineEdit:hover {{
            border-color: #{button_bd_hover};
            color: #{text_highlighted};
            background-color: #{button_bg_hover};
        }}

        QLineEdit:disabled {{
            color: #808086;
            background-color: #{button_bg_off};
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
            border-color: #{button_bd_off};
            color: #{text_normal};
            background-color: #{button_bg_off};
        }}*/

        /* TreeView, with no rounded corners and darker. */
        QTreeView {{
            border-style: solid;
            border-width: 1px;
            border-color: #{button_bd_off};
        }}

        ",
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

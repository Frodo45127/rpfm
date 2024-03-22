//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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
use qt_widgets::QStatusBar;
use qt_widgets::{QMessageBox, q_message_box::{Icon, StandardButton}};
use qt_widgets::QWidget;

use qt_ui_tools::QUiLoader;

use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QObject;
use qt_core::WidgetAttribute;

use cpp_core::CastInto;
use cpp_core::CppBox;
use cpp_core::CppDeletable;
use cpp_core::DynamicCast;
use cpp_core::Ptr;
use cpp_core::Ref;
use cpp_core::StaticUpcast;

use anyhow::{anyhow, Result};

use rpfm_lib::integrations::log::*;

use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::ASSETS_PATH;
use crate::locale::qtr;

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

pub fn atomic_from_cpp_box<T: CppDeletable>(cpp_box: CppBox<T>) -> AtomicPtr<T> {
    AtomicPtr::new(cpp_box.into_raw_ptr())
}

pub fn atomic_from_q_box<T: StaticUpcast<QObject> + CppDeletable>(q_box: QBox<T>) -> AtomicPtr<T> {
    unsafe { AtomicPtr::new(q_box.as_mut_raw_ptr()) }
}

pub fn atomic_from_ptr<T: Sized>(ptr: Ptr<T>) -> AtomicPtr<T> {
    AtomicPtr::new(ptr.as_mut_raw_ptr())
}

pub fn q_ptr_from_atomic<T: Sized + StaticUpcast<QObject>>(ptr: &AtomicPtr<T>) -> QPtr<T> {
    unsafe { QPtr::from_raw(ptr.load(Ordering::SeqCst)) }
}

pub fn ptr_from_atomic<T: Sized>(ptr: &AtomicPtr<T>) -> Ptr<T> {
    unsafe { Ptr::from_raw(ptr.load(Ordering::SeqCst)) }
}

pub fn ref_from_atomic<T: Sized>(ptr: &AtomicPtr<T>) -> Ref<T> {
    unsafe { Ref::from_raw(ptr.load(Ordering::SeqCst)).unwrap() }
}

pub fn ref_from_atomic_ref<T: Sized>(ptr: &AtomicPtr<T>) -> Ref<T> {
    unsafe { Ref::from_raw(ptr.load(Ordering::SeqCst)).unwrap() }
}

/// This functions logs the provided message to the status bar, so it can be seen by the user.
pub fn log_to_status_bar<T: Display>(status_bar: QPtr<QStatusBar>, text: T) {
    unsafe { status_bar.show_message_2a(&QString::from_std_str(text.to_string()), 5000); }
    info!("{}", text);
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

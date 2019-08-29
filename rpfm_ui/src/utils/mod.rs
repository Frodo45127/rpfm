//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
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

use qt_widgets::grid_layout::GridLayout;
use qt_widgets::layout::Layout;
use qt_widgets::message_box::{MessageBox, Icon};
use qt_widgets::widget::Widget;

use qt_core::flags::Flags;

use cpp_utils::CppBox;

use std::fmt::Display;

use crate::ORANGE;
use crate::SLIGHTLY_DARKER_GREY;
use crate::MEDIUM_DARKER_GREY;
use crate::DARK_GREY;
use crate::KINDA_WHITY_GREY;
use crate::EVEN_MORE_WHITY_GREY;
use crate::QString;

//----------------------------------------------------------------------------//
//              Utility functions (helpers and stuff like that)
//----------------------------------------------------------------------------//

/// This function creates a modal dialog, for showing successes or errors.
///
/// It requires:
/// - parent: a pointer to the widget that'll be the parent of the dialog.
/// - text: something that implements the trait `Display`, to put in the dialog window.
/// - is_success: true for `Success` Dialog, false for `Error` Dialog.
pub fn show_dialog<T: Display>(parent: *mut Widget, text: T, is_success: bool) {

    // Depending on the type of the dialog, set everything specific here.
    let title = if is_success { "Success!" } else { "Error!" };
    let icon = if is_success { Icon::Information } else { Icon::Critical };

    // Create and run the dialog.
    unsafe { MessageBox::new_unsafe((
        icon,
        &QString::from_std_str(title),
        &QString::from_std_str(&text.to_string()),
        Flags::from_int(1024), // Ok button.
        parent,
    )) }.exec();
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
///
/// This is the safe version for `&mut Widget`. REMEMBER TO DO AN `into_raw` AFTER USING THIS!
pub fn create_grid_layout_safe() -> CppBox<GridLayout> {
    let mut widget_layout = GridLayout::new();
    
    // Due to how Qt works, if we want a decent look on windows, we have to do some specific tweaks there.
    if cfg!(target_os = "windows") {
        widget_layout.set_contents_margins((2, 2, 2, 2));
        widget_layout.set_spacing(1);
    }
    else {
        widget_layout.set_contents_margins((0, 0, 0, 0));
        widget_layout.set_spacing(0);            
    }

    widget_layout
}

/// This function creates a `GridLayout` for the provided widget with the settings we want.
///
/// This is the unsafe version for `*mut Widget`.
pub fn create_grid_layout_unsafe(widget: *mut Widget) -> *mut GridLayout {
    let mut widget_layout = GridLayout::new();
    unsafe { widget.as_mut().unwrap().set_layout(widget_layout.as_mut_ptr() as *mut Layout); }
    
    // Due to how Qt works, if we want a decent look on windows, we have to do some specific tweaks there.
    if cfg!(target_os = "windows") {
        widget_layout.set_contents_margins((2, 2, 2, 2));
        widget_layout.set_spacing(1);
    }
    else {
        widget_layout.set_contents_margins((0, 0, 0, 0));
        widget_layout.set_spacing(0);           
    }

    widget_layout.into_raw()
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
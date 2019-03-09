//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// In this file are all the helper functions used by the PackedFile Tables.

use qt_widgets::abstract_button::AbstractButton;
use qt_widgets::button_group::ButtonGroup;
use qt_widgets::dialog::Dialog;
use qt_widgets::double_spin_box::DoubleSpinBox;
use qt_widgets::grid_layout::GridLayout;
use qt_widgets::group_box::GroupBox;
use qt_widgets::label::Label;
use qt_widgets::layout::Layout;
use qt_widgets::line_edit::LineEdit;
use qt_widgets::push_button::PushButton;
use qt_widgets::radio_button::RadioButton;
use qt_widgets::widget::Widget;

use qt_core::connection::Signal;

use cpp_utils::StaticCast;

use std::f32;

use crate::QString;
use crate::AppUI;

/// This function creates the entire "Apply Maths" dialog for tables. It returns the operation to apply and the value.
pub fn create_apply_maths_dialog(app_ui: &AppUI) -> Option<(String, f64)> {

    // Create and configure the "Apply Maths" Dialog.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Apply Maths"));
    dialog.set_modal(true);
    let main_grid = GridLayout::new().into_raw();

    // Create the button group with the different operations, and set by default the "+" selected.
    let operations_frame = GroupBox::new(&QString::from_std_str("Operations")).into_raw();
    let operations_grid = GridLayout::new().into_raw();
    unsafe { operations_frame.as_mut().unwrap().set_layout(operations_grid as *mut Layout); }

    let mut button_group = ButtonGroup::new();
    let mut operation_plus = RadioButton::new(&QString::from_std_str("+"));
    let mut operation_minus = RadioButton::new(&QString::from_std_str("-"));
    let mut operation_mult = RadioButton::new(&QString::from_std_str("*"));
    let mut operation_div = RadioButton::new(&QString::from_std_str("/"));
    unsafe { button_group.add_button(operation_plus.static_cast_mut() as *mut AbstractButton); }
    unsafe { button_group.add_button(operation_minus.static_cast_mut() as *mut AbstractButton); }
    unsafe { button_group.add_button(operation_mult.static_cast_mut() as *mut AbstractButton); }
    unsafe { button_group.add_button(operation_div.static_cast_mut() as *mut AbstractButton); }
    operation_plus.click();

    // Create a little frame with some instructions.
    let instructions_frame = GroupBox::new(&QString::from_std_str("Instructions")).into_raw();
    let instructions_grid = GridLayout::new().into_raw();
    unsafe { instructions_frame.as_mut().unwrap().set_layout(instructions_grid as *mut Layout); }
    let mut instructions_label = Label::new(&QString::from_std_str(
    "It's easy:
     - Choose the operation on the left.
     - Write the operand on the SpinBox.
     - Click the button on the right.
     - ???
     - Profit!
    "    
    ));
    unsafe { instructions_grid.as_mut().unwrap().add_widget((instructions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }

    // We use a double SpinBox for the value, so we can do any operation with F32 floats.
    let mut value_spin_box = DoubleSpinBox::new();
    value_spin_box.set_decimals(3);
    value_spin_box.set_range(f32::MIN as f64, f32::MAX as f64);
    let apply_button = PushButton::new(&QString::from_std_str("Apply")).into_raw();

    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_plus.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }
    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_minus.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_mult.static_cast_mut() as *mut Widget, 2, 0, 1, 1)); }
    unsafe { operations_grid.as_mut().unwrap().add_widget((operation_div.static_cast_mut() as *mut Widget, 3, 0, 1, 1)); }

    unsafe { main_grid.as_mut().unwrap().add_widget((operations_frame as *mut Widget, 0, 0, 4, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((instructions_frame as *mut Widget, 1, 1, 3, 2)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((value_spin_box.static_cast_mut() as *mut Widget, 0, 1, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((apply_button as *mut Widget, 0, 2, 1, 1)); }
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    unsafe { apply_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    if dialog.exec() == 1 {
        let operation = unsafe { button_group.checked_button().as_ref().unwrap().text().to_std_string() };
        let value = value_spin_box.value();
        Some((operation, value))
    } else { None }
}

/// This function creates the entire "Rewrite selection" dialog for tables. It returns the rewriting sequence, or None.
pub fn create_rewrite_selection_dialog(app_ui: &AppUI) -> Option<String> {

    // Create and configure the dialog.
    let mut dialog = unsafe { Dialog::new_unsafe(app_ui.window as *mut Widget) };
    dialog.set_window_title(&QString::from_std_str("Rewrite Selection"));
    dialog.set_modal(true);
    dialog.resize((400, 50));
    let main_grid = GridLayout::new().into_raw();

    // Create a little frame with some instructions.
    let instructions_frame = GroupBox::new(&QString::from_std_str("Instructions")).into_raw();
    let instructions_grid = GridLayout::new().into_raw();
    unsafe { instructions_frame.as_mut().unwrap().set_layout(instructions_grid as *mut Layout); }
    let mut instructions_label = Label::new(&QString::from_std_str(
    "\
It's easy, but you'll not understand it without an example, so here it's one:
 - You selected a cell that says 'you'.
 - Write 'whatever {x} want' in the box below.
 - Hit 'Accept'.
 - RPFM will turn that into 'whatever you want' and put it in the cell.
And, in case you ask, works with numeric cells too, as long as the resulting text is a valid number.
    "    
    ));
    unsafe { instructions_grid.as_mut().unwrap().add_widget((instructions_label.static_cast_mut() as *mut Widget, 0, 0, 1, 1)); }

    let mut rewrite_sequence_line_edit = LineEdit::new(());
    rewrite_sequence_line_edit.set_placeholder_text(&QString::from_std_str("Write here whatever you want. {x} it's your current text."));
    let accept_button = PushButton::new(&QString::from_std_str("Accept")).into_raw();

    unsafe { main_grid.as_mut().unwrap().add_widget((instructions_frame as *mut Widget, 0, 0, 1, 2)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((rewrite_sequence_line_edit.static_cast_mut() as *mut Widget, 1, 0, 1, 1)); }
    unsafe { main_grid.as_mut().unwrap().add_widget((accept_button as *mut Widget, 1, 1, 1, 1)); }
    unsafe { dialog.set_layout(main_grid as *mut Layout); }

    unsafe { accept_button.as_mut().unwrap().signals().released().connect(&dialog.slots().accept()); }

    if dialog.exec() == 1 { 
        let prefix = rewrite_sequence_line_edit.text().to_std_string();
        if prefix.is_empty() { None } else { Some(rewrite_sequence_line_edit.text().to_std_string()) } 
    } else { None }
}

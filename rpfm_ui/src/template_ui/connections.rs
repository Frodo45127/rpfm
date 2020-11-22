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
Module with all the code to connect all the template actions.
!*/

use super::{TemplateUI, SaveTemplateUI, slots::{TemplateUISlots, SaveTemplateUISlots}};

/// This function connects all the actions from the provided `SaveTemplateUI` with their slots in `SaveTemplateUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections_template(ui: &TemplateUI, slots: &TemplateUISlots) {
    ui.accept_button.released().connect(ui.dialog.slot_accept());

    ui.options.borrow().iter().map(|(_, y)| y).for_each(|x| { x.toggled().connect(&slots.toggle_required); });
}

/// This function connects all the actions from the provided `SaveTemplateUI` with their slots in `SaveTemplateUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections_save_template(ui: &SaveTemplateUI, slots: &SaveTemplateUISlots) {
    ui.step_2_add_button.released().connect(&slots.step_2_slot_add);
    ui.step_2_remove_button.released().connect(&slots.step_2_slot_remove);
    ui.step_3_add_button.released().connect(&slots.step_3_slot_add);
    ui.step_3_remove_button.released().connect(&slots.step_3_slot_remove);
}


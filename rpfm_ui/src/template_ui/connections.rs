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
    //ui.accept_button.released().connect(ui.dialog.slot_accept());

    //ui.options.borrow().iter().map(|(_, y)| y).for_each(|x| { x.toggled().connect(&slots.toggle_required); });
}

/// This function connects all the actions from the provided `SaveTemplateUI` with their slots in `SaveTemplateUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections_save_template(ui: &SaveTemplateUI, slots: &SaveTemplateUISlots) {
    ui.sections_add_button.released().connect(&slots.sections_slot_add);
    ui.sections_remove_button.released().connect(&slots.sections_slot_remove);
    ui.options_add_button.released().connect(&slots.options_slot_add);
    ui.options_remove_button.released().connect(&slots.options_slot_remove);
    ui.params_add_button.released().connect(&slots.params_slot_add);
    ui.params_remove_button.released().connect(&slots.params_slot_remove);
}


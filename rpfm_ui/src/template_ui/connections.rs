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

use qt_widgets::QCheckBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QLineEdit;
use qt_widgets::QComboBox;
use qt_widgets::QSpinBox;

use rpfm_lib::template::ParamType;

use super::{TemplateUI, SaveTemplateUI, slots::{TemplateUISlots, SaveTemplateUISlots}};

/// This function connects all the actions from the provided `SaveTemplateUI` with their slots in `SaveTemplateUISlots`.
///
/// This function is just glue to trigger after initializing both, the actions and the slots. It's here
/// to not polute the other modules with a ton of connections.
pub unsafe fn set_connections_template(ui: &TemplateUI, slots: &TemplateUISlots) {
    ui.params.borrow().iter().filter_map(|(_, widget, param_type, is_required)| if *is_required { Some((widget, param_type)) } else { None }).for_each(|(widget, param_type)| {
        match param_type {
            ParamType::Checkbox => widget.static_downcast::<QCheckBox>().toggled().connect(&slots.update_view),
            ParamType::Integer => widget.static_downcast::<QSpinBox>().editing_finished().connect(&slots.update_view),
            ParamType::Float => widget.static_downcast::<QDoubleSpinBox>().editing_finished().connect(&slots.update_view),
            ParamType::Text => widget.static_downcast::<QLineEdit>().editing_finished().connect(&slots.update_view),

            // For these types, first ensure what type of field do we have!!!!
            ParamType::TableField(_) => {
                if !widget.dynamic_cast::<QComboBox>().is_null() {
                    widget.static_downcast::<QComboBox>().current_index_changed().connect(&slots.update_view)
                } else if !widget.dynamic_cast::<QLineEdit>().is_null() {
                    widget.static_downcast::<QLineEdit>().editing_finished().connect(&slots.update_view)
                } else {
                    unimplemented!()
                }
            },
            //ParamType::Table(_) => {},
        };
    });

    ui.wazard.current_id_changed().connect(&slots.edited_required);
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


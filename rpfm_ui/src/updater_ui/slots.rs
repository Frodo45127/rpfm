//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::QApplication;

use qt_core::QBox;
use qt_core::SlotNoArgs;

use getset::*;

use std::env::current_exe;
use std::process::{Command as SystemCommand, exit};
use std::rc::Rc;

use rpfm_ui_common::clone;

use crate::communications::*;
use crate::utils::{qtr, show_dialog};
use super::UpdaterUI;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct UpdaterUISlots {
    update_program: QBox<SlotNoArgs>,
    update_schemas: QBox<SlotNoArgs>,
    update_twautogen: QBox<SlotNoArgs>,
    update_old_ak: QBox<SlotNoArgs>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl UpdaterUISlots {

    pub unsafe fn new(ui: &Rc<UpdaterUI>) -> Self {
        let update_program = SlotNoArgs::new(ui.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Update Program");
                ui.update_program_button.set_text(&qtr("updater_update_program_updating"));
                ui.update_program_button.set_enabled(false);

                match send_ipc_command_result_async(Command::UpdateMainProgram, response_extractor!()) {
                    Ok(()) => {
                        ui.update_program_button.set_text(&qtr("updater_update_program_updated"));

                        // Re-enable the button so it can be used to restart the program.
                        ui.update_program_button.set_enabled(true);
                        ui.update_program_button.disconnect();
                        ui.update_program_button.released().connect(&SlotNoArgs::new(ui.main_widget(), move || {

                            // Make sure we close both threads and the window. In windows the main window doesn't get closed for some reason.
                            QApplication::close_all_windows();

                            let exe_path = current_exe().unwrap();
                            SystemCommand::new(exe_path).spawn().unwrap();
                            exit(10);
                        }));
                    },
                    Err(error) => {
                        show_dialog(ui.dialog(), error, false);
                        ui.update_program_button.set_text(&qtr("updater_update_program_error"));
                    }
                }
            }
        ));

        let update_schemas = SlotNoArgs::new(ui.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Update Schemas");
                ui.update_schemas_button.set_text(&qtr("updater_update_schemas_updating"));
                ui.update_schemas_button.set_enabled(false);

                match send_ipc_command_result_async(Command::UpdateSchemas, response_extractor!()) {
                    Ok(()) => {
                        ui.update_schemas_button.set_text(&qtr("updater_update_schemas_updated"));
                    },
                    Err(error) => {
                        show_dialog(ui.dialog(), error, false);
                        ui.update_schemas_button.set_text(&qtr("updater_update_schemas_error"));
                    }
                }
            }
        ));

        let update_twautogen = SlotNoArgs::new(ui.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Update TW Autogen");
                ui.update_twautogen_button.set_text(&qtr("updater_update_twautogen_updating"));
                ui.update_twautogen_button.set_enabled(false);

                match send_ipc_command_result_async(Command::UpdateLuaAutogen, response_extractor!()) {
                    Ok(()) => {
                        ui.update_twautogen_button.set_text(&qtr("updater_update_twautogen_updated"));
                    },
                    Err(error) => {
                        show_dialog(ui.dialog(), error, false);
                        ui.update_twautogen_button.set_text(&qtr("updater_update_twautogen_error"));
                    }
                }
            }
        ));

        let update_old_ak = SlotNoArgs::new(ui.main_widget(), clone!(
            ui => move || {
                rpfm_telemetry::track_action("Update Empire/Napoleon AK");
                ui.update_old_ak_button.set_text(&qtr("updater_update_old_ak_updating"));
                ui.update_old_ak_button.set_enabled(false);

                match send_ipc_command_result_async(Command::UpdateEmpireAndNapoleonAK, response_extractor!()) {
                    Ok(()) => {
                        ui.update_old_ak_button.set_text(&qtr("updater_update_old_ak_updated"));
                    },
                    Err(error) => {
                        show_dialog(ui.dialog(), error, false);
                        ui.update_old_ak_button.set_text(&qtr("updater_update_old_ak_error"));
                    }
                }
            }
        ));

        Self {
            update_program,
            update_schemas,
            update_twautogen,
            update_old_ak,
        }
    }
}

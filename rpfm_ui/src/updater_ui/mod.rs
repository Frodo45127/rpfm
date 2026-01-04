//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDialog;
use qt_widgets::{QWidget, QPushButton, QDialogButtonBox, QLabel, QGroupBox};

use qt_core::QBox;
use qt_core::QPtr;

use anyhow::Result;
use getset::*;

use std::fmt::Display;
use std::rc::Rc;

use rpfm_ipc::helpers::APIResponse;

use rpfm_lib::integrations::git::GitResponse;

use rpfm_ui_common::PROGRAM_PATH;
use rpfm_ui_common::utils::*;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::settings_ui::backend::{settings_bool, settings_string};
use crate::updater_ui::slots::UpdaterUISlots;
use crate::utils::{qtr, qtre};

pub const CHANGELOG_FILE: &str = "Changelog.txt";

pub const STABLE: &str = "Stable";
pub const BETA: &str = "Beta";

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/updater_dialog.ui";
const VIEW_RELEASE: &str = "ui/updater_dialog.ui";

mod slots;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct UpdaterUI {
    main_widget: QBox<QWidget>,
    update_schemas_button: QPtr<QPushButton>,
    update_program_button: QPtr<QPushButton>,
    update_twautogen_button: QPtr<QPushButton>,
    update_old_ak_button: QPtr<QPushButton>,
    accept_button: QPtr<QPushButton>,
    cancel_button: QPtr<QPushButton>,
}

/// This enum controls the channels through where RPFM will try to update.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UpdateChannel {
    Stable,
    Beta
}

//---------------------------------------------------------------------------//
//                              UI functions
//---------------------------------------------------------------------------//

impl UpdaterUI {

    /// This function checks for updates, and if it find any update, it shows the update dialog.
    pub unsafe fn new_with_precheck(app_ui: &Rc<AppUI>) -> Result<()> {
        let mut update_available = false;

        let mut receiver_updates = None;
        let mut receiver_schema_updates = None;
        let mut receiver_lua_autogen_updates = None;
        let mut receiver_old_ak_updates = None;

        if settings_bool("check_updates_on_start") {
            receiver_updates = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckUpdates));
        }

        if settings_bool("check_schema_updates_on_start") {
            receiver_schema_updates = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckSchemaUpdates));
        }

        if settings_bool("check_lua_autogen_updates_on_start") {
            receiver_lua_autogen_updates = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckLuaAutogenUpdates));
        }

        if settings_bool("check_old_ak_updates_on_start") {
            receiver_old_ak_updates = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckEmpireAndNapoleonAKUpdates));
        }

        let updates_for_program = if let Some(receiver) = receiver_updates {
            let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
            match response {
                Response::APIResponse(response) => {
                    match response {
                        APIResponse::NewStableUpdate(_) |
                        APIResponse::NewBetaUpdate(_) |
                        APIResponse::NewUpdateHotfix(_) => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        let updates_for_schema = if let Some(receiver) = receiver_schema_updates {
            let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
            match response {
                Response::APIResponseGit(response) => {
                    match response {
                        GitResponse::NoLocalFiles |
                        GitResponse::NewUpdate |
                        GitResponse::Diverged => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        let updates_for_twautogen = if let Some(receiver) = receiver_lua_autogen_updates {
            let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
            match response {
                Response::APIResponseGit(response) => {
                    match response {
                        GitResponse::NoLocalFiles |
                        GitResponse::NewUpdate |
                        GitResponse::Diverged => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        let updates_for_old_ak = if let Some(receiver) = receiver_old_ak_updates {
            let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver);
            match response {
                Response::APIResponseGit(response) => {
                    match response {
                        GitResponse::NoLocalFiles |
                        GitResponse::NewUpdate |
                        GitResponse::Diverged => {
                            update_available |= true;
                        }
                        _ => {},
                    }
                    Some(response)
                }

                Response::Error(_) => None,
                _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
            }
        } else {
            None
        };

        // Only show the dialog if there are updates.
        if update_available {
            Self::new(app_ui, updates_for_program, updates_for_schema, updates_for_twautogen, updates_for_old_ak)?;
        }

        Ok(())
    }

    pub unsafe fn new(app_ui: &Rc<AppUI>, precheck_program: Option<APIResponse>, precheck_schema: Option<GitResponse>, precheck_twautogen: Option<GitResponse>, precheck_old_ak: Option<GitResponse>) -> Result<()> {

        // Load the UI Template.
        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(app_ui.main_window(), template_path)?;

        let info_groupbox: QPtr<QGroupBox> = find_widget(&main_widget.static_upcast(), "info_groupbox")?;
        let info_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "info_label")?;
        let update_schemas_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_schemas_label")?;
        let update_program_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_program_label")?;
        let update_twautogen_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_twautogen_label")?;
        let update_old_ak_label: QPtr<QLabel> = find_widget(&main_widget.static_upcast(), "update_old_ak_label")?;
        let update_schemas_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_schemas_button")?;
        let update_program_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_program_button")?;
        let update_twautogen_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_twautogen_button")?;
        let update_old_ak_button: QPtr<QPushButton> = find_widget(&main_widget.static_upcast(), "update_old_ak_button")?;
        let button_box: QPtr<QDialogButtonBox> = find_widget(&main_widget.static_upcast(), "button_box")?;
        let accept_button: QPtr<QPushButton> = button_box.button(StandardButton::Ok);
        let cancel_button: QPtr<QPushButton> = button_box.button(StandardButton::Cancel);

        let changelog_path = PROGRAM_PATH.join(CHANGELOG_FILE);

        info_groupbox.set_title(&qtr("updater_info_title"));
        info_label.set_text(&qtre("updater_info", &[&changelog_path.to_string_lossy(), &settings_string("update_channel")]));
        info_label.set_open_external_links(true);

        update_program_label.set_text(&qtr("updater_update_program"));
        update_schemas_label.set_text(&qtr("updater_update_schemas"));
        update_twautogen_label.set_text(&qtr("updater_update_twautogen"));
        update_old_ak_label.set_text(&qtr("updater_update_old_ak"));

        update_program_button.set_text(&qtr("updater_update_program_checking"));
        update_schemas_button.set_text(&qtr("updater_update_schemas_checking"));
        update_twautogen_button.set_text(&qtr("updater_update_twautogen_checking"));
        update_old_ak_button.set_text(&qtr("updater_update_old_ak_checking"));

        update_program_button.set_enabled(false);
        update_schemas_button.set_enabled(false);
        update_twautogen_button.set_enabled(false);
        update_old_ak_button.set_enabled(false);

        // Show the dialog before checking for updates.
        main_widget.static_downcast::<QDialog>().set_window_title(&qtr("updater_title"));
        main_widget.static_downcast::<QDialog>().show();

        let receiver_program = CENTRAL_COMMAND.read().unwrap().send(Command::CheckUpdates);
        let receiver_schemas = CENTRAL_COMMAND.read().unwrap().send(Command::CheckSchemaUpdates);
        let receiver_twautogen = CENTRAL_COMMAND.read().unwrap().send(Command::CheckLuaAutogenUpdates);
        let receiver_old_ak = CENTRAL_COMMAND.read().unwrap().send(Command::CheckEmpireAndNapoleonAKUpdates);

        // If we have prechecks done, do not re-check for updates on them.
        match precheck_program {
            Some(response) => {
                match response {
                    APIResponse::NewStableUpdate(last_release) |
                    APIResponse::NewBetaUpdate(last_release) |
                    APIResponse::NewUpdateHotfix(last_release) => {
                        update_program_button.set_text(&qtre("updater_update_program_available", &[&last_release]));
                        update_program_button.set_enabled(true);
                    }
                    APIResponse::NoUpdate |
                    APIResponse::UnknownVersion => {
                        update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                    }
                }
            }
            None => {

                let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver_program);
                match response {
                    Response::APIResponse(response) => {
                        match response {
                            APIResponse::NewStableUpdate(last_release) |
                            APIResponse::NewBetaUpdate(last_release) |
                            APIResponse::NewUpdateHotfix(last_release) => {
                                update_program_button.set_text(&qtre("updater_update_program_available", &[&last_release]));
                                update_program_button.set_enabled(true);
                            }
                            APIResponse::NoUpdate |
                            APIResponse::UnknownVersion => {
                                update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        match precheck_schema {
            Some(response) => {
                match response {
                    GitResponse::NoLocalFiles |
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_schemas_button.set_text(&qtr("updater_update_schemas_available"));
                        update_schemas_button.set_enabled(true);
                    }
                    GitResponse::NoUpdate => {
                        update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                    }
                }
            }
            None => {
                let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver_schemas);
                match response {
                    Response::APIResponseGit(response) => {
                        match response {
                            GitResponse::NoLocalFiles |
                            GitResponse::NewUpdate |
                            GitResponse::Diverged => {
                                update_schemas_button.set_text(&qtr("updater_update_schemas_available"));
                                update_schemas_button.set_enabled(true);
                            }
                            GitResponse::NoUpdate => {
                                update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        match precheck_twautogen {
            Some(response) => {
                match response {
                    GitResponse::NoLocalFiles |
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_twautogen_button.set_text(&qtr("updater_update_twautogen_available"));
                        update_twautogen_button.set_enabled(true);
                    }
                    GitResponse::NoUpdate => {
                        update_twautogen_button.set_text(&qtr("updater_update_twautogen_no_updates"));
                    }
                }
            }
            None => {
                let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver_twautogen);
                match response {
                    Response::APIResponseGit(response) => {
                        match response {
                            GitResponse::NoLocalFiles |
                            GitResponse::NewUpdate |
                            GitResponse::Diverged => {
                                update_twautogen_button.set_text(&qtr("updater_update_twautogen_available"));
                                update_twautogen_button.set_enabled(true);
                            }
                            GitResponse::NoUpdate => {
                                update_twautogen_button.set_text(&qtr("updater_update_twautogen_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_twautogen_button.set_text(&qtr("updater_update_twautogen_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        match precheck_old_ak {
            Some(response) => {
                match response {
                    GitResponse::NoLocalFiles |
                    GitResponse::NewUpdate |
                    GitResponse::Diverged => {
                        update_old_ak_button.set_text(&qtr("updater_update_old_ak_available"));
                        update_old_ak_button.set_enabled(true);
                    }
                    GitResponse::NoUpdate => {
                        update_old_ak_button.set_text(&qtr("updater_update_old_ak_no_updates"));
                    }
                }
            }
            None => {
                let response = CENTRAL_COMMAND.read().unwrap().recv_try(&receiver_old_ak);
                match response {
                    Response::APIResponseGit(response) => {
                        match response {
                            GitResponse::NoLocalFiles |
                            GitResponse::NewUpdate |
                            GitResponse::Diverged => {
                                update_old_ak_button.set_text(&qtr("updater_update_old_ak_available"));
                                update_old_ak_button.set_enabled(true);
                            }
                            GitResponse::NoUpdate => {
                                update_old_ak_button.set_text(&qtr("updater_update_old_ak_no_updates"));
                            }
                        }
                    }

                    Response::Error(_) => {
                        update_old_ak_button.set_text(&qtr("updater_update_old_ak_no_updates"));
                    }
                    _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                }
            },
        }

        let ui = Rc::new(Self {
            main_widget,
            update_schemas_button,
            update_program_button,
            update_twautogen_button,
            update_old_ak_button,
            accept_button,
            cancel_button,
        });

        let slots = UpdaterUISlots::new(&ui);
        ui.set_connections(&slots);

        Ok(())
    }

    pub unsafe fn set_connections(&self, slots: &UpdaterUISlots) {
        self.update_program_button.released().connect(slots.update_program());
        self.update_schemas_button.released().connect(slots.update_schemas());
        self.update_twautogen_button.released().connect(slots.update_twautogen());
        self.update_old_ak_button.released().connect(slots.update_old_ak());

        self.accept_button.released().connect(self.dialog().slot_accept());
        self.cancel_button.released().connect(self.dialog().slot_close());
    }

    pub unsafe fn dialog(&self) -> QPtr<QDialog> {
        self.main_widget().static_downcast::<QDialog>()
    }
}


impl Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        Display::fmt(match &self {
            UpdateChannel::Stable => STABLE,
            UpdateChannel::Beta => BETA,
        }, f)
    }
}

/// This function returns the currently selected update channel.
pub fn update_channel() -> UpdateChannel {
    match &*settings_string("update_channel") {
        BETA => UpdateChannel::Beta,
        _ => UpdateChannel::Stable,
    }
}

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
use qt_core::QEventLoop;
use qt_core::QPtr;
use qt_core::QTimer;
use qt_core::SlotNoArgs;

use anyhow::Result;
use crossbeam::channel::Receiver;
use getset::*;

use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use rpfm_ipc::helpers::APIResponse;
use rpfm_ipc::settings_keys::*;

use rpfm_lib::integrations::git::GitResponse;

use rpfm_telemetry::warn;

use rpfm_ui_common::clone;
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

/// Interval, in milliseconds, between polls of the update checks launched on start.
const PRECHECK_POLL_INTERVAL_MS: i32 = 250;

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

/// Update checks launched on start. Each receiver is dropped once its check answers or disconnects.
#[derive(Default)]
struct Precheck {
    receiver_program: Option<Receiver<Response>>,
    receiver_schema: Option<Receiver<Response>>,
    receiver_twautogen: Option<Receiver<Response>>,
    receiver_old_ak: Option<Receiver<Response>>,
    program: Option<APIResponse>,
    schema: Option<GitResponse>,
    twautogen: Option<GitResponse>,
    old_ak: Option<GitResponse>,
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

impl Precheck {

    /// Checks all pending receivers once, without blocking.
    ///
    /// # Returns
    ///
    /// `true` if all checks have finished, `false` otherwise.
    fn poll(&mut self) -> bool {
        poll_check(&mut self.receiver_program, &mut self.program, "program", |response| match response {
            Response::APIResponse(response) => Some(response),
            Response::Error(_) => None,
            response => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
        });
        poll_check(&mut self.receiver_schema, &mut self.schema, "schema", git_response);
        poll_check(&mut self.receiver_twautogen, &mut self.twautogen, "TW autogen", git_response);
        poll_check(&mut self.receiver_old_ak, &mut self.old_ak, "Empire/Napoleon AK", git_response);

        self.is_finished()
    }

    /// Returns `true` if there are no checks waiting for an answer.
    fn is_finished(&self) -> bool {
        self.receiver_program.is_none() &&
        self.receiver_schema.is_none() &&
        self.receiver_twautogen.is_none() &&
        self.receiver_old_ak.is_none()
    }

    /// Returns `true` if any of the finished checks found an update.
    fn update_available(&self) -> bool {
        matches!(self.program, Some(APIResponse::NewStableUpdate(_) | APIResponse::NewBetaUpdate(_) | APIResponse::NewUpdateHotfix(_))) ||
        [&self.schema, &self.twautogen, &self.old_ak].iter().any(|response| matches!(response, Some(GitResponse::NoLocalFiles | GitResponse::NewUpdate | GitResponse::Diverged)))
    }
}

/// Checks a pending receiver without blocking, storing its answer and dropping it once it's done.
///
/// # Arguments
///
/// * `receiver` - The receiver of the check. Set to `None` once the check answers or disconnects.
/// * `result` - Where the answer of the check is stored.
/// * `name` - Name of the check, used for logging.
/// * `extract` - Turns the response into the answer of the check. `None` means the check failed.
fn poll_check<T>(receiver: &mut Option<Receiver<Response>>, result: &mut Option<T>, name: &str, extract: fn(Response) -> Option<T>) {
    let Some(pending) = receiver.as_ref() else { return };
    match pending.try_recv() {
        Ok(response) => {
            *result = extract(response);
            *receiver = None;
        }
        Err(error) => if error.is_disconnected() {
            warn!("Update precheck ({name}) skipped: background channel disconnected.");
            *receiver = None;
        }
    }
}

/// Extracts the answer of a git update check from its response.
fn git_response(response: Response) -> Option<GitResponse> {
    match response {
        Response::APIResponseGit(response) => Some(response),
        Response::Error(_) => None,
        response => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
    }
}

impl UpdaterUI {

    /// Launches the update checks enabled for startup, and shows the update dialog if any of them finds an update.
    ///
    /// The checks are polled from a timer, so this returns immediately instead of blocking the startup.
    ///
    /// # Arguments
    ///
    /// * `app_ui` - The main UI, used as parent for the polling timer and the update dialog.
    pub unsafe fn new_with_precheck(app_ui: &Rc<AppUI>) {
        let mut precheck = Precheck::default();

        if !cfg!(target_os = "linux") && settings_bool(CHECK_UPDATES_ON_START) {
            precheck.receiver_program = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckUpdates));
        }

        if settings_bool(CHECK_SCHEMA_UPDATES_ON_START) {
            precheck.receiver_schema = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckSchemaUpdates));
        }

        if settings_bool(CHECK_LUA_AUTOGEN_UPDATES_ON_START) {
            precheck.receiver_twautogen = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckLuaAutogenUpdates));
        }

        if settings_bool(CHECK_OLD_AK_UPDATES_ON_START) {
            precheck.receiver_old_ak = Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckEmpireAndNapoleonAKUpdates));
        }

        if precheck.is_finished() {
            return;
        }

        let timer = QTimer::new_1a(app_ui.main_window());
        timer.set_interval(PRECHECK_POLL_INTERVAL_MS);

        // The slot is a child of the timer, so the timer is alive whenever the slot runs.
        let timer_ptr = timer.as_ptr();
        let precheck = RefCell::new(precheck);
        let slot = SlotNoArgs::new(&timer, clone!(
            app_ui => move || {
                let (program, schema, twautogen, old_ak) = {
                    let mut precheck = precheck.borrow_mut();
                    if !precheck.poll() {
                        return;
                    }

                    timer_ptr.stop();
                    if !precheck.update_available() {
                        timer_ptr.delete_later();
                        return;
                    }

                    (precheck.program.take(), precheck.schema.take(), precheck.twautogen.take(), precheck.old_ak.take())
                };

                if let Err(error) = Self::new(&app_ui, program, schema, twautogen, old_ak) {
                    warn!("Failed to open the updater dialog: {error}");
                }

                // Deleting the timer also deletes this slot, so it must be the last thing done here.
                timer_ptr.delete_later();
            }
        ));

        timer.timeout().connect(&slot);
        timer.start_0a();
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
        info_label.set_text(&qtre("updater_info", &[&changelog_path.to_string_lossy(), &settings_string(UPDATE_CHANNEL)]));
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

        // On Linux, program updates are managed by the package manager or Flatpak.
        if cfg!(target_os = "linux") {
            update_program_label.set_visible(false);
            update_program_button.set_visible(false);
        }

        // Show the dialog before checking for updates.
        main_widget.static_downcast::<QDialog>().set_window_title(&qtr("updater_title"));
        main_widget.static_downcast::<QDialog>().show();

        let receiver_program = if !cfg!(target_os = "linux") {
            Some(CENTRAL_COMMAND.read().unwrap().send(Command::CheckUpdates))
        } else {
            None
        };
        let receiver_schemas = CENTRAL_COMMAND.read().unwrap().send(Command::CheckSchemaUpdates);
        let receiver_twautogen = CENTRAL_COMMAND.read().unwrap().send(Command::CheckLuaAutogenUpdates);
        let receiver_old_ak = CENTRAL_COMMAND.read().unwrap().send(Command::CheckEmpireAndNapoleonAKUpdates);

        // Apply prechecks immediately for any that were already resolved.
        let mut pending_program = if cfg!(target_os = "linux") {
            false
        } else {
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
                    false
                }
                None => true,
            }
        };

        let mut pending_schema = match precheck_schema {
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
                false
            }
            None => true,
        };

        let mut pending_twautogen = match precheck_twautogen {
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
                false
            }
            None => true,
        };

        let mut pending_old_ak = match precheck_old_ak {
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
                false
            }
            None => true,
        };

        // Poll all pending receivers concurrently so no check blocks the others.
        if pending_program || pending_schema || pending_twautogen || pending_old_ak {
            let event_loop = QEventLoop::new_0a();

            while pending_program || pending_schema || pending_twautogen || pending_old_ak {
                if pending_program {
                    if let Some(ref receiver_program) = receiver_program {
                        match receiver_program.try_recv() {
                            Ok(response) => {
                                pending_program = false;
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
                            }
                            Err(error) => if error.is_disconnected() {
                                warn!("Update refresh (program) skipped: background channel disconnected.");
                                pending_program = false;
                                update_program_button.set_text(&qtr("updater_update_program_no_updates"));
                            }
                        }
                    }
                }

                if pending_schema {
                    match receiver_schemas.try_recv() {
                        Ok(response) => {
                            pending_schema = false;
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
                        }
                        Err(error) => if error.is_disconnected() {
                            warn!("Update refresh (schema) skipped: background channel disconnected.");
                            pending_schema = false;
                            update_schemas_button.set_text(&qtr("updater_update_schemas_no_updates"));
                        }
                    }
                }

                if pending_twautogen {
                    match receiver_twautogen.try_recv() {
                        Ok(response) => {
                            pending_twautogen = false;
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
                        }
                        Err(error) => if error.is_disconnected() {
                            warn!("Update refresh (TW autogen) skipped: background channel disconnected.");
                            pending_twautogen = false;
                            update_twautogen_button.set_text(&qtr("updater_update_twautogen_no_updates"));
                        }
                    }
                }

                if pending_old_ak {
                    match receiver_old_ak.try_recv() {
                        Ok(response) => {
                            pending_old_ak = false;
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
                        }
                        Err(error) => if error.is_disconnected() {
                            warn!("Update refresh (Empire/Napoleon AK) skipped: background channel disconnected.");
                            pending_old_ak = false;
                            update_old_ak_button.set_text(&qtr("updater_update_old_ak_no_updates"));
                        }
                    }
                }

                event_loop.process_events();
            }
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
    match &*settings_string(UPDATE_CHANNEL) {
        BETA => UpdateChannel::Beta,
        _ => UpdateChannel::Stable,
    }
}

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

use rpfm_ipc::api::RpcResponse;
use rpfm_ipc::api::updates::{CheckUpdate, UpdateComponent, UpdateStatus};
use rpfm_ipc::settings_keys::*;

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

/// An update check. Its receiver is dropped once the check answers or disconnects.
struct UpdateCheck {
    component: UpdateComponent,
    receiver: Option<Receiver<RpcResponse>>,
    status: Option<UpdateStatus>,
}

/// Update checks launched on start.
struct Precheck {
    program: Option<UpdateCheck>,
    schema: Option<UpdateCheck>,
    twautogen: Option<UpdateCheck>,
    old_ak: Option<UpdateCheck>,
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

impl UpdateCheck {

    /// Launches the update check of a component.
    fn start(component: UpdateComponent) -> Self {
        Self {
            component,
            receiver: Some(CENTRAL_COMMAND.read().unwrap().call(&CheckUpdate { component })),
            status: None,
        }
    }

    /// Returns the check of a component, launching it unless it was already done with the provided result.
    fn start_unless_done(component: UpdateComponent, status: Option<UpdateStatus>) -> Self {
        match status {
            Some(status) => Self { component, receiver: None, status: Some(status) },
            None => Self::start(component),
        }
    }

    /// Returns if the check is waiting for its answer.
    fn is_pending(&self) -> bool {
        self.receiver.is_some()
    }

    /// Checks for the answer once, without blocking.
    ///
    /// # Returns
    ///
    /// `true` if the check has finished, `false` otherwise.
    fn poll(&mut self) -> bool {
        let Some(receiver) = self.receiver.as_ref() else { return true };
        match receiver.try_recv() {
            Ok(response) => {
                self.status = match api_result(response) {
                    Ok(status) => Some(status),
                    Err(error) => {
                        warn!("Update check ({:?}) failed: {error}", self.component);
                        None
                    }
                };
                self.receiver = None;
            }
            Err(error) => if error.is_disconnected() {
                warn!("Update check ({:?}) skipped: background channel disconnected.", self.component);
                self.receiver = None;
            }
        }

        self.receiver.is_none()
    }

    /// Returns if the check found an update.
    fn update_available(&self) -> bool {
        self.status.as_ref().is_some_and(|status| status.available)
    }
}

impl Precheck {

    /// Returns the prechecks, in the order the dialog shows them.
    fn checks_mut(&mut self) -> [&mut Option<UpdateCheck>; 4] {
        [&mut self.program, &mut self.schema, &mut self.twautogen, &mut self.old_ak]
    }

    /// Checks all pending checks once, without blocking.
    ///
    /// # Returns
    ///
    /// `true` if all checks have finished, `false` otherwise.
    fn poll(&mut self) -> bool {
        let mut finished = true;
        for check in self.checks_mut().into_iter().flatten() {
            finished &= check.poll();
        }

        finished
    }

    /// Returns `true` if any of the finished checks found an update.
    fn update_available(&mut self) -> bool {
        self.checks_mut().into_iter().flatten().any(|check| check.update_available())
    }

    /// Takes the result of a precheck.
    fn take_status(check: &mut Option<UpdateCheck>) -> Option<UpdateStatus> {
        check.take().and_then(|check| check.status)
    }
}

/// Shows the result of an update check in its button, enabling it if there is an update.
///
/// # Arguments
///
/// * `button` - Button of the component.
/// * `texts` - Prefix of the keys of the button's texts.
/// * `status` - Result of the check. `None` if it failed.
unsafe fn show_status(button: &QPtr<QPushButton>, texts: &str, status: Option<&UpdateStatus>) {
    match status {
        Some(status) if status.available => {
            let key = format!("{texts}_available");
            match &status.version {
                Some(version) => button.set_text(&qtre(&key, &[version])),
                None => button.set_text(&qtr(&key)),
            }
            button.set_enabled(true);
        }
        _ => button.set_text(&qtr(&format!("{texts}_no_updates"))),
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
        let start_if = |enabled: bool, component| enabled.then(|| UpdateCheck::start(component));
        let mut precheck = Precheck {
            program: start_if(!cfg!(target_os = "linux") && settings_bool(CHECK_UPDATES_ON_START), UpdateComponent::Program),
            schema: start_if(settings_bool(CHECK_SCHEMA_UPDATES_ON_START), UpdateComponent::Schemas),
            twautogen: start_if(settings_bool(CHECK_LUA_AUTOGEN_UPDATES_ON_START), UpdateComponent::LuaAutogen),
            old_ak: start_if(settings_bool(CHECK_OLD_AK_UPDATES_ON_START), UpdateComponent::OldAssemblyKit),
        };

        if precheck.checks_mut().iter().all(|check| check.is_none()) {
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

                    (
                        Precheck::take_status(&mut precheck.program),
                        Precheck::take_status(&mut precheck.schema),
                        Precheck::take_status(&mut precheck.twautogen),
                        Precheck::take_status(&mut precheck.old_ak),
                    )
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

    pub unsafe fn new(app_ui: &Rc<AppUI>, precheck_program: Option<UpdateStatus>, precheck_schema: Option<UpdateStatus>, precheck_twautogen: Option<UpdateStatus>, precheck_old_ak: Option<UpdateStatus>) -> Result<()> {

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

        // Checks that weren't done before are launched now, and their buttons updated as they answer.
        let mut checks = vec![
            (UpdateCheck::start_unless_done(UpdateComponent::Schemas, precheck_schema), &update_schemas_button, "updater_update_schemas"),
            (UpdateCheck::start_unless_done(UpdateComponent::LuaAutogen, precheck_twautogen), &update_twautogen_button, "updater_update_twautogen"),
            (UpdateCheck::start_unless_done(UpdateComponent::OldAssemblyKit, precheck_old_ak), &update_old_ak_button, "updater_update_old_ak"),
        ];

        if !cfg!(target_os = "linux") {
            checks.push((UpdateCheck::start_unless_done(UpdateComponent::Program, precheck_program), &update_program_button, "updater_update_program"));
        }

        for (check, button, texts) in &checks {
            if !check.is_pending() {
                show_status(button, texts, check.status.as_ref());
            }
        }

        let event_loop = QEventLoop::new_0a();
        while checks.iter().any(|(check, _, _)| check.is_pending()) {
            for (check, button, texts) in &mut checks {
                if check.is_pending() && check.poll() {
                    show_status(button, texts, check.status.as_ref());
                }
            }

            event_loop.process_events();
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

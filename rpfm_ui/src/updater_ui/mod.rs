//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use anyhow::{anyhow, Result};
use itertools::Itertools;
use getset::*;
use self_update::{backends::github::ReleaseList, Download, get_target, cargo_crate_version, Move, update::Release};
use tempfile::Builder;

use std::env::current_exe;
use std::fs::{DirBuilder, File};
use std::rc::Rc;

use rpfm_lib::integrations::git::GitResponse;
use rpfm_lib::utils::files_from_subdir;

use rpfm_ui_common::locale::{qtr, qtre};
use rpfm_ui_common::PROGRAM_PATH;
use rpfm_ui_common::settings::*;
use rpfm_ui_common::utils::*;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::updater_ui::slots::UpdaterUISlots;

const UPDATE_EXTENSION: &str = "zip";
const REPO_OWNER: &str = "Frodo45127";
const REPO_NAME: &str = "rpfm";

const UPDATE_FOLDER_PREFIX: &str = "updates";

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

/// This enum controls the possible responses from the server when checking for RPFM updates.
#[derive(Debug)]
pub enum APIResponse {

    /// This means a beta update was found.
    NewBetaUpdate(String),

    /// This means a major stable update was found.
    NewStableUpdate(String),

    /// This means a minor stable update was found.
    NewUpdateHotfix(String),

    /// This means no update was found.
    NoUpdate,

    /// This means don't know if there was an update or not, because the version we got was invalid.
    UnknownVersion,
}

//---------------------------------------------------------------------------//
//                              UI functions
//---------------------------------------------------------------------------//

impl UpdaterUI {

    /// This function checks for updates, and if it find any update, it shows the update dialog.
    pub unsafe fn new_with_precheck(app_ui: &Rc<AppUI>) -> Result<()> {
        let mut update_available = false;

        let updates_for_program = if setting_bool("check_updates_on_start") {
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckUpdates);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
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

        let updates_for_schema = if setting_bool("check_schema_updates_on_start") {
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
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

        let updates_for_twautogen = if setting_bool("check_lua_autogen_updates_on_start") {
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckLuaAutogenUpdates);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
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

        let updates_for_old_ak = if setting_bool("check_old_ak_updates_on_start") {
            let receiver = CENTRAL_COMMAND.send_network(Command::CheckEmpireAndNapoleonAKUpdates);
            let response = CENTRAL_COMMAND.recv_try(&receiver);
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
        info_label.set_text(&qtre("updater_info", &[&changelog_path.to_string_lossy(), &setting_string("update_channel")]));
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
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckUpdates);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
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
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckSchemaUpdates);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
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
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckLuaAutogenUpdates);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
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
                let receiver = CENTRAL_COMMAND.send_network(Command::CheckEmpireAndNapoleonAKUpdates);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
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

//---------------------------------------------------------------------------//
//                              Backend functions
//---------------------------------------------------------------------------//

/// This function takes care of updating RPFM itself when a new version comes out.
pub fn update_main_program() -> Result<()> {
    let update_channel = update_channel();
    let last_release = last_release(update_channel)?;

    // Get the download for our architecture.
    let asset = last_release.asset_for(get_target(), None).ok_or_else(|| anyhow!("No download available for your architecture."))?;
    let mut tmp_path = std::env::current_exe().unwrap();
    tmp_path.pop();
    let tmp_dir = Builder::new()
        .prefix(UPDATE_FOLDER_PREFIX)
        .tempdir_in(tmp_path)?;

    DirBuilder::new().recursive(true).create(&tmp_dir)?;

    // Nested stuff, because this seems to have problems with creating his own files before using them.
    {
        let tmp_zip_path = tmp_dir.path().join(&asset.name);
        let tmp_zip = File::create(&tmp_zip_path)?;

        Download::from_url(&asset.download_url)
            .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse().unwrap())
            .download_to(&tmp_zip)?;

        // self_update extractor doesn't work. It fails on every-single-test I did. So we use another one.
        let tmp_zip = File::open(&tmp_zip_path)?;
        zip_extract::extract(tmp_zip, tmp_dir.path(), true).map_err(|_| anyhow!("There was an error while extracting the update. This means either I uploaded a broken file, or your download was incomplete. In any case, no changes have been done so… try again later."))?;
    }

    let mut dest_base_path = current_exe()?;
    dest_base_path.pop();

    for updated_file in &files_from_subdir(tmp_dir.path(), true)? {

        // Ignore the downloaded ZIP.
        if let Some(extension) = updated_file.extension() {
            if let Some(extension) = extension.to_str() {
                if extension == UPDATE_EXTENSION {
                    continue;
                }
            }
        }

        let mut tmp_file = updated_file.to_path_buf();
        tmp_file.set_file_name(&format!("{}_replacement_tmp", updated_file.file_name().unwrap().to_str().unwrap()));

        // Fix for files in folders: we have to get the destination path with the folders included.
        let tmp_file_relative = updated_file.strip_prefix(tmp_dir.path()).unwrap();
        let dest_file = dest_base_path.join(tmp_file_relative);

        // Make sure the destination folder actually exists, or this will fail.
        let mut dest_folder = dest_base_path.join(tmp_file_relative);
        dest_folder.pop();
        DirBuilder::new().recursive(true).create(&dest_folder)?;

        Move::from_source(updated_file)
            .replace_using_temp(&tmp_file)
            .to_dest(&dest_file)?;
    }

    // Open the changelog because people don't read it.
    let changelog_path = dest_base_path.join(CHANGELOG_FILE);
    let _ = open::that(changelog_path);

    Ok(())
}

/// This function takes care of checking for new RPFM updates.
///
/// Also, this has a special behavior: If we have a beta version and we have the stable channel selected,
/// it'll pick the newest stable release, even if it's older than our beta. That way we can easily opt-out of betas.
pub fn check_updates_rpfm() -> Result<APIResponse> {
    let update_channel = update_channel();
    let last_release = last_release(update_channel)?;

    let current_version = cargo_crate_version!().split('.').map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();
    let last_version = &last_release.version.split('.').map(|x| x.parse::<i32>().unwrap_or(0)).collect::<Vec<i32>>();

    // Before doing anything else, check if we are going back to stable after a beta, and we are currently in a beta version.
    // In that case, return the last stable as valid.
    if let UpdateChannel::Stable = update_channel {
        if current_version[2] >= 99 {
            return Ok(APIResponse::NewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join("."))));
        }
    }

    // Get the version numbers from our version and from the latest released version, so we can compare them.
    let first = (last_version[0], current_version[0]);
    let second = (last_version[1], current_version[1]);
    let third = (last_version[2], current_version[2]);

    // If this is triggered, there has been a problem parsing the current/remote version.
    if first.0 == 0 && second.0 == 0 && third.0 == 0 || first.1 == 0 && second.1 == 0 && third.1 == 0 {
        Ok(APIResponse::UnknownVersion)
    }

    // If the current version is different than the last released version...
    else if last_version != &current_version {

        // If the latest released version is lesser than the current version...
        // No update. We are using a newer build than the last build released (dev?).
        if first.0 < first.1 { Ok(APIResponse::NoUpdate) }

        // If the latest released version is greater than the current version...
        // New major update. No more checks needed.
        else if first.0 > first.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::NewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::NewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // If the latest released version the same than the current version, we check the second, then the third number.
        // No update. We are using a newer build than the last build released (dev?).
        else if second.0 < second.1 { Ok(APIResponse::NoUpdate) }

        // New major update. No more checks needed.
        else if second.0 > second.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::NewStableUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::NewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // We check the last number in the versions, and repeat. Scraping the barrel...
        // No update. We are using a newer build than the last build released (dev?).
        else if third.0 < third.1 { Ok(APIResponse::NoUpdate) }

        // If the latest released version only has the last number higher, is a hotfix.
        else if third.0 > third.1 {
            match update_channel {
                UpdateChannel::Stable => Ok(APIResponse::NewUpdateHotfix(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
                UpdateChannel::Beta => Ok(APIResponse::NewBetaUpdate(format!("v{}", last_version.iter().map(|x| x.to_string()).join(".")))),
            }
        }

        // This means both are the same, and the checks will never reach this place thanks to the parent if.
        else { unreachable!("check_updates") }
    }
    else {
        Ok(APIResponse::NoUpdate)
    }
}

/// This function returns the last release available, according to our update channel.
pub fn last_release(update_channel: UpdateChannel) -> Result<Release> {
    let releases = ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()?
        .fetch()?;

    match releases.iter().find(|release| {
        match update_channel {
            UpdateChannel::Stable => release.version.split('.').collect::<Vec<&str>>()[2].parse::<i32>().unwrap_or(0) < 99,
            UpdateChannel::Beta => true
        }
    }) {
        Some(last_release) => Ok(last_release.clone()),
        None => Err(anyhow!("Failed to get last release (should never happen)."))
    }
}

/// This function returns the currently selected update channel.
pub fn update_channel() -> UpdateChannel {
    match &*setting_string("update_channel") {
        BETA => UpdateChannel::Beta,
        _ => UpdateChannel::Stable,
    }
}

/// Implementation of ToString.
impl ToString for UpdateChannel {
    fn to_string(&self) -> String {
        match &self {
            UpdateChannel::Stable => STABLE.to_owned(),
            UpdateChannel::Beta => BETA.to_owned(),
        }
    }
}

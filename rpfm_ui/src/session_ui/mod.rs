//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the code for the Session Selection dialog.
//!
//! This dialog allows users to view active server sessions and select one to connect to.

use qt_widgets::QButtonGroup;
use qt_widgets::QDialog;
use qt_widgets::q_dialog_button_box::StandardButton;
use qt_widgets::QDialogButtonBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLabel;
use qt_widgets::QMainWindow;
use qt_widgets::QRadioButton;
use qt_widgets::QScrollArea;
use qt_widgets::QVBoxLayout;

use qt_core::QBox;
use qt_core::QString;

use anyhow::Result;
use getset::Getters;

use std::rc::Rc;

use rpfm_ipc::helpers::SessionInfo;

use rpfm_log::*;

use crate::communications::CURRENT_SESSION_ID;
use crate::utils::{qtr, qtre};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct holds all the widgets used in the Session Selection dialog.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct SessionUI {
    dialog: QBox<QDialog>,
    status_label: QBox<QLabel>,
    sessions_group_box: QBox<QGroupBox>,
    button_group: QBox<QButtonGroup>,
    _button_box: QBox<QDialogButtonBox>,
    current_session_id: Option<u64>,
}

/// The result of the session dialog.
#[derive(Debug, Clone)]
pub enum SessionDialogResult {
    /// User selected an existing session.
    SessionSelected(u64),
    /// User cancelled the dialog.
    Cancelled,
    /// User selected the current session (no action needed).
    CurrentSession,
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

impl SessionUI {

    /// Creates a new Session Selection dialog.
    ///
    /// # Arguments
    /// * `parent` - The parent window for the dialog.
    ///
    /// # Safety
    /// This function uses Qt FFI and must be called from the main thread.
    pub unsafe fn new(parent: &QBox<QMainWindow>) -> Result<Rc<Self>> {
        let current_session_id = CURRENT_SESSION_ID.read().unwrap().clone();

        let dialog = QDialog::new_1a(parent);
        dialog.set_window_title(&qtr("session_dialog_title"));
        dialog.set_modal(true);
        dialog.resize_2a(450, 300);

        let layout = QVBoxLayout::new_1a(&dialog);

        // Status label
        let status_label = QLabel::new();
        status_label.set_text(&qtr("session_dialog_loading"));
        layout.add_widget(&status_label);

        // Group box for radio buttons inside a scroll area
        let scroll_area = QScrollArea::new_0a();
        scroll_area.set_widget_resizable(true);

        let sessions_group_box = QGroupBox::new();
        sessions_group_box.set_title(&qtr("session_dialog_group_title"));

        let _group_layout = QVBoxLayout::new_1a(&sessions_group_box);

        // Button group to manage radio button exclusivity
        let button_group = QButtonGroup::new_1a(&dialog);

        scroll_area.set_widget(&sessions_group_box);
        layout.add_widget(&scroll_area);

        // Button box with OK/Cancel
        let button_box = QDialogButtonBox::new();
        button_box.add_button_standard_button(StandardButton::Ok);
        button_box.add_button_standard_button(StandardButton::Cancel);
        layout.add_widget(&button_box);

        // Connect buttons
        button_box.accepted().connect(dialog.slot_accept());
        button_box.rejected().connect(dialog.slot_reject());

        let ui = Rc::new(Self {
            dialog,
            status_label,
            sessions_group_box,
            button_group,
            _button_box: button_box,
            current_session_id,
        });

        Ok(ui)
    }

    /// Shows the dialog and returns the selected session, or None if cancelled.
    ///
    /// # Safety
    /// This function uses Qt FFI and must be called from the main thread.
    pub unsafe fn show(&self) -> SessionDialogResult {
        // Clear and load sessions before showing
        self.load_sessions();

        if self.dialog.exec() == 1 {
            // Get the checked button's ID (which is the session ID)
            let checked_id = self.button_group.checked_id();
            if checked_id >= 0 {
                let session_id = checked_id as u64;

                // Check if this is the current session
                if let Some(current_id) = self.current_session_id {
                    if session_id == current_id {
                        return SessionDialogResult::CurrentSession;
                    }
                }

                return SessionDialogResult::SessionSelected(session_id);
            }
            // No selection but OK clicked - treat as cancelled
            SessionDialogResult::Cancelled
        } else {
            SessionDialogResult::Cancelled
        }
    }

    /// Loads sessions from the server and populates the radio buttons.
    ///
    /// # Safety
    /// This function uses Qt FFI.
    pub unsafe fn load_sessions(&self) {
        self.status_label.set_text(&qtr("session_dialog_loading"));

        match Self::fetch_sessions() {
            Ok(sessions) => {
                if sessions.is_empty() {
                    self.status_label.set_text(&qtr("session_dialog_no_sessions"));
                } else {
                    self.status_label.set_text(&qtre("session_dialog_found_sessions", &[&sessions.len().to_string()]));
                    self.populate_radio_buttons(&sessions);
                }
            }
            Err(e) => {
                error!("Failed to fetch sessions: {}", e);
                self.status_label.set_text(&qtre("session_dialog_load_error", &[&e.to_string()]));
            }
        }
    }

    /// Fetches active sessions from the server's REST endpoint.
    fn fetch_sessions() -> Result<Vec<SessionInfo>> {
        let url = "http://127.0.0.1:45127/sessions";

        // Use a blocking HTTP client since we're in a Qt event context
        let response = reqwest::blocking::get(url)?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned status: {}", response.status());
        }

        let sessions: Vec<SessionInfo> = response.json()?;
        Ok(sessions)
    }

    /// Populates the dialog with radio buttons for each session.
    ///
    /// # Safety
    /// This function uses Qt FFI.
    unsafe fn populate_radio_buttons(&self, sessions: &[SessionInfo]) {

        // Get the group box's layout
        let layout = self.sessions_group_box.layout();

        // Clear any existing widgets from the layout
        while layout.count() > 0 {
            let item = layout.take_at(0);
            if !item.is_null() {
                let widget = item.widget();
                if !widget.is_null() {
                    widget.delete_later();
                }
            }
        }

        for session in sessions {
            let session_id = *session.session_id();

            // Build the label text with pack names and session info
            let pack_names = session.pack_names();
            let pack_info = if pack_names.is_empty() {
                "No pack open".to_string()
            } else {
                pack_names.join(", ")
            };

            let status = if *session.is_shutting_down() {
                "Shutting down"
            } else if *session.connection_count() > 0 {
                "Connected"
            } else {
                "Disconnected"
            };

            let timeout_text = match session.timeout_remaining_secs() {
                Some(0) => " - Expiring...".to_string(),
                Some(secs) => {
                    let mins = secs / 60;
                    let secs_rem = secs % 60;
                    if mins > 0 {
                        format!(" - {}m {}s remaining", mins, secs_rem)
                    } else {
                        format!(" - {}s remaining", secs_rem)
                    }
                }
                None => String::new(),
            };

            // Mark current session
            let current_marker = if self.current_session_id == Some(session_id) {
                " (current)"
            } else {
                ""
            };

            let label = format!(
                "Session {} - {} [{}{}]{}",
                session_id,
                pack_info,
                status,
                timeout_text,
                current_marker
            );

            let radio_button = QRadioButton::from_q_string(&QString::from_std_str(&label));

            // Use session_id as the button ID in the group
            self.button_group.add_button_2a(&radio_button, session_id as i32);

            layout.add_widget(&radio_button);

            // Pre-select the current session
            if self.current_session_id == Some(session_id) {
                radio_button.set_checked(true);
            }
        }

        // Add stretch at the end to push buttons to the top
        let layout_ptr = layout.dynamic_cast::<QVBoxLayout>();
        if !layout_ptr.is_null() {
            layout_ptr.add_stretch_1a(1);
        }
    }
}

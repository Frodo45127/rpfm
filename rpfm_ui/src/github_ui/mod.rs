//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! GitHub sign-in, used to submit translations to the Translation Hub.
//!
//! The server keeps the token: the UI only drives the device flow sign-in and shows which account is signed in.

use qt_widgets::QDialog;
use qt_widgets::QDialogButtonBox;
use qt_widgets::q_dialog_button_box::{ButtonRole, StandardButton};
use qt_widgets::QLabel;
use qt_widgets::QVBoxLayout;
use qt_widgets::QWidget;

use qt_gui::QDesktopServices;
use qt_gui::QGuiApplication;

use qt_core::AlignmentFlag;
use qt_core::QFlags;
use qt_core::QString;
use qt_core::QTimer;
use qt_core::QUrl;
use qt_core::SlotNoArgs;
use qt_core::TextInteractionFlag;

use cpp_core::{CastInto, Ptr};

use anyhow::{anyhow, Result};

use std::cell::RefCell;
use std::rc::Rc;

use rpfm_ipc::api::github::{GetGitHubAccount, GitHubSignInState, PollGitHubSignIn, SignOutOfGitHub, StartGitHubSignIn};

use crate::communications::call_api_async;
use crate::utils::{qtr, tr};

/// Sign in to GitHub with the device flow.
///
/// Copies the code to the clipboard, opens GitHub's page to enter it, and shows the code in a dialog
/// until the user approves the sign-in on GitHub or cancels.
///
/// # Arguments
///
/// * `parent` - Widget the dialog is shown over.
///
/// # Returns
///
/// The login of the signed-in account, or `None` if the user cancelled.
///
/// # Errors
///
/// Returns an error if the sign-in can't be started, the code expires, or the user rejects it on GitHub.
pub unsafe fn sign_in(parent: impl CastInto<Ptr<QWidget>>) -> Result<Option<String>> {
    let code = call_api_async(&StartGitHubSignIn {})?;
    QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(code.user_code()));
    open_url(code.verification_uri());

    let dialog = QDialog::new_1a(parent);
    dialog.set_window_title(&qtr("github_sign_in_title"));
    let layout = QVBoxLayout::new_1a(&dialog);

    let intro_label = QLabel::from_q_string_q_widget(&qtr("github_sign_in_intro"), &dialog);
    intro_label.set_word_wrap(true);
    layout.add_widget(&intro_label);

    let code_label = QLabel::from_q_string_q_widget(&QString::from_std_str(code.user_code()), &dialog);
    code_label.set_style_sheet(&QString::from_std_str("font-size: 28px; font-weight: bold; font-family: monospace; padding: 12px;"));
    code_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
    code_label.set_text_interaction_flags(QFlags::from(TextInteractionFlag::TextSelectableByMouse));
    layout.add_widget(&code_label);

    let status_label = QLabel::from_q_string_q_widget(&qtr("github_sign_in_waiting"), &dialog);
    status_label.set_word_wrap(true);
    layout.add_widget(&status_label);

    let button_box = QDialogButtonBox::from_q_widget(&dialog);
    let copy_button = button_box.add_button_q_string_button_role(&qtr("github_sign_in_copy"), ButtonRole::ActionRole);
    let open_button = button_box.add_button_q_string_button_role(&qtr("github_sign_in_open"), ButtonRole::ActionRole);
    button_box.add_button_standard_button(StandardButton::Cancel);
    layout.add_widget(&button_box);

    // Single-shot, restarted after each poll, so a slow poll can't overlap the next one.
    let timer = QTimer::new_1a(&dialog);
    timer.set_single_shot(true);
    timer.set_interval(poll_interval_ms(*code.interval()));

    let result: Rc<RefCell<Result<Option<String>>>> = Rc::new(RefCell::new(Ok(None)));
    let dialog_ptr = dialog.as_ptr();
    let timer_ptr = timer.as_ptr();
    let device_code = code.device_code().to_owned();
    let poll_result = result.clone();
    let poll_slot = SlotNoArgs::new(&dialog, move || {
        let state = call_api_async(&PollGitHubSignIn { device_code: device_code.to_owned() });

        // The user may have cancelled while the poll was running.
        if !dialog_ptr.is_visible() {
            return;
        }

        match state {
            Ok(GitHubSignInState::Pending) => timer_ptr.start_0a(),
            Ok(GitHubSignInState::SlowDown(interval)) => {
                timer_ptr.set_interval(poll_interval_ms(interval));
                timer_ptr.start_0a();
            },
            Ok(GitHubSignInState::SignedIn(login)) => {
                *poll_result.borrow_mut() = Ok(Some(login));
                dialog_ptr.accept();
            },
            Ok(GitHubSignInState::Expired) => {
                *poll_result.borrow_mut() = Err(anyhow!(tr("github_sign_in_expired")));
                dialog_ptr.reject();
            },
            Ok(GitHubSignInState::Denied) => {
                *poll_result.borrow_mut() = Err(anyhow!(tr("github_sign_in_denied")));
                dialog_ptr.reject();
            },
            Err(error) => {
                *poll_result.borrow_mut() = Err(error);
                dialog_ptr.reject();
            },
        }
    });

    let user_code = code.user_code().to_owned();
    let copy_slot = SlotNoArgs::new(&dialog, move || {
        QGuiApplication::clipboard().set_text_1a(&QString::from_std_str(&user_code));
    });

    let verification_uri = code.verification_uri().to_owned();
    let open_slot = SlotNoArgs::new(&dialog, move || {
        open_url(&verification_uri);
    });

    timer.timeout().connect(&poll_slot);
    copy_button.released().connect(&copy_slot);
    open_button.released().connect(&open_slot);
    button_box.rejected().connect(dialog.slot_reject());

    timer.start_0a();
    dialog.exec();
    timer.stop();

    result.replace(Ok(None))
}

/// Login of the GitHub account the user is signed in as.
///
/// # Returns
///
/// The login, or `None` if not signed in.
///
/// # Errors
///
/// Returns an error if the server can't read the stored sign-in, usually because the system's keyring isn't available.
pub fn account() -> Result<Option<String>> {
    call_api_async(&GetGitHubAccount {}).map(|account| account.login)
}

/// Sign out of GitHub.
///
/// # Errors
///
/// Returns an error if the server can't delete the stored sign-in.
pub fn sign_out() -> Result<()> {
    call_api_async(&SignOutOfGitHub {}).map(|_| ())
}

/// Open a URL in the user's browser.
unsafe fn open_url(url: &str) {
    QDesktopServices::open_url(&QUrl::new_1a(&QString::from_std_str(url)));
}

/// Polling interval for the timer, in milliseconds, from GitHub's interval in seconds.
fn poll_interval_ms(seconds: u64) -> i32 {
    (seconds.max(1) * 1000).min(i32::MAX as u64) as i32
}

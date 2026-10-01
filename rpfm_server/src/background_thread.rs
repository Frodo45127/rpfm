//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Per-session command loop, translating the legacy [`Command`] protocol into
//! [`SessionState`] operations. Version 2 API requests are handled by [`crate::api`].
//!
//! Each [`Session`] spawns one task running [`background_loop`]. The loop
//! pulls [`SessionMessage`]s off the session's mpsc channel,
//! runs the matching operation on the session's [`SessionState`], reading any
//! option it needs from the session's settings, and ships the result back as a [`Response`] over the per-request `reply_sender`.
//!
//! Running commands serially per session is what keeps state consistent
//! across many concurrent requests in the same session: a `SavePack`
//! followed by a `ClosePack` always sees the right Pack, even when the
//! WebSocket multiplexer is firing requests as fast as the client sends
//! them.
//!
//! Telemetry: each dispatched command is recorded via
//! [`rpfm_telemetry::record_action`] so usage counters reflect what the
//! session actually did.

use anyhow::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::sync::Arc;

use rpfm_ipc::messages::{Command, Response};
use rpfm_ipc::settings_keys::*;


use rpfm_telemetry::info;

use crate::comms::CentralCommand;
use crate::api;
use crate::session::{Session, SessionMessage};
use rpfm_ipc::settings::*;
use crate::state::{ExtractOptions, SessionState};

/// Extracts the variant name (e.g. `"NewPack"`) from a [`Command`] for telemetry.
///
/// Uses the `Debug` impl via a custom `fmt::Write` that captures only the leading
/// identifier, so we don't pay the cost of formatting any inner data.
fn command_name(cmd: &Command) -> String {
    struct NameOnly {
        out: String,
        done: bool,
    }

    impl std::fmt::Write for NameOnly {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            if self.done {
                return Ok(());
            }
            for c in s.chars() {
                if c.is_alphanumeric() || c == '_' {
                    self.out.push(c);
                } else {
                    self.done = true;
                    return Ok(());
                }
            }
            Ok(())
        }
    }

    let mut capture = NameOnly { out: String::new(), done: false };
    let _ = std::fmt::write(&mut capture, format_args!("{:?}", cmd));
    capture.out
}

/// The per-session command loop.
///
/// Receives legacy commands and version 2 requests from the session's mpsc
/// `receiver` and processes them serially against the session's
/// [`SessionState`]. Each message gets exactly one response through its reply sender.
///
/// One instance runs per [`Session`], spawned by [`Session::new`]. The loop
/// terminates when the session is dropped or [`Command::Exit`] is dispatched.
pub async fn background_loop(mut receiver: UnboundedReceiver<SessionMessage>, session: Arc<Session>) {
    let mut state = SessionState::new(session.clone());

    info!("Background Thread looping around…");
    while let Some(message) = receiver.recv().await {
        let (command, sender) = match message {
            SessionMessage::Command(command, sender) => (*command, sender),
            SessionMessage::Api(request, sender) => {
                rpfm_telemetry::record_action(&request.method);
                let settings = session.settings();
                let _ = sender.send(api::dispatch(&mut state, request, &settings, &|_| {}));
                continue;
            }
            SessionMessage::Job(job, request) => {
                let jobs = session.jobs().clone();
                if !jobs.start(job) {
                    continue;
                }

                rpfm_telemetry::record_action(&request.method);
                let settings = session.settings();
                let response = api::dispatch(&mut state, request, &settings, &|stage| jobs.set_stage(job, stage));
                jobs.finish(job, response.outcome);
                continue;
            }
        };

        // Record the action for telemetry, skipping lifecycle commands so we only
        // measure real user-facing work. Counters are dropped silently when disabled.
        match &command {
            Command::Exit => break,
            Command::ClientDisconnecting => {}
            command => rpfm_telemetry::record_action(&command_name(command)),
        }

        let settings = session.settings();
        dispatch(&mut state, command, &sender, settings).await;
    }
}

/// Runs a command on the session's state, and sends its response back.
async fn dispatch(state: &mut SessionState, command: Command, sender: &UnboundedSender<Response>, settings: Arc<Settings>) {
    let disable_uuid_regeneration = settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES);
    let extract_options = ExtractOptions {
        disable_uuid_regeneration,
        tsv_keys_first: settings.bool(TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV),
    };

    match command {

        // Handled by the loop.
        Command::Exit => {}

        // ClientDisconnecting is handled at the WebSocket level. If it reaches here, just acknowledge it.
        Command::ClientDisconnecting => success(sender),

        // Packs.

        // Cleaning is the last resort when saving fails, so it doesn't check the pack's type.
        Command::GetPackFileDataForTreeView(pack_key) => reply(sender, state.pack_tree_data(&pack_key), Response::ContainerInfoVecRFileInfo),

        // Files.
        Command::GetRFileInfo(pack_key, path) => reply(sender, state.file_info(&pack_key, &path), Response::OptionRFileInfo),
        Command::GetPackedFilesInfo(pack_key, paths) => reply(sender, state.files_info(&pack_key, &paths), Response::VecRFileInfo),
        Command::NewPackedFile(pack_key, path, new_file) => reply(sender, state.new_file(&pack_key, &path, new_file), done),
        Command::AddPackedFiles(pack_key, source_paths, destination_paths, paths_to_ignore) => {
            let result = state.add_files_from_disk(&pack_key, &source_paths, &destination_paths, &paths_to_ignore, settings.bool(INCLUDE_BASE_FOLDER_ON_ADD_FROM_FOLDER));
            reply(sender, result, |(added_paths, error)| Response::VecContainerPathOptionString(added_paths, error));
        }
        Command::AddPackedFilesFromPackFile(target_key, source_key, paths) => reply(sender, state.add_files_from_pack(&target_key, &source_key, &paths), Response::VecContainerPath),
        Command::AddPackedFilesFromPackFileToAnimpack(source_key, anim_pack_key, anim_pack_path, paths) => reply(sender, state.add_files_to_animpack(&source_key, &anim_pack_key, &anim_pack_path, &paths), Response::VecContainerPath),
        Command::AddPackedFilesFromAnimpack(anim_pack_key, dest_key, data_source, anim_pack_path, paths) => reply(sender, state.add_files_from_animpack(&anim_pack_key, &dest_key, data_source, &anim_pack_path, &paths), Response::VecContainerPath),
        Command::DeleteFromAnimpack(pack_key, anim_pack_path, paths) => reply(sender, state.delete_from_animpack(&pack_key, &anim_pack_path, &paths), done),
        Command::DeletePackedFiles(pack_key, paths) => reply(sender, state.delete_files(&pack_key, &paths), Response::VecContainerPath),
        Command::CopyPackedFiles(paths_by_pack) => {
            state.copy_files(&paths_by_pack, false);
            success(sender);
        }
        Command::CutPackedFiles(paths_by_pack) => {
            state.copy_files(&paths_by_pack, true);
            success(sender);
        }
        Command::PastePackedFiles(target_key, destination_path) => reply(sender, state.paste_files(&target_key, &destination_path), |(added, deleted)| Response::VecContainerPathBTreeMapStringVecContainerPath(added, deleted)),
        Command::DuplicatePackedFiles(pack_key, paths) => reply(sender, state.duplicate_files(&pack_key, &paths), Response::VecContainerPath),
        Command::ExtractPackedFiles(pack_key, paths_by_source, destination, as_tsv) => {
            let result = state.extract_files(&pack_key, &paths_by_source, &destination, as_tsv, extract_options);
            reply(sender, result, |paths| Response::StringVecPathBuf("files_extracted_success".to_owned(), paths));
        }
        Command::RenamePackedFiles(pack_key, renames) => reply(sender, state.rename_files(&pack_key, &renames), Response::VecContainerPathContainerPath),
        Command::FolderExists(pack_key, path) => reply(sender, state.folder_exists(&pack_key, &path), Response::Bool),
        Command::PackedFileExists(pack_key, path) => reply(sender, state.file_exists(&pack_key, &path), Response::Bool),
        Command::OpenPackedFileInExternalProgram(pack_key, data_source, path) => reply(sender, state.open_in_external_program(&pack_key, data_source, &path, extract_options), Response::PathBuf),
        Command::SavePackedFileFromExternalView(pack_key, path, external_path) => reply(sender, state.save_file_from_external(&pack_key, &path, &external_path), done),
        Command::CleanCache(pack_key, paths) => reply(sender, state.clean_cache(&pack_key, &paths, disable_uuid_regeneration), done),
        Command::SavePackedFilesToPackFileAndClean(pack_key, files, optimize) => {
            let result = state.save_files_and_optimize(&pack_key, files, optimize.then(|| settings.optimizer_options()));
            reply(sender, result, |(added, deleted)| Response::VecContainerPathVecContainerPath(added, deleted));
        }
        Command::GetRFilesFromAllSources(paths, lowercase_paths) => send(sender, Response::HashMapDataSourceHashMapStringRFile(state.files_from_all_sources(&paths, lowercase_paths))),
        Command::GetPackedFilesNamesStartingWitPathFromAllSources(path) => send(sender, Response::HashMapDataSourceHashSetContainerPath(state.file_paths_from_all_sources(&path))),
        Command::ImportDependenciesToOpenPackFile(pack_key, paths_by_source) => reply(sender, state.import_dependencies(&pack_key, &paths_by_source), |(added, not_added)| Response::VecContainerPathVecString(added, not_added)),

        // Tables.

        // Search.

        // Game and dependencies.

        // Schema.
        Command::FieldsProcessed(definition) => send(sender, Response::VecField(definition.fields_processed())),

        // Diagnostics and Lua.

        // Tools.

        // GitHub and the Translation Hub.

    }
}

/// Sends a response back to the client.
fn send(sender: &UnboundedSender<Response>, response: Response) {
    CentralCommand::send_back(sender, response);
}

/// Sends a [`Response::Success`] back to the client.
fn success(sender: &UnboundedSender<Response>) {
    send(sender, Response::Success);
}

/// Sends the result of an operation back to the client, wrapping its value with `wrap`, or as [`Response::Error`] if it failed.
fn reply<T>(sender: &UnboundedSender<Response>, result: Result<T>, wrap: impl FnOnce(T) -> Response) {
    let response = match result {
        Ok(value) => wrap(value),
        Err(error) => Response::Error(error.to_string()),
    };

    send(sender, response);
}

/// Response wrapper for operations that return nothing on success.
fn done<T>(_: T) -> Response {
    Response::Success
}


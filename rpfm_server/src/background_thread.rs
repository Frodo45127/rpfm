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
//! them. Commands that don't touch the session's state (update checks,
//! GitHub sign-in, …) run on the blocking thread pool instead, so they don't
//! hold the loop.
//!
//! Telemetry: each dispatched command is recorded via
//! [`rpfm_telemetry::record_action`] so usage counters reflect what the
//! session actually did.

use anyhow::{anyhow, Result};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::path::Path;
use std::sync::Arc;

use rpfm_ipc::messages::{Command, Response};
use rpfm_ipc::settings_keys::*;

use rpfm_lib::files::{Container, pack::PFHFlags, RFileDecoded};

use rpfm_telemetry::info;

use crate::comms::CentralCommand;
use crate::api;
use crate::session::{Session, SessionMessage};
use rpfm_ipc::settings::*;
use crate::state::{DecodedFile, ExtractOptions, MergeOutcome, SaveOptions, SessionState, plugin_scripts};
use crate::translation_hub::{self, SubmitOutcome};

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

    let save_options = SaveOptions {
        disable_uuid_regeneration,
        allow_editing_of_ca_packfiles: settings.bool(ALLOW_EDITING_OF_CA_PACKFILES),
        clean: false,
    };

    match command {

        // Handled by the loop.
        Command::Exit => {}

        // ClientDisconnecting is handled at the WebSocket level. If it reaches here, just acknowledge it.
        Command::ClientDisconnecting => success(sender),

        // Packs.
        Command::NewPack => send(sender, Response::String(state.new_pack(&settings))),
        Command::OpenPackFiles(paths) => reply(sender, state.open_packs(&paths, settings.bool(USE_LAZY_LOADING)), |(key, info)| Response::StringContainerInfo(key, info)),
        Command::LoadAllCAPackFiles => reply(sender, state.open_ca_packs(&settings), |(key, info)| Response::StringContainerInfo(key, info)),
        Command::ClosePack(pack_key) => reply(sender, state.close_pack(&pack_key), done),
        Command::CloseAllPacks => {
            state.close_all_packs();
            success(sender);
        }
        Command::ListOpenPacks => send(sender, Response::VecStringContainerInfo(state.open_packs_info())),
        Command::SavePack(pack_key) => reply(sender, state.save_pack(&pack_key, None, save_options), Response::ContainerInfo),
        Command::SavePackAs(pack_key, path) => reply(sender, state.save_pack(&pack_key, Some(&path), save_options), Response::ContainerInfo),

        // Cleaning is the last resort when saving fails, so it doesn't check the pack's type.
        Command::CleanAndSavePackAs(pack_key, path) => {
            let options = SaveOptions { allow_editing_of_ca_packfiles: true, clean: true, ..save_options };
            reply(sender, state.save_pack(&pack_key, Some(&path), options), Response::ContainerInfo);
        }
        Command::GetPackFileDataForTreeView(pack_key) => reply(sender, state.pack_tree_data(&pack_key), Response::ContainerInfoVecRFileInfo),
        Command::GetPackFilePath(pack_key) => reply(sender, state.pack_path(&pack_key), Response::PathBuf),
        Command::GetPackFileName(pack_key) => reply(sender, state.pack_name(&pack_key), Response::String),
        Command::SetPackFileType(pack_key, pack_type) => reply(sender, state.set_pack_file_type(&pack_key, pack_type), done),
        Command::ChangeIndexIncludesTimestamp(pack_key, enabled) => reply(sender, state.set_pack_flag(&pack_key, PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, enabled), done),
        Command::ChangeIndexIsEncrypted(pack_key, enabled) => reply(sender, state.set_pack_flag(&pack_key, PFHFlags::HAS_ENCRYPTED_INDEX, enabled), done),
        Command::ChangeDataIsEncrypted(pack_key, enabled) => reply(sender, state.set_pack_flag(&pack_key, PFHFlags::HAS_ENCRYPTED_DATA, enabled), done),
        Command::ChangeCompressionFormat(pack_key, format) => reply(sender, state.set_compression_format(&pack_key, format), Response::CompressionFormat),
        Command::GetDependencyPackFilesList(pack_key) => reply(sender, state.pack_dependencies(&pack_key), Response::VecBoolString),
        Command::SetDependencyPackFilesList(pack_key, dependencies) => reply(sender, state.set_pack_dependencies(&pack_key, dependencies), done),
        Command::GetPackSettings(pack_key) => reply(sender, state.pack_settings(&pack_key), Response::PackSettings),
        Command::SetPackSettings(pack_key, pack_settings) => reply(sender, state.set_pack_settings(&pack_key, pack_settings), done),
        Command::SetPackOperationalMode(pack_key, mode) => reply(sender, state.set_pack_operational_mode(&pack_key, mode), done),
        Command::GetPackOperationalMode(pack_key) => send(sender, Response::OperationalMode(state.pack_operational_mode(&pack_key))),
        Command::TriggerBackupAutosave(pack_key) => reply(sender, state.backup_autosave(&pack_key, &settings, disable_uuid_regeneration, settings.i32(AUTOSAVE_AMOUNT) as usize), done),
        Command::GetMissingDefinitions(pack_key) => reply(sender, state.export_missing_definitions(&pack_key), done),
        Command::OpenContainingFolder(pack_key) => reply(sender, state.open_containing_folder(&pack_key), done),

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
        Command::DecodePackedFile(pack_key, path, data_source) => {
            info!("Trying to decode a file. Path: {}. Data Source: {}", path, data_source);
            reply(sender, state.decode_file(&pack_key, &path, data_source, settings.bool(ENABLE_ESF_EDITOR)), decoded_file_response);
        }
        Command::SavePackedFileFromView(pack_key, path, decoded) => reply(sender, state.save_file_from_view(&pack_key, &path, *decoded), done),
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
        Command::GetPackedFileRawData(pack_key, path) => reply(sender, state.file_raw_data(&pack_key, &path, disable_uuid_regeneration), Response::VecU8),
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
        Command::MergeFiles(pack_key, paths, merged_path, delete_source_files, options) => reply(sender, state.merge_files(&pack_key, &paths, &merged_path, delete_source_files, &options), |outcome| match outcome {
            MergeOutcome::Merged(path) => Response::String(path),
            MergeOutcome::Conflicts(conflicts) => Response::MergeConflicts(conflicts),
        }),
        Command::UpdateTable(pack_key, path) => reply(sender, state.update_table(&pack_key, &path), |(old_version, new_version, deleted, added)| Response::I32I32VecStringVecString(old_version, new_version, deleted, added)),
        Command::ExportTSV(pack_key, internal_path, external_path, data_source) => reply(sender, state.export_tsv(&pack_key, &internal_path, &external_path, data_source, extract_options.tsv_keys_first), done),
        Command::ImportTSV(pack_key, internal_path, external_path) => reply(sender, state.import_tsv(&pack_key, &internal_path, &external_path), Response::RFileDecoded),
        Command::CascadeEdition(pack_key, table_name, definition, changes) => reply(sender, state.cascade_edition(&pack_key, &table_name, &definition, &changes), |(paths, info)| Response::VecContainerPathVecRFileInfo(paths, info)),
        Command::GetTablesByTableName(pack_key, table_name) => reply(sender, state.table_paths_by_name(&pack_key, &table_name), Response::VecString),
        Command::AddKeysToKeyDeletes(pack_key, table_file_name, key_table_name, keys) => reply(sender, state.add_keys_to_key_deletes(&pack_key, &table_file_name, &key_table_name, &keys), Response::OptionContainerPath),
        Command::LocalArtSetIds(_pack_key) => send(sender, Response::HashSetString(state.column_values("campaign_character_arts_tables", "art_set_id", true, false))),
        Command::DependenciesArtSetIds => send(sender, Response::HashSetString(state.column_values("campaign_character_arts_tables", "art_set_id", false, true))),
        Command::DependenciesColumnValues(table_name, column_name) => send(sender, Response::HashSetString(state.column_values(&table_name, &column_name, true, true))),
        Command::GetTableListFromDependencyPackFile => send(sender, Response::VecString(state.dependency_table_names())),
        Command::GetTableVersionFromDependencyPackFile(table_name) => reply(sender, state.dependency_table_version(&table_name), Response::I32),
        Command::GetTableDefinitionFromDependencyPackFile(table_name) => reply(sender, state.dependency_table_definition(&table_name), Response::Definition),
        Command::GetTablesFromDependencies(table_name) => reply(sender, state.dependency_tables(&table_name), Response::VecRFile),

        // Search.

        // Game and dependencies.
        Command::GetGameSelected => send(sender, Response::String(state.game().key().to_owned())),
        Command::SetGameSelected(game_key, rebuild_dependencies) => {
            let result = state.set_game_selected(&game_key, rebuild_dependencies, &settings, disable_uuid_regeneration);
            let dependencies_rebuilt = matches!(result, Ok((_, Some(_))));
            reply(sender, result, |(format, info)| Response::CompressionFormatDependenciesInfo(format, info));

            // Decode the dependencies tables after answering, so the UI can do its own thing meanwhile.
            if dependencies_rebuilt {
                state.decode_dependency_tables();
            }
        }
        Command::GenerateDependenciesCache => reply(sender, state.generate_dependencies_cache(&settings, settings.bool(IGNORE_GAME_FILES_IN_AK)), Response::DependenciesInfo),
        Command::RebuildDependencies(only_parent_packs) => reply(sender, state.rebuild_dependencies(only_parent_packs, &settings), Response::DependenciesInfo),
        Command::IsThereADependencyDatabase(include_asskit) => send(sender, Response::Bool(state.is_dependency_database_loaded(include_asskit))),

        // Schema.
        Command::GetCustomTableList => reply(sender, state.custom_table_names(), Response::VecString),
        Command::FieldsProcessed(definition) => send(sender, Response::VecField(definition.fields_processed())),

        // Diagnostics and Lua.
        Command::LuaHovers(source) => send(sender, Response::VecU64U64U64U64String(state.lua_hovers(&source, &settings))),

        // Tools.
        Command::GetPackTranslation(pack_key, src_lang, language) => reply(sender, state.pack_translation(&pack_key, &src_lang, &language), Response::PackTranslation),
        Command::GenerateVanillaTranslationSource(src_lang) => reply(sender, state.generate_vanilla_translation_source(&src_lang, &settings), Response::Bool),
        Command::BuildCeo(pack_key, akit_path, bob_exe_path) => reply(sender, state.build_ceo(&pack_key, Path::new(&akit_path), Path::new(&bob_exe_path)), done),
        Command::BuildCeoPost(pack_key, akit_path) => reply(sender, state.build_ceo_post(&pack_key, &akit_path), Response::VecContainerPath),
        Command::BuildCeoEntries(pack_key, entries) => reply(sender, state.build_ceo_entries(&pack_key, &entries), Response::VecContainerPath),
        Command::GetTraitCeos => send(sender, Response::VecStringTuples(state.trait_ceos())),
        Command::GetPluginScripts => reply(sender, plugin_scripts(), Response::VecString),
        Command::RunPluginScript(pack_key, script_path, paths) => reply(sender, state.run_plugin_script(&pack_key, &script_path, &paths, extract_options), |(paths, message)| Response::VecContainerPathOptionString(paths, message)),

        // GitHub and the Translation Hub.
        Command::GitHubSignInStart => spawn_reply(sender, translation_hub::sign_in_start, Response::GitHubDeviceCode),
        Command::GitHubSignInPoll(device_code) => spawn_reply(sender, move || translation_hub::sign_in_poll(&device_code), Response::GitHubSignInState),
        Command::GitHubAccount => spawn_reply(sender, translation_hub::account, Response::OptionString),
        Command::GitHubSignOut => spawn_reply(sender, translation_hub::sign_out, done),
        Command::SubmitTranslation(pack_name, src_lang, language) => {
            let game_key = state.game().key().to_owned();
            spawn_reply(sender, move || translation_hub::submit(&game_key, &pack_name, &src_lang, &language), |outcome| match outcome {
                SubmitOutcome::Submitted(result) => Response::SubmissionResult(result),
                SubmitOutcome::SignInRequired => Response::GitHubSignInRequired,
            });
        }

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

/// Runs a blocking job on the blocking thread pool, so it doesn't stall the async runtime.
async fn run_blocking<T: Send + 'static>(job: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(job).await
        .unwrap_or_else(|error| Err(anyhow!("The background task failed: {error}")))
}

/// Runs a job that doesn't need the session's state on the blocking thread pool, replying when it's done.
///
/// The loop moves on to the next command without waiting for the job.
fn spawn_reply<T, J, W>(sender: &UnboundedSender<Response>, job: J, wrap: W)
where
    T: Send + 'static,
    J: FnOnce() -> Result<T> + Send + 'static,
    W: FnOnce(T) -> Response + Send + 'static,
{
    let sender = sender.clone();
    tokio::spawn(async move {
        reply(&sender, run_blocking(job).await, wrap);
    });
}

/// Legacy response for a decoded file. Each file type has its own response variant.
fn decoded_file_response(decoded: DecodedFile) -> Response {
    let (decoded, info) = match decoded {
        DecodedFile::Decoded(decoded, info) => (decoded, info),
        DecodedFile::Notes(notes) => return Response::Text(notes),
        DecodedFile::Unsupported => return Response::Unknown,
        DecodedFile::External => return Response::Success,
    };

    match *decoded {
        RFileDecoded::AnimFragmentBattle(data) => Response::AnimFragmentBattleRFileInfo(data, info),
        RFileDecoded::AnimPack(data) => Response::AnimPackRFileInfo(data.files().values().map(From::from).collect(), info),
        RFileDecoded::AnimsTable(data) => Response::AnimsTableRFileInfo(data, info),
        RFileDecoded::Atlas(data) => Response::AtlasRFileInfo(data, info),
        RFileDecoded::Audio(data) => Response::AudioRFileInfo(data, info),
        RFileDecoded::DB(table) => Response::DBRFileInfo(table, info),
        RFileDecoded::ESF(data) => Response::ESFRFileInfo(data, info),
        RFileDecoded::GroupFormations(data) => Response::GroupFormationsRFileInfo(data, info),
        RFileDecoded::Image(image) => Response::ImageRFileInfo(image, info),
        RFileDecoded::Loc(table) => Response::LocRFileInfo(table, info),
        RFileDecoded::MatchedCombat(data) => Response::MatchedCombatRFileInfo(data, info),
        RFileDecoded::PortraitSettings(data) => Response::PortraitSettingsRFileInfo(data, info),
        RFileDecoded::RigidModel(data) => Response::RigidModelRFileInfo(data, info),
        RFileDecoded::Text(text) => Response::TextRFileInfo(text, info),
        RFileDecoded::UIC(uic) => Response::UICRFileInfo(uic, info),
        RFileDecoded::UnitVariant(data) => Response::UnitVariantRFileInfo(data, info),
        RFileDecoded::Video(data) => Response::VideoInfoRFileInfo(From::from(&data), info),
        RFileDecoded::VMD(data) => Response::VMDRFileInfo(data, info),
        RFileDecoded::WSModel(data) => Response::WSModelRFileInfo(data, info),
        RFileDecoded::Anim(_) |
        RFileDecoded::BMD(_) |
        RFileDecoded::BMDVegetation(_) |
        RFileDecoded::Dat(_) |
        RFileDecoded::Font(_) |
        RFileDecoded::HlslCompiled(_) |
        RFileDecoded::Pack(_) |
        RFileDecoded::SoundBank(_) |
        RFileDecoded::Unknown(_) => Response::Unknown,
    }
}

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Dispatcher of the version 2 API, translating [`RpcRequest`]s into [`SessionState`] operations.
//!
//! See [`rpfm_ipc::api`] for the methods and their types.

use serde_json::{Map, Value};

use rpfm_ipc::api::{ApiError, Done, Request, RpcRequest, RpcResponse};
use rpfm_ipc::api::files::{AddFilesFromDisk, CopyFiles, CreateFile, DeleteFiles, DuplicateFiles, ExtractFiles, ListFiles, RenameFiles};
use rpfm_ipc::api::packs::{ClosePack, CloseAllPacks, GetPackInfo, NewPack, OpenPack, OpenVanillaPacks, SavePack, UpdatePack};
use rpfm_ipc::api::session::{GenerateDependenciesCache, GetSessionStatus, RebuildDependencies, SetGame};
use rpfm_ipc::api::tables::{EditTable, GetTableDefinition, GetTableInfo, GetTableRows};
use rpfm_ipc::settings_keys::{ALLOW_EDITING_OF_CA_PACKFILES, DISABLE_UUID_REGENERATION_ON_DB_TABLES, IGNORE_GAME_FILES_IN_AK, INCLUDE_BASE_FOLDER_ON_ADD_FROM_FOLDER, TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV, USE_LAZY_LOADING};

use crate::settings::Settings;
use crate::state::{ExtractOptions, SaveOptions, SessionState};

/// Methods that run as jobs.
const JOB_METHODS: [&str; 3] = [SetGame::METHOD, GenerateDependenciesCache::METHOD, RebuildDependencies::METHOD];

/// Runs a request on the session's state.
///
/// # Arguments
///
/// * `state` - State of the session.
/// * `request` - The request to run.
/// * `settings` - Settings, for the options the request doesn't set.
/// * `report_stage` - Called by long operations with the step they're on.
///
/// # Returns
///
/// The response to the request.
pub fn dispatch(state: &mut SessionState, request: RpcRequest, settings: &Settings, report_stage: &dyn Fn(&str)) -> RpcResponse {
    let params = request.params;
    let result = match request.method.as_str() {
        GetSessionStatus::METHOD => call(params, |_: GetSessionStatus| Ok(state.session_status())),
        SetGame::METHOD => call(params, |request: SetGame| {
            report_stage("Loading the schema and the dependencies");
            let (_, dependencies_info) = state.set_game_selected(&request.game, request.rebuild_dependencies, settings, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES))?;
            if dependencies_info.is_some() {
                report_stage("Decoding the tables of the dependencies");
                state.decode_dependency_tables();
            }

            Ok(state.session_status())
        }),
        GenerateDependenciesCache::METHOD => call(params, |request: GenerateDependenciesCache| {
            report_stage("Generating the dependencies cache");
            let ignore_game_files = request.ignore_game_files_in_assembly_kit.unwrap_or_else(|| settings.bool(IGNORE_GAME_FILES_IN_AK));
            state.generate_dependencies_cache(settings, ignore_game_files)?;
            Ok(state.session_status())
        }),
        RebuildDependencies::METHOD => call(params, |request: RebuildDependencies| {
            report_stage("Rebuilding the dependencies");
            state.rebuild_dependencies(request.only_parent_packs, settings)?;
            Ok(state.session_status())
        }),

        GetPackInfo::METHOD => call(params, |request: GetPackInfo| state.pack_details(&request.pack)),
        NewPack::METHOD => call(params, |_: NewPack| {
            let key = state.new_pack(settings);
            state.pack_summary(&key)
        }),
        OpenPack::METHOD => call(params, |request: OpenPack| {
            let lazy_loading = request.lazy_loading.unwrap_or_else(|| settings.bool(USE_LAZY_LOADING));
            let (key, _) = state.open_packs(&request.paths, lazy_loading)?;
            state.pack_summary(&key)
        }),
        OpenVanillaPacks::METHOD => call(params, |_: OpenVanillaPacks| {
            let (key, _) = state.open_ca_packs(settings)?;
            state.pack_summary(&key)
        }),
        ClosePack::METHOD => call(params, |request: ClosePack| state.close_pack(&request.pack).map(|_| Done {})),
        CloseAllPacks::METHOD => call(params, |_: CloseAllPacks| {
            state.close_all_packs();
            Ok(Done {})
        }),
        SavePack::METHOD => call(params, |request: SavePack| {
            let options = SaveOptions {
                disable_uuid_regeneration: request.disable_uuid_regeneration.unwrap_or_else(|| settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES)),
                allow_editing_of_ca_packfiles: request.allow_editing_ca_packs.unwrap_or_else(|| settings.bool(ALLOW_EDITING_OF_CA_PACKFILES)),
                clean: request.clean,
            };

            state.save_pack(&request.pack, request.path.as_deref(), options)?;
            state.pack_summary(&request.pack)
        }),
        UpdatePack::METHOD => call(params, |request: UpdatePack| state.update_pack(&request)),

        ListFiles::METHOD => call(params, |request: ListFiles| state.list_files(&request)),
        CreateFile::METHOD => call(params, |request: CreateFile| state.create_file(&request)),
        AddFilesFromDisk::METHOD => call(params, |request: AddFilesFromDisk| {
            let include_base_folder = request.include_base_folder.unwrap_or_else(|| settings.bool(INCLUDE_BASE_FOLDER_ON_ADD_FROM_FOLDER));
            state.add_disk_files(&request, include_base_folder)
        }),
        CopyFiles::METHOD => call(params, |request: CopyFiles| state.copy_files_to_pack(&request)),
        DeleteFiles::METHOD => call(params, |request: DeleteFiles| state.delete_paths(&request)),
        RenameFiles::METHOD => call(params, |request: RenameFiles| state.rename_paths(&request)),
        DuplicateFiles::METHOD => call(params, |request: DuplicateFiles| state.duplicate_paths(&request)),
        ExtractFiles::METHOD => call(params, |request: ExtractFiles| {
            let options = ExtractOptions {
                disable_uuid_regeneration: settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES),
                tsv_keys_first: request.tsv_keys_first.unwrap_or_else(|| settings.bool(TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV)),
            };

            state.extract_paths(&request, options)
        }),

        GetTableInfo::METHOD => call(params, |request: GetTableInfo| state.table_info(&request.file)),
        GetTableRows::METHOD => call(params, |request: GetTableRows| state.table_rows(&request)),
        EditTable::METHOD => call(params, |request: EditTable| state.edit_table(&request)),
        GetTableDefinition::METHOD => call(params, |request: GetTableDefinition| state.table_definition(&request)),
        method => Err(ApiError::MethodNotFound(method.to_owned())),
    };

    RpcResponse::new(request.id, result)
}

/// Returns if a method runs as a job. See [`Request::IS_JOB`].
pub fn is_job_method(method: &str) -> bool {
    JOB_METHODS.contains(&method)
}

/// Deserializes the params of a request.
pub fn parse_params<R: Request>(params: Value) -> Result<R, ApiError> {

    // Methods without params can be called without the params field, which arrives as null.
    let params = if params.is_null() { Value::Object(Map::new()) } else { params };
    serde_json::from_value::<R>(params).map_err(|error| ApiError::InvalidParams(error.to_string()))
}

/// Deserializes the params of a request, runs its operation, and serializes its response.
fn call<R: Request>(params: Value, operation: impl FnOnce(R) -> anyhow::Result<R::Response>) -> Result<Value, ApiError> {
    let request = parse_params::<R>(params)?;
    let response = operation(request).map_err(api_error)?;
    serde_json::to_value(response).map_err(|error| ApiError::Internal(error.to_string()))
}

/// Returns the [`ApiError`] an operation failed with, or wraps its message in [`ApiError::Internal`] if it's another error.
fn api_error(error: anyhow::Error) -> ApiError {
    error.downcast::<ApiError>().unwrap_or_else(|error| ApiError::Internal(error.to_string()))
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_methods_are_the_ones_marked_as_jobs() {
        assert!(SetGame::IS_JOB && GenerateDependenciesCache::IS_JOB && RebuildDependencies::IS_JOB);
        assert!(JOB_METHODS.iter().all(|method| is_job_method(method)));
        assert!(!is_job_method(GetSessionStatus::METHOD) && !GetSessionStatus::IS_JOB);
    }
}

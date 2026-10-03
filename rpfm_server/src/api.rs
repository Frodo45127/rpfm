//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Dispatcher of the API, translating [`RpcRequest`]s into [`SessionState`] operations.
//!
//! See [`rpfm_ipc::api`] for the methods and their types.

use serde_json::{Map, Value};

use std::cell::Cell;

use rpfm_ipc::api::{ApiError, Done, Request, RpcRequest, RpcResponse};
use rpfm_ipc::api::diagnostics::{GetDiagnosticsReport, IgnoreDiagnostics, ListDiagnostics, RunDiagnostics};
use rpfm_ipc::api::files::{
    AddFilesFromDisk, AddToAnimPack, CopyFiles, CreateFile, DeleteFiles, DeleteFromAnimPack, DuplicateFiles, ExtractFiles, ExtractFromAnimPack, ExternalFile, FilesFromAllSources, GetFilesFromAllSources, GetFilesInfo, GetViewData, OpenInExternalProgram, PasteFiles,
    SaveExternalFile, SaveFiles, ListAnimPack,
    ListFiles, ReadFile, RenameFiles, WriteFile,
};
use rpfm_ipc::api::search::{GetSearchReport, ListSearchMatches, ReplaceSearchMatches, RunSearch};
use rpfm_ipc::api::notes::{AddNote, DeleteNote, ListNotes, NoteList};
use rpfm_ipc::api::packs::{BackupPack, ClosePack, CloseAllPacks, GetPackInfo, GetPackSettings, NewPack, OpenPack, OpenVanillaPacks, SavePack, UpdatePack, UpdatePackSettings};
use rpfm_ipc::api::schema::{
    DeleteDefinition, GetMissingDefinitions, GetRawDefinitions, GetReferencingColumns, GetTablePatches, ImportPatches, ListSchemaTables, PatchColumn, RemovePatches, SetDefinition,
    UpdateSchemaFromAssemblyKit, UpdateSchemas,
};
use rpfm_ipc::api::github::{GetGitHubAccount, GitHubAccount, PollGitHubSignIn, SignOutOfGitHub, StartGitHubSignIn};
use rpfm_ipc::api::translations::{GenerateVanillaTexts, GetPackTranslation, ListTranslations, SubmitTranslation, TranslationSubmission, VanillaTextsAvailable};
use rpfm_ipc::api::session::{GenerateDependenciesCache, GetDependenciesInfo, GetDependencyTableData, GetSessionStatus, ListDependencyTables, RebuildDependencies, SetGame};
use rpfm_ipc::api::references::{FindDefinition, FindLoc, FindUsages, GetLocSource, GetReferenceValues, GetTableReferenceData, LocSourceLookup};
use rpfm_ipc::api::tables::{AddKeyDeletes, EditTable, ExportTsv, GetColumnValues, GetTableDefinition, GetTableInfo, GetTableRows, ImportTsv, MergeTables, RenameKey, UpgradeTable};
use rpfm_ipc::api::updates::{ApplyUpdate, CheckUpdate};
use rpfm_ipc::api::tools::{
    AddCeoEntries, AnimsBySkeleton, BuildCeo, ExportGltf, FinishStartpos, GenerateMissingLocs, GetLuaHovers, GetOptimizerOptions, GetStartposCampaigns,
    ImportCeo, InitMyMod, ListPluginScripts, ListTraitCeos, LiveExport, LuaHover, LuaHovers, LuaTestResults, OptimizePack, OptimizerOptionValues, PackMap,
    optimizer_option_values, PatchSiegeAi, PluginScripts, RunLuaTests, RunPluginScript, SetVideoFormat, StartStartpos, TraitCeos, UpdateAnimIds,
};
use rpfm_ipc::settings_keys::{ALLOW_EDITING_OF_CA_PACKFILES, AUTOSAVE_AMOUNT, ENABLE_ESF_EDITOR, MYMOD_BASE_PATH, DISABLE_UUID_REGENERATION_ON_DB_TABLES, IGNORE_GAME_FILES_IN_AK, INCLUDE_BASE_FOLDER_ON_ADD_FROM_FOLDER, TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV, USE_LAZY_LOADING};

use rpfm_lib::schema::{SCHEMA_BRANCH, SCHEMA_REMOTE, SCHEMA_REPO};

use rpfm_ipc::settings::{schemas_path, Settings};
use crate::state::{ExtractOptions, SaveOptions, SessionState, optimizer_options_with, plugin_scripts};
use crate::translation_hub::{self, SubmitOutcome};
use crate::updater::{apply_component, check_component, git_update_repo};

/// Methods that run as jobs.
const JOB_METHODS: [&str; 9] = [
    OptimizePack::METHOD,
    RunLuaTests::METHOD,
    SetGame::METHOD,
    GenerateDependenciesCache::METHOD,
    RebuildDependencies::METHOD,
    RunDiagnostics::METHOD,
    RunSearch::METHOD,
    UpdateSchemas::METHOD,
    UpdateSchemaFromAssemblyKit::METHOD,
];

/// Link between a running request and the job it runs in.
pub struct JobContext<'a> {

    /// Called by long operations with the step they're on.
    report_stage: &'a dyn Fn(&str),

    /// Called by long operations with the steps they did and their total. Returns `false` if they should stop
    /// to let the requests queued behind them run.
    report_progress: &'a (dyn Fn(usize, usize) -> bool + Sync),

    /// If the request stopped because `report_progress` told it to, so it has to run again later.
    yielded: Cell<bool>,
}

impl<'a> JobContext<'a> {

    /// Builds the context of a request.
    ///
    /// # Arguments
    ///
    /// * `report_stage` - Called by long operations with the step they're on.
    /// * `report_progress` - Called by long operations with the steps they did and their total. Returns `false`
    ///   if they should stop to let the requests queued behind them run.
    pub fn new(report_stage: &'a dyn Fn(&str), report_progress: &'a (dyn Fn(usize, usize) -> bool + Sync)) -> Self {
        Self { report_stage, report_progress, yielded: Cell::new(false) }
    }

    /// Reports the step a long operation is on.
    fn report_stage(&self, stage: &str) {
        (self.report_stage)(stage);
    }

    /// Returns if the request stopped early to let the requests queued behind it run.
    pub fn yielded(&self) -> bool {
        self.yielded.get()
    }
}

/// Runs a request on the session's state.
///
/// # Arguments
///
/// * `state` - State of the session.
/// * `request` - The request to run.
/// * `settings` - Settings, for the options the request doesn't set.
/// * `context` - The job the request runs in.
///
/// # Returns
///
/// The response to the request.
pub fn dispatch(state: &mut SessionState, request: RpcRequest, settings: &Settings, context: &JobContext) -> RpcResponse {
    let params = request.params;
    let result = match request.method.as_str() {
        GetSessionStatus::METHOD => call(params, |_: GetSessionStatus| Ok(state.session_status())),
        SetGame::METHOD => call(params, |request: SetGame| {
            context.report_stage("Loading the schema and the dependencies");
            let (_, dependencies_info) = state.set_game_selected(&request.game, request.rebuild_dependencies, settings, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES))?;
            if dependencies_info.is_some() {
                context.report_stage("Decoding the tables of the dependencies");
                state.decode_dependency_tables();
            }

            Ok(state.session_status())
        }),
        GenerateDependenciesCache::METHOD => call(params, |request: GenerateDependenciesCache| {
            context.report_stage("Generating the dependencies cache");
            let ignore_game_files = request.ignore_game_files_in_assembly_kit.unwrap_or_else(|| settings.bool(IGNORE_GAME_FILES_IN_AK));
            state.generate_dependencies_cache(settings, ignore_game_files)?;
            Ok(state.session_status())
        }),
        RebuildDependencies::METHOD => call(params, |request: RebuildDependencies| {
            context.report_stage("Rebuilding the dependencies");
            state.rebuild_dependencies(request.only_parent_packs, settings)?;
            Ok(state.session_status())
        }),
        ListDependencyTables::METHOD => call(params, |_: ListDependencyTables| Ok(state.dependency_table_versions())),
        GetDependenciesInfo::METHOD => call(params, |_: GetDependenciesInfo| Ok(state.dependencies_info())),
        GetDependencyTableData::METHOD => call(params, |request: GetDependencyTableData| state.dependency_table_data(&request.table_name)),

        GetPackInfo::METHOD => call(params, |request: GetPackInfo| state.pack_details(&request.pack)),
        BackupPack::METHOD => call(params, |request: BackupPack| {
            let disable_uuid_regeneration = settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES);
            state.backup_autosave(&request.pack, settings, disable_uuid_regeneration, settings.i32(AUTOSAVE_AMOUNT) as usize).map(|_| Done {})
        }),
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
        GetPackSettings::METHOD => call(params, |request: GetPackSettings| state.pack_settings_values(&request.pack)),
        UpdatePackSettings::METHOD => call(params, |request: UpdatePackSettings| state.update_pack_settings(&request)),
        ListNotes::METHOD => call(params, |request: ListNotes| Ok(NoteList { notes: state.note_entries(&request.pack, &request.path)? })),
        AddNote::METHOD => call(params, |request: AddNote| state.add_note_entry(&request)),
        DeleteNote::METHOD => call(params, |request: DeleteNote| state.delete_note(&request.pack, &request.path, request.id).map(|_| Done {})),

        ListFiles::METHOD => call(params, |request: ListFiles| state.list_files(&request)),
        CreateFile::METHOD => call(params, |request: CreateFile| state.create_file(&request)),
        AddFilesFromDisk::METHOD => call(params, |request: AddFilesFromDisk| {
            let include_base_folder = request.include_base_folder.unwrap_or_else(|| settings.bool(INCLUDE_BASE_FOLDER_ON_ADD_FROM_FOLDER));
            state.add_disk_files(&request, include_base_folder)
        }),
        CopyFiles::METHOD => call(params, |request: CopyFiles| state.copy_files_to_pack(&request)),
        ReadFile::METHOD => call(params, |request: ReadFile| state.read_file(&request, settings.bool(ENABLE_ESF_EDITOR), settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES))),
        WriteFile::METHOD => call(params, |request: WriteFile| state.write_file(&request).map(|_| Done {})),
        ListAnimPack::METHOD => call(params, |request: ListAnimPack| state.list_animpack(&request.file)),
        AddToAnimPack::METHOD => call(params, |request: AddToAnimPack| state.add_to_animpack(&request)),
        ExtractFromAnimPack::METHOD => call(params, |request: ExtractFromAnimPack| state.extract_from_animpack(&request)),
        DeleteFromAnimPack::METHOD => call(params, |request: DeleteFromAnimPack| state.delete_in_animpack(&request).map(|_| Done {})),
        DeleteFiles::METHOD => call(params, |request: DeleteFiles| state.delete_paths(&request)),
        RenameFiles::METHOD => call(params, |request: RenameFiles| state.rename_paths(&request)),
        DuplicateFiles::METHOD => call(params, |request: DuplicateFiles| state.duplicate_paths(&request)),
        GetFilesFromAllSources::METHOD => call(params, |request: GetFilesFromAllSources| Ok(FilesFromAllSources { files: state.files_from_all_sources(&request.paths, request.lowercase_paths) })),
        SaveFiles::METHOD => call(params, |request: SaveFiles| state.save_files_and_optimize(&request.pack, request.files, request.optimize.then(|| settings.optimizer_options()))),
        OpenInExternalProgram::METHOD => call(params, |request: OpenInExternalProgram| {
            let options = ExtractOptions {
                disable_uuid_regeneration: settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES),
                tsv_keys_first: settings.bool(TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV),
            };

            Ok(ExternalFile { path: state.open_in_external_program(&request.pack, &request.path, options)? })
        }),
        SaveExternalFile::METHOD => call(params, |request: SaveExternalFile| state.save_file_from_external(&request.pack, &request.path, &request.external_path).map(|_| Done {})),
        PasteFiles::METHOD => call(params, |request: PasteFiles| state.paste_files(&request)),
        GetFilesInfo::METHOD => call(params, |request: GetFilesInfo| state.files_info(&request)),
        GetViewData::METHOD => call(params, |request: GetViewData| state.view_data(&request.file, settings.bool(ENABLE_ESF_EDITOR))),
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
        GetColumnValues::METHOD => call(params, |request: GetColumnValues| state.column_values_page(&request)),
        MergeTables::METHOD => call(params, |request: MergeTables| state.merge_tables(&request)),
        UpgradeTable::METHOD => call(params, |request: UpgradeTable| state.upgrade_table(&request)),
        RenameKey::METHOD => call(params, |request: RenameKey| state.rename_key(&request)),
        AddKeyDeletes::METHOD => call(params, |request: AddKeyDeletes| state.add_key_deletes(&request)),
        ExportTsv::METHOD => call(params, |request: ExportTsv| {
            let keys_first = request.tsv_keys_first.unwrap_or_else(|| settings.bool(TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV));
            state.export_table_tsv(&request, keys_first).map(|_| Done {})
        }),
        ImportTsv::METHOD => call(params, |request: ImportTsv| state.import_table_tsv(&request)),

        ListSchemaTables::METHOD => call(params, |request: ListSchemaTables| state.schema_tables(&request.prefix)),
        PatchColumn::METHOD => call(params, |request: PatchColumn| state.patch_column(&request, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES)).map(|_| Done {})),
        RemovePatches::METHOD => call(params, |request: RemovePatches| state.remove_patches(&request, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES)).map(|_| Done {})),
        UpdateSchemas::METHOD => call(params, |_: UpdateSchemas| {
            context.report_stage("Downloading the schemas");
            git_update_repo(schemas_path, SCHEMA_REPO, SCHEMA_BRANCH, SCHEMA_REMOTE)?;

            context.report_stage("Reloading the schema");
            state.reload_schema(settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES));

            context.report_stage("Rebuilding the dependencies");
            state.rebuild_dependencies_after_schema_update(settings)?;
            Ok(state.session_status())
        }),
        UpdateSchemaFromAssemblyKit::METHOD => call(params, |request: UpdateSchemaFromAssemblyKit| {
            context.report_stage("Updating the schema from the Assembly Kit");
            let ignore_game_files = request.ignore_game_files.unwrap_or_else(|| settings.bool(IGNORE_GAME_FILES_IN_AK));
            state.update_schema_from_asskit(settings, ignore_game_files, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES))?;
            Ok(state.session_status())
        }),

        FindDefinition::METHOD => call(params, |request: FindDefinition| state.find_definition(request.pack.as_deref(), &request.table, &request.column, &request.value)),
        FindUsages::METHOD => call(params, |request: FindUsages| state.find_usages(&request)),
        GetTableReferenceData::METHOD => call(params, |request: GetTableReferenceData| state.table_reference_data(&request)),
        FindLoc::METHOD => call(params, |request: FindLoc| state.find_loc(request.pack.as_deref(), &request.key)),
        GetLocSource::METHOD => call(params, |request: GetLocSource| Ok(LocSourceLookup { source: state.loc_source(&request.key) })),

        RunDiagnostics::METHOD => call(params, |request: RunDiagnostics| {
            context.report_stage("Checking the open packs");
            let summary = state.run_diagnostics(&request, settings, context.report_progress);
            context.yielded.set(summary.is_none());
            summary.ok_or_else(|| anyhow::anyhow!("The check stopped to let other requests run."))
        }),
        ListDiagnostics::METHOD => call(params, |request: ListDiagnostics| state.list_diagnostics(&request)),
        IgnoreDiagnostics::METHOD => call(params, |request: IgnoreDiagnostics| state.ignore_diagnostics(&request)),
        GetDiagnosticsReport::METHOD => call(params, |_: GetDiagnosticsReport| state.diagnostics_report()),

        RunSearch::METHOD => call(params, |request: RunSearch| {
            context.report_stage("Searching");
            state.run_search(&request)
        }),
        ListSearchMatches::METHOD => call(params, |request: ListSearchMatches| state.list_search_matches(&request)),
        ReplaceSearchMatches::METHOD => call(params, |request: ReplaceSearchMatches| state.replace_search_matches(&request)),
        GetSearchReport::METHOD => call(params, |_: GetSearchReport| state.search_report()),
        GetOptimizerOptions::METHOD => call(params, |_: GetOptimizerOptions| Ok(OptimizerOptionValues { options: optimizer_option_values(&settings.optimizer_options()) })),
        OptimizePack::METHOD => call(params, |request: OptimizePack| {
            context.report_stage("Optimizing the pack");
            let options = optimizer_options_with(&settings.optimizer_options(), &request.options)?;
            state.optimize_pack_files(&request.pack, &options)
        }),
        PatchSiegeAi::METHOD => call(params, |request: PatchSiegeAi| state.patch_siege_ai_files(&request.pack)),
        PackMap::METHOD => call(params, |request: PackMap| {
            let tiles = request.tiles.into_iter().map(|tile| (tile.path, tile.folder)).collect();
            state.pack_map_files(&request.pack, request.tile_maps, tiles, settings.optimizer_options())
        }),
        GenerateMissingLocs::METHOD => call(params, |_: GenerateMissingLocs| state.generate_missing_locs()),
        UpdateAnimIds::METHOD => call(params, |request: UpdateAnimIds| state.update_anim_id_files(&request.pack, request.starting_id, request.id_offset)),
        AnimsBySkeleton::METHOD => call(params, |request: AnimsBySkeleton| Ok(state.anims_by_skeleton(&request.skeleton))),
        ExportGltf::METHOD => call(params, |request: ExportGltf| state.export_gltf(&request.file, &request.destination).map(|_| Done {})),
        SetVideoFormat::METHOD => call(params, |request: SetVideoFormat| {
            let format = serde_json::from_value(Value::String(request.format.clone()))
                .map_err(|_| ApiError::InvalidParams(format!("Unknown video format: {}. Valid ones: CaVp8, Ivf.", request.format)))?;
            state.set_video_format(&request.pack, &request.path, format).map(|_| Done {})
        }),
        LiveExport::METHOD => call(params, |request: LiveExport| {
            let keys_first = settings.bool(TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV);
            state.live_export(&request.pack, settings, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES), keys_first).map(|_| Done {})
        }),
        InitMyMod::METHOD => call(params, |request: InitMyMod| state.init_mymod(&settings.path_buf(MYMOD_BASE_PATH), &request)),
        RunLuaTests::METHOD => call(params, |request: RunLuaTests| {
            context.report_stage("Running the tests");
            let report = state.lua_run_tests(&request.code, request.campaign, settings)?;
            Ok(LuaTestResults { report: serde_json::to_value(report)? })
        }),
        GetStartposCampaigns::METHOD => call(params, |request: GetStartposCampaigns| state.startpos_campaigns(&request.pack)),
        StartStartpos::METHOD => call(params, |request: StartStartpos| state.start_startpos(&request, settings).map(|_| Done {})),
        FinishStartpos::METHOD => call(params, |request: FinishStartpos| state.finish_startpos(request.cancel, settings)),

        GetRawDefinitions::METHOD => call(params, |request: GetRawDefinitions| state.raw_definitions(&request.table_name, request.version)),
        SetDefinition::METHOD => call(params, |request: SetDefinition| state.set_definition(&request, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES)).map(|_| Done {})),
        DeleteDefinition::METHOD => call(params, |request: DeleteDefinition| state.delete_definition_and_save(&request, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES)).map(|_| Done {})),
        GetReferencingColumns::METHOD => call(params, |request: GetReferencingColumns| state.referencing_columns_of(&request)),
        GetTablePatches::METHOD => call(params, |request: GetTablePatches| state.table_patches(&request)),
        GetMissingDefinitions::METHOD => call(params, |request: GetMissingDefinitions| state.missing_definitions(&request.pack)),
        ImportPatches::METHOD => call(params, |request: ImportPatches| state.import_patches(&request, settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES)).map(|_| Done {})),
        GetReferenceValues::METHOD => call(params, |request: GetReferenceValues| state.reference_values(&request)),
        ListTranslations::METHOD => call(params, |request: ListTranslations| state.list_translations(&request)),
        GenerateVanillaTexts::METHOD => call(params, |request: GenerateVanillaTexts| {
            Ok(VanillaTextsAvailable { available: state.generate_vanilla_translation_source(&request.language, settings)? })
        }),
        ListPluginScripts::METHOD => call(params, |_: ListPluginScripts| Ok(PluginScripts { scripts: plugin_scripts()? })),
        RunPluginScript::METHOD => call(params, |request: RunPluginScript| {
            let options = ExtractOptions {
                disable_uuid_regeneration: settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES),
                tsv_keys_first: settings.bool(TABLES_USE_OLD_COLUMN_ORDER_FOR_TSV),
            };

            state.run_plugin_script(&request, options)
        }),
        GetLuaHovers::METHOD => call(params, |request: GetLuaHovers| {
            let hovers = state.lua_hovers(&request.source, settings).into_iter()
                .map(|(start_line, start_column, end_line, end_column, docs)| LuaHover { start_line, start_column, end_line, end_column, docs })
                .collect();

            Ok(LuaHovers { hovers })
        }),
        ListTraitCeos::METHOD => call(params, |_: ListTraitCeos| Ok(TraitCeos { traits: state.trait_ceos() })),
        AddCeoEntries::METHOD => call(params, |request: AddCeoEntries| state.build_ceo_entries(&request.pack, &request.entries)),
        BuildCeo::METHOD => call(params, |request: BuildCeo| state.build_ceo(&request.pack, &request.assembly_kit, &request.bob).map(|_| Done {})),
        ImportCeo::METHOD => call(params, |request: ImportCeo| state.build_ceo_post(&request.pack, &request.assembly_kit)),
        GetPackTranslation::METHOD => call(params, |request: GetPackTranslation| state.pack_translation(&request.pack, &request.source_language, &request.language)),
        SubmitTranslation::METHOD => call(params, |request: SubmitTranslation| {
            Ok(match translation_hub::submit(state.game().key(), &request.pack_name, &request.source_language, &request.language)? {
                SubmitOutcome::Submitted(result) => TranslationSubmission::Submitted { url: result.url().to_owned(), created: *result.created() },
                SubmitOutcome::SignInRequired => TranslationSubmission::SignInRequired,
            })
        }),

        method => Err(ApiError::MethodNotFound(method.to_owned())),
    };

    RpcResponse::new(request.id, result)
}

/// Returns if a method doesn't touch the session's state, so it can run without waiting for the session's other requests.
pub fn is_stateless_method(method: &str) -> bool {
    matches!(method, CheckUpdate::METHOD | ApplyUpdate::METHOD | StartGitHubSignIn::METHOD | PollGitHubSignIn::METHOD | GetGitHubAccount::METHOD | SignOutOfGitHub::METHOD)
}

/// Runs a request that doesn't touch the session's state. See [`is_stateless_method`].
///
/// # Arguments
///
/// * `request` - The request to run.
/// * `settings` - Settings, for the options the request doesn't set.
///
/// # Returns
///
/// The response to the request.
pub fn dispatch_stateless(request: RpcRequest, settings: &Settings) -> RpcResponse {
    let params = request.params;
    let result = match request.method.as_str() {
        CheckUpdate::METHOD => call(params, |request: CheckUpdate| check_component(request.component, settings)),
        ApplyUpdate::METHOD => call(params, |request: ApplyUpdate| apply_component(request.component, settings).map(|_| Done {})),
        StartGitHubSignIn::METHOD => call(params, |_: StartGitHubSignIn| translation_hub::sign_in_start()),
        PollGitHubSignIn::METHOD => call(params, |request: PollGitHubSignIn| translation_hub::sign_in_poll(&request.device_code)),
        GetGitHubAccount::METHOD => call(params, |_: GetGitHubAccount| Ok(GitHubAccount { login: translation_hub::account()? })),
        SignOutOfGitHub::METHOD => call(params, |_: SignOutOfGitHub| translation_hub::sign_out().map(|_| Done {})),
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
        const { assert!(SetGame::IS_JOB && GenerateDependenciesCache::IS_JOB && RebuildDependencies::IS_JOB && RunDiagnostics::IS_JOB && RunSearch::IS_JOB) };
        const { assert!(UpdateSchemas::IS_JOB && UpdateSchemaFromAssemblyKit::IS_JOB && OptimizePack::IS_JOB && RunLuaTests::IS_JOB) };
        assert!(JOB_METHODS.iter().all(|method| is_job_method(method)));
        assert!(!is_job_method(GetSessionStatus::METHOD) && !GetSessionStatus::IS_JOB);
    }
}

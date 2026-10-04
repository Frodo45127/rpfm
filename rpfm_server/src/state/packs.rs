//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Pack lifecycle and metadata operations: open, create, save, close, flags, settings and notes.

use anyhow::{anyhow, Result};
use rayon::prelude::*;
use uuid::Uuid;

use std::collections::BTreeMap;
use std::fs::DirBuilder;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::notes::{AddNote, NoteEntry};
use rpfm_ipc::api::files::{FilesInfo, GetFilesInfo};
use rpfm_ipc::api::packs::{PackDependency, PackDetails, PackSettingsValues, PackSummary, UpdatePack, UpdatePackSettings};
use rpfm_ipc::api::schema::MissingDefinitions;
use rpfm_ipc::helpers::ContainerInfo;
use rpfm_ipc::api::packs::OperationalMode;

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{Container, DecodeableExtraData, db::DB, FileType, pack::{Pack, PackSettings, PFHFlags}};
use rpfm_lib::games::{GameInfo, pfh_file_type::PFHFileType};
use rpfm_lib::notes::Note;
use rpfm_lib::utils::{files_from_subdir, files_in_folder_from_newest_to_oldest};

use rpfm_ipc::settings::{backup_autosave_path, Settings};
use rpfm_ipc::settings_keys::{AUTOSAVE_AMOUNT, DISABLE_UUID_REGENERATION_ON_DB_TABLES};

use rpfm_telemetry::warn;

use super::{SaveOptions, SessionState, decode_tables, encode_extra_data, loaded_schema, pack, pack_mut, pack_summary};

/// Stem used to seed names for newly created Packs (`new_pack.pack`, `new_pack_2.pack`, …).
const DEFAULT_PACK_STEM: &str = "new_pack";

/// Extension appended to [`DEFAULT_PACK_STEM`] when materialising a new Pack's filename.
const DEFAULT_PACK_EXT: &str = ".pack";

/// Size of the autosave folder, in bytes, over which a warning is logged after each autosave (25 GB).
const AUTOSAVE_FOLDER_SIZE_WARNING: u64 = 25 * 1024 * 1024 * 1024;

/// Name of the pack with all the CA packs of the selected game merged.
const CA_PACKS_NAME: &str = "CA PackFiles";

impl SessionState {

    /// Creates a new empty pack.
    ///
    /// # Arguments
    ///
    /// * `name` - File name of the pack. If `None`, a free `new_pack.pack`-like name is used.
    /// * `settings` - Settings, to find the game's install folder.
    ///
    /// # Returns
    ///
    /// The key of the new pack.
    ///
    /// # Errors
    ///
    /// Fails if the name is not a file name ending in `.pack`.
    pub fn new_pack(&mut self, name: Option<&str>, settings: &Settings) -> Result<String> {
        let name = match name {
            Some(name) if !name.ends_with(DEFAULT_PACK_EXT) || name.contains(['/', '\\']) => {
                return Err(ApiError::InvalidParams(format!("The name of the pack must be a file name ending in {DEFAULT_PACK_EXT}: {name}")).into());
            }
            Some(name) => name.to_owned(),
            None => derive_new_pack_name(&self.packs),
        };

        let pack_version = self.game.pfh_version_by_file_type(PFHFileType::Mod);
        let mut pack = Pack::new_with_name_and_version(&name, pack_version);

        if let Some(version_number) = self.game.game_version_number(&settings.path_buf(self.game.key())) {
            pack.set_game_version(version_number);
        }

        Ok(self.insert_pack(pack))
    }

    /// Opens one or more packs, merged into a single one.
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths of the packs to open. If there are more than one, the merged pack has no path, and is named after the first one.
    /// * `lazy_loading` - If file data should be read from disk only when needed.
    ///
    /// # Returns
    ///
    /// The key and info of the opened pack.
    ///
    /// # Errors
    ///
    /// Fails if any of the packs is already open, or if any of them can't be read.
    pub fn open_packs(&mut self, paths: &[PathBuf], lazy_loading: bool) -> Result<(String, ContainerInfo)> {
        if let Some(path) = paths.iter().find(|path| {
            let normalized = path.to_string_lossy().replace('\\', "/");
            self.packs.values().any(|pack| pack.disk_file_path() == normalized.as_str())
        }) {
            return Err(anyhow!("Pack '{}' is already open. Close it first if you want to reopen it.", path.display()));
        }

        let mut pack = Pack::read_and_merge(paths, &self.game, lazy_loading, false, false)?;
        decode_tables(&mut pack.files_by_type_mut(&[FileType::DB, FileType::Loc]), &self.schema);

        if paths.len() > 1 {
            let name = paths[0].file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
            pack.set_disk_file_path(name);
        }

        let info = ContainerInfo::from(&pack);
        let key = self.insert_pack(pack);
        Ok((key, info))
    }

    /// Opens all the CA packs of the selected game, merged into a single one.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, to find the game's install folder.
    ///
    /// # Returns
    ///
    /// The key and info of the opened pack.
    ///
    /// # Errors
    ///
    /// Fails if they're already open, or if they can't be read.
    pub fn open_ca_packs(&mut self, settings: &Settings) -> Result<(String, ContainerInfo)> {
        if self.packs.values().any(|pack| pack.disk_file_path() == CA_PACKS_NAME) {
            return Err(anyhow!("Pack '{}' is already open. Close it first if you want to reopen it.", CA_PACKS_NAME));
        }

        let mut pack = Pack::read_and_merge_ca_packs(&self.game, &settings.path_buf(self.game.key()))?;
        decode_tables(&mut pack.files_by_type_mut(&[FileType::DB, FileType::Loc]), &self.schema);
        pack.set_disk_file_path(CA_PACKS_NAME.to_owned());

        let info = ContainerInfo::from(&pack);
        let key = self.insert_pack(pack);
        Ok((key, info))
    }

    /// Closes a pack without saving it.
    pub fn close_pack(&mut self, pack_key: &str) -> Result<()> {
        self.packs.remove(pack_key).ok_or_else(|| ApiError::PackNotFound(pack_key.to_owned()))?;
        self.pack_modes.remove(pack_key);
        self.changed_packs.remove(pack_key);
        self.session.remove_pack(pack_key);
        Ok(())
    }

    /// Closes all open packs without saving them.
    pub fn close_all_packs(&mut self) {
        for pack_key in self.packs.keys() {
            self.session.remove_pack(pack_key);
        }

        self.packs.clear();
        self.pack_modes.clear();
        self.changed_packs.clear();
    }

    /// Saves a pack to disk.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to save.
    /// * `path` - Absolute path to save the pack to, creating its missing folders. If `None`, the pack is saved to its current path.
    /// * `options` - How to save the pack.
    ///
    /// # Returns
    ///
    /// The info of the saved pack.
    ///
    /// # Errors
    ///
    /// Fails if the pack is of a CA type and editing them is not allowed, if it has no path on disk yet, if the new path is not absolute, or if saving fails.
    pub fn save_pack(&mut self, pack_key: &str, path: Option<&Path>, options: SaveOptions) -> Result<ContainerInfo> {
        let pack = pack_mut(&mut self.packs, pack_key)?;

        let pack_type = *pack.header().pfh_file_type();
        if !options.allow_editing_of_ca_packfiles && pack_type != PFHFileType::Mod && pack_type != PFHFileType::Movie {
            return Err(anyhow!("Pack cannot be saved due to being of CA-Only type. Either change the Pack Type or enable \"Allow Edition of CA Packs\" in the settings."));
        }

        // New packs only have a bare name, which would silently save to the server's path.
        if path.is_none() && !Path::new(pack.disk_file_path()).is_absolute() {
            return Err(anyhow!("Pack '{}' has never been saved to disk. Use Save As to choose where to save it.", pack_key));
        }

        if let Some(path) = path {
            if !path.is_absolute() {
                return Err(ApiError::InvalidParams(format!("The path to save the pack to must be absolute: {}", path.display())).into());
            }

            if let Some(parent) = path.parent() {
                DirBuilder::new().recursive(true).create(parent)?;
            }
        }

        if options.clean {
            pack.clean_undecoded();
        }

        let extra_data = encode_extra_data(&self.game, pack.compression_format(), options.disable_uuid_regeneration);
        pack.save(path, &self.game, &extra_data)
            .map_err(|error| anyhow!("Error while trying to save the currently open PackFile: {}", error))?;
        self.session.set_pack_name(pack_key, &pack.disk_file_name());
        self.changed_packs.remove(pack_key);

        Ok(ContainerInfo::from(&*pack))
    }

    /// Changes properties of a pack. Only the properties set in the request are changed.
    ///
    /// # Errors
    ///
    /// Fails, without changing anything, if the request enables encryption on a pack whose version doesn't support it.
    pub fn update_pack(&mut self, request: &UpdatePack) -> Result<PackDetails> {
        let pack = pack_mut(&mut self.packs, &request.pack)?;

        let enables_encryption = request.index_encrypted == Some(true) || request.data_encrypted == Some(true);
        if enables_encryption && !pack.pfh_version().supports_encryption() {
            return Err(anyhow!("Encryption is not supported in {} Packs.", pack.pfh_version().value()));
        }

        if let Some(pack_type) = request.pack_type {
            pack.set_pfh_file_type(pack_type);
        }

        if let Some(compression) = request.compression {
            pack.set_compression_format(compression, &self.game);
        }

        let mut bitmask = pack.bitmask();
        for (flag, state) in [
            (PFHFlags::HAS_ENCRYPTED_INDEX, request.index_encrypted),
            (PFHFlags::HAS_ENCRYPTED_DATA, request.data_encrypted),
            (PFHFlags::HAS_INDEX_WITH_TIMESTAMPS, request.index_includes_timestamp),
        ] {
            if let Some(state) = state {
                bitmask.set(flag, state);
            }
        }
        pack.set_bitmask(bitmask);

        if let Some(ref dependencies) = request.dependencies {
            pack.set_dependencies(dependencies.iter().map(|dependency| (dependency.enabled, dependency.name.clone())).collect());
        }

        if let Some(ref mode) = request.operational_mode {
            self.pack_modes.insert(request.pack.clone(), mode.clone());
        }

        self.pack_details(&request.pack)
    }

    /// Returns the short description of an open pack.
    pub fn pack_summary(&self, pack_key: &str) -> Result<PackSummary> {
        Ok(pack_summary(pack_key, pack(&self.packs, pack_key)?))
    }

    /// Returns the info of files of a pack: the ones at the requested paths, or all of them.
    pub fn files_info(&self, request: &GetFilesInfo) -> Result<FilesInfo> {
        let pack = pack(&self.packs, &request.pack)?;
        let files = match request.paths {
            Some(ref paths) => paths.iter().filter_map(|path| pack.files().get(path)).map(From::from).collect(),
            None => pack.files().par_iter().map(|(_, file)| From::from(file)).collect(),
        };

        Ok(FilesInfo { files })
    }

    /// Returns the details of an open pack.
    pub fn pack_details(&self, pack_key: &str) -> Result<PackDetails> {
        let pack = pack(&self.packs, pack_key)?;
        let bitmask = pack.bitmask();

        Ok(PackDetails {
            summary: pack_summary(pack_key, pack),
            version: pack.pfh_version(),
            compression: pack.compression_format(),
            index_encrypted: bitmask.contains(PFHFlags::HAS_ENCRYPTED_INDEX),
            data_encrypted: bitmask.contains(PFHFlags::HAS_ENCRYPTED_DATA),
            index_includes_timestamp: bitmask.contains(PFHFlags::HAS_INDEX_WITH_TIMESTAMPS),
            extended_header: bitmask.contains(PFHFlags::HAS_EXTENDED_HEADER),
            timestamp: pack.internal_timestamp(),
            dependencies: pack.dependencies().iter()
                .map(|(enabled, name)| PackDependency { enabled: *enabled, name: name.clone() })
                .collect(),
            operational_mode: self.pack_operational_mode(pack_key),
        })
    }

    /// Returns the path of a pack on disk.
    pub fn pack_path(&self, pack_key: &str) -> Result<PathBuf> {
        Ok(PathBuf::from(pack(&self.packs, pack_key)?.disk_file_path()))
    }

    /// Returns the file name of a pack.
    pub fn pack_name(&self, pack_key: &str) -> Result<String> {
        Ok(pack(&self.packs, pack_key)?.disk_file_name())
    }

    /// Changes the compression format of a pack.
    ///
    /// # Returns
    ///
    /// The format actually set, which may differ from the requested one if the game doesn't support it.
    pub fn set_compression_format(&mut self, pack_key: &str, compression_format: CompressionFormat) -> Result<CompressionFormat> {
        Ok(pack_mut(&mut self.packs, pack_key)?.set_compression_format(compression_format, &self.game))
    }

    /// Returns the settings of a pack.
    pub fn pack_settings(&self, pack_key: &str) -> Result<PackSettings> {
        Ok(pack(&self.packs, pack_key)?.settings().clone())
    }

    /// Appends a line to the list of files a pack ignores in the diagnostics.
    pub fn add_pack_ignored_diagnostics_line(&mut self, pack_key: &str, line: String) -> Result<()> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let settings_text = pack.settings_mut().settings_text_mut();
        match settings_text.get_mut("diagnostics_files_to_ignore") {
            Some(diagnostics_ignored) => diagnostics_ignored.push_str(&line),
            None => { settings_text.insert("diagnostics_files_to_ignore".to_owned(), line); },
        }

        Ok(())
    }

    /// Returns the operational mode of a pack. Unknown packs are in normal mode.
    pub fn pack_operational_mode(&self, pack_key: &str) -> OperationalMode {
        self.pack_modes.get(pack_key).cloned().unwrap_or(OperationalMode::Normal)
    }

    /// Returns the settings of a pack.
    pub fn pack_settings_values(&self, pack_key: &str) -> Result<PackSettingsValues> {
        Ok(PackSettingsValues::from(pack(&self.packs, pack_key)?.settings()))
    }

    /// Changes settings of a pack. Only the keys set in the request are changed.
    ///
    /// # Returns
    ///
    /// All the settings of the pack after the change.
    pub fn update_pack_settings(&mut self, request: &UpdatePackSettings) -> Result<PackSettingsValues> {
        let settings = pack_mut(&mut self.packs, &request.pack)?.settings_mut();
        settings.settings_text_mut().extend(request.settings.text.clone());
        settings.settings_string_mut().extend(request.settings.string.clone());
        settings.settings_bool_mut().extend(request.settings.bool.clone());
        settings.settings_number_mut().extend(request.settings.number.clone());
        self.pack_settings_values(&request.pack)
    }

    /// Returns the notes of a pack for a path, with the notes of the folders containing it, as API entries.
    ///
    /// If the path is empty, all the notes of the pack are returned.
    pub fn note_entries(&self, pack_key: &str, path: &str) -> Result<Vec<NoteEntry>> {
        if path.is_empty() {
            let notes = pack(&self.packs, pack_key)?.notes().file_notes().values().flatten().map(note_entry).collect();
            return Ok(notes);
        }

        Ok(self.notes_for_path(pack_key, path)?.iter().map(note_entry).collect())
    }

    /// Attaches a note to a file or folder of a pack.
    ///
    /// # Returns
    ///
    /// The added note, with its ID.
    pub fn add_note_entry(&mut self, request: &AddNote) -> Result<NoteEntry> {
        let mut note = Note::default();
        note.set_path(request.path.clone());
        note.set_message(request.message.clone());
        note.set_url(request.url.clone());
        note.set_id(request.id.unwrap_or_default());
        Ok(note_entry(&self.add_note(&request.pack, note, request.id.is_some())?))
    }

    /// Returns the notes of a pack under a path.
    pub fn notes_for_path(&self, pack_key: &str, path: &str) -> Result<Vec<Note>> {
        Ok(pack(&self.packs, pack_key)?.notes().notes_by_path(path))
    }

    /// Adds a note to a pack.
    ///
    /// # Returns
    ///
    /// The added note, with its ID assigned.
    pub fn add_note(&mut self, pack_key: &str, note: Note, replace: bool) -> Result<Note> {
        Ok(pack_mut(&mut self.packs, pack_key)?.notes_mut().add_note(note, replace))
    }

    /// Deletes a note from a pack.
    pub fn delete_note(&mut self, pack_key: &str, path: &str, id: u64) -> Result<()> {
        pack_mut(&mut self.packs, pack_key)?.notes_mut().delete_note(path, id);
        Ok(())
    }

    /// Marks packs as changed since their last autosave.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the changed pack, or `None` if any open pack may have changed.
    pub fn mark_packs_changed(&mut self, pack_key: Option<&str>) {
        match pack_key {
            Some(pack_key) if self.packs.contains_key(pack_key) => { self.changed_packs.insert(pack_key.to_owned()); },
            Some(_) => {},
            None => self.changed_packs.extend(self.packs.keys().cloned()),
        }
    }

    /// Saves a backup copy of each pack changed since its last autosave in the autosave folder, keeping the newest ones.
    ///
    /// The packs are copied here, but saved from another thread, so the session can keep running requests meanwhile.
    /// Vanilla packs, packs with autosaves disabled and packs that are neither mods nor movies are skipped.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, for the game's install folder, the amount of autosaves to keep, and how to encode tables.
    pub fn autosave(&mut self, settings: &Settings) {
        let Ok(autosave_path) = backup_autosave_path() else {
            return;
        };

        let ca_paths = self.game.ca_packs_paths(&settings.path_buf(self.game.key()))
            .unwrap_or_default()
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();

        let packs = std::mem::take(&mut self.changed_packs).into_iter()
            .filter_map(|pack_key| self.packs.get(&pack_key))
            .filter(|pack| {
                let pack_type = pack.pfh_file_type();
                !pack.settings().setting_bool("disable_autosaves").unwrap_or(&true) &&
                    (pack_type == PFHFileType::Mod || pack_type == PFHFileType::Movie) &&
                    !ca_paths.contains(&pack.disk_file_path().replace('\\', "/"))
            })
            .cloned()
            .collect::<Vec<_>>();

        if packs.is_empty() {
            return;
        }

        let game = self.game.clone();
        let disable_uuid_regeneration = settings.bool(DISABLE_UUID_REGENERATION_ON_DB_TABLES);
        let autosave_amount = settings.i32(AUTOSAVE_AMOUNT).max(0) as usize;
        std::thread::spawn(move || {
            for mut pack in packs {
                let folder = autosave_path.join(pack.disk_file_name());
                if let Err(error) = save_autosave(&mut pack, &folder, &game, disable_uuid_regeneration, autosave_amount) {
                    warn!("Failed to autosave the pack {}: {error}", pack.disk_file_name());
                }
            }

            let size = files_from_subdir(&autosave_path, true).unwrap_or_default().iter()
                .filter_map(|path| path.metadata().ok())
                .map(|metadata| metadata.len())
                .sum::<u64>();

            if size > AUTOSAVE_FOLDER_SIZE_WARNING {
                warn!("The autosave folder is using {} GB. Consider lowering the amount of autosaves to keep, or deleting the ones you don't need.", size / 1024 / 1024 / 1024);
            }
        });
    }

    /// Returns the tables of a pack with rows that can't be decoded with the schema, sorted by path.
    ///
    /// This is slow, and only useful when a new patch lands and you want to know what tables need decoding.
    pub fn missing_definitions(&mut self, pack_key: &str) -> Result<MissingDefinitions> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let schema = loaded_schema(&self.schema)?;

        let mut extra_data = DecodeableExtraData::default();
        extra_data.set_schema(Some(schema));
        let extra_data = Some(extra_data);

        let mut files = pack.files_by_type_mut(&[FileType::DB]);
        files.sort_by_key(|file| file.path_in_container_raw().to_lowercase());

        let mut paths = vec![];
        for file in files {
            if file.decode(&extra_data, false, false).is_err() && file.load().is_ok() {
                if let Ok(raw_data) = file.cached() {
                    let mut reader = Cursor::new(raw_data);
                    if let Ok((_, _, _, entry_count)) = DB::read_header(&mut reader) {
                        if entry_count > 0 {
                            paths.push(file.path_in_container_raw().to_owned());
                        }
                    }
                }
            }
        }

        Ok(MissingDefinitions { paths })
    }

    /// Exports the files of a pack to the game's data folder, so they can be tested without saving the pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to export.
    /// * `settings` - Settings, to find the game's install folder.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when encoded.
    /// * `tsv_keys_first` - If exported TSV files use the old column order, with keys first.
    pub fn live_export(&mut self, pack_key: &str, settings: &Settings, disable_uuid_regeneration: bool, tsv_keys_first: bool) -> Result<()> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let game_path = settings.path_buf(self.game.key());
        pack.live_export(&self.game, &game_path, disable_uuid_regeneration, tsv_keys_first)?;
        Ok(())
    }

    /// Adds a pack to the open ones, in normal mode.
    ///
    /// # Returns
    ///
    /// The key of the pack.
    fn insert_pack(&mut self, pack: Pack) -> String {
        let key = Uuid::new_v4().to_string();

        self.session.set_pack_name(&key, &pack.disk_file_name());
        self.pack_modes.insert(key.clone(), OperationalMode::Normal);
        self.packs.insert(key.clone(), pack);
        key
    }
}

/// Returns a note as an API entry.
fn note_entry(note: &Note) -> NoteEntry {
    NoteEntry {
        id: *note.id(),
        path: note.path().to_owned(),
        message: note.message().to_owned(),
        url: note.url().clone(),
    }
}

/// Saves a pack as a new autosave in a folder, removing the oldest autosaves over the amount to keep.
///
/// # Arguments
///
/// * `pack` - Copy of the pack to save.
/// * `folder` - Folder with the autosaves of the pack.
/// * `game` - Game the pack is for.
/// * `disable_uuid_regeneration` - If tables keep their GUID when encoded.
/// * `autosave_amount` - Amount of autosaves to keep.
///
/// # Errors
///
/// If the folder can't be created, or the pack can't be saved.
fn save_autosave(pack: &mut Pack, folder: &Path, game: &GameInfo, disable_uuid_regeneration: bool, autosave_amount: usize) -> Result<()> {
    DirBuilder::new().recursive(true).create(folder)?;

    let date = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    let extra_data = encode_extra_data(game, pack.compression_format(), disable_uuid_regeneration);
    pack.save(Some(&folder.join(format!("{date}.pack"))), game, &extra_data)?;

    for file in files_in_folder_from_newest_to_oldest(folder)?.iter().skip(autosave_amount) {
        let _ = std::fs::remove_file(file);
    }

    Ok(())
}

/// Derives a pack name for new (unsaved) packs no open pack has, like "new_pack.pack", "new_pack_2.pack", etc.
fn derive_new_pack_name(packs: &BTreeMap<String, Pack>) -> String {
    let taken = |name: &str| packs.values().any(|pack| pack.disk_file_name() == name);
    let base = format!("{}{}", DEFAULT_PACK_STEM, DEFAULT_PACK_EXT);
    if !taken(&base) {
        return base;
    }

    (2..).map(|suffix| format!("{}_{}{}", DEFAULT_PACK_STEM, suffix, DEFAULT_PACK_EXT))
        .find(|candidate| !taken(candidate))
        .expect("an unbounded range always finds a free name")
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::session::Session;

    use super::*;

    #[tokio::test]
    async fn pack_keys_are_new_uuids() {
        let mut state = SessionState::new(Session::new(1, true));
        let settings = Settings::default();

        let first = state.new_pack(None, &settings).unwrap();
        state.close_pack(&first).unwrap();
        let second = state.new_pack(None, &settings).unwrap();

        assert!(Uuid::parse_str(&first).is_ok());
        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn new_packs_get_free_names_and_no_path() {
        let mut state = SessionState::new(Session::new(1, true));
        let settings = Settings::default();

        let named = state.new_pack(Some("my_mod.pack"), &settings).unwrap();
        let first = state.new_pack(None, &settings).unwrap();
        let second = state.new_pack(None, &settings).unwrap();

        assert_eq!(state.pack_summary(&named).unwrap().name, "my_mod.pack");
        assert_eq!(state.pack_summary(&first).unwrap().name, "new_pack.pack");
        assert_eq!(state.pack_summary(&second).unwrap().name, "new_pack_2.pack");
        assert_eq!(state.pack_summary(&second).unwrap().path, None);
    }

    #[tokio::test]
    async fn new_pack_names_must_be_pack_file_names() {
        let mut state = SessionState::new(Session::new(1, true));
        let settings = Settings::default();

        assert!(state.new_pack(Some("my_mod"), &settings).is_err());
        assert!(state.new_pack(Some("folder/my_mod.pack"), &settings).is_err());
    }

    #[tokio::test]
    async fn changes_mark_their_pack_or_all_open_ones() {
        let mut state = SessionState::new(Session::new(1, true));
        let settings = Settings::default();
        let first = state.new_pack(None, &settings).unwrap();
        let second = state.new_pack(None, &settings).unwrap();

        state.mark_packs_changed(Some(&first));
        state.mark_packs_changed(Some("not_open"));
        assert_eq!(state.changed_packs, BTreeSet::from([first.clone()]));

        state.mark_packs_changed(None);
        assert_eq!(state.changed_packs, BTreeSet::from([first.clone(), second.clone()]));

        state.close_pack(&first).unwrap();
        assert_eq!(state.changed_packs, BTreeSet::from([second]));
    }
}

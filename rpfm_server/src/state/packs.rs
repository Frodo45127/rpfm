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

use std::collections::BTreeMap;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::packs::{PackDependency, PackDetails, PackSummary, UpdatePack};
use rpfm_ipc::helpers::{ContainerInfo, RFileInfo};
use rpfm_ipc::messages::OperationalMode;

use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, db::DB, FileType, pack::{Pack, PackSettings, PFHFlags}};
use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::notes::Note;
use rpfm_lib::utils::files_in_folder_from_newest_to_oldest;

use crate::settings::{backup_autosave_path, Settings};

use super::{SaveOptions, SessionState, decode_tables, encode_extra_data, pack, pack_mut, pack_summary};

/// Stem used to seed names for newly created Packs (`new_pack.pack`, `new_pack_2.pack`, …).
const DEFAULT_PACK_STEM: &str = "new_pack";

/// Extension appended to [`DEFAULT_PACK_STEM`] when materialising a new Pack's filename.
const DEFAULT_PACK_EXT: &str = ".pack";

/// Key of the pack with all the CA packs of the selected game merged.
const CA_PACKS_KEY: &str = "CA PackFiles";

impl SessionState {

    /// Creates a new empty pack.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings, to find the game's install folder.
    ///
    /// # Returns
    ///
    /// The key of the new pack.
    pub fn new_pack(&mut self, settings: &Settings) -> String {
        let pack_version = self.game.pfh_version_by_file_type(PFHFileType::Mod);
        let key = derive_new_pack_name(&self.packs);
        let mut pack = Pack::new_with_name_and_version(&key, pack_version);

        if let Some(version_number) = self.game.game_version_number(&settings.path_buf(self.game.key())) {
            pack.set_game_version(version_number);
        }

        self.insert_pack(key.clone(), pack);
        key
    }

    /// Opens one or more packs, merged into a single one.
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths of the packs to open. The first one gives the merged pack its key.
    /// * `lazy_loading` - If file data should be read from disk only when needed.
    ///
    /// # Returns
    ///
    /// The key and info of the opened pack.
    ///
    /// # Errors
    ///
    /// Fails if the first pack is already open, or if any of them can't be read.
    pub fn open_packs(&mut self, paths: &[PathBuf], lazy_loading: bool) -> Result<(String, ContainerInfo)> {
        let key = match paths.first() {
            Some(first_path) => first_path.to_string_lossy().to_string(),
            None => format!("{}{}", DEFAULT_PACK_STEM, DEFAULT_PACK_EXT),
        };

        let already_open = paths.first().is_some_and(|first_path| {
            let normalized = first_path.to_string_lossy().replace('\\', "/");
            self.packs.values().any(|pack| pack.disk_file_path() == normalized.as_str())
        });

        if already_open {
            return Err(anyhow!("Pack '{}' is already open. Close it first if you want to reopen it.", key));
        }

        let mut pack = Pack::read_and_merge(paths, &self.game, lazy_loading, false, false)?;
        decode_tables(&mut pack.files_by_type_mut(&[FileType::DB, FileType::Loc]), &self.schema);

        let key = unique_pack_key(&key, &self.packs);
        let info = ContainerInfo::from(&pack);
        self.insert_pack(key.clone(), pack);
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
        let key = CA_PACKS_KEY.to_owned();
        if self.packs.contains_key(&key) {
            return Err(anyhow!("Pack '{}' is already open. Close it first if you want to reopen it.", key));
        }

        let mut pack = Pack::read_and_merge_ca_packs(&self.game, &settings.path_buf(self.game.key()))?;
        decode_tables(&mut pack.files_by_type_mut(&[FileType::DB, FileType::Loc]), &self.schema);

        let info = ContainerInfo::from(&pack);
        self.insert_pack(key.clone(), pack);
        Ok((key, info))
    }

    /// Closes a pack without saving it.
    pub fn close_pack(&mut self, pack_key: &str) -> Result<()> {
        self.packs.remove(pack_key).ok_or_else(|| ApiError::PackNotFound(pack_key.to_owned()))?;
        self.pack_modes.remove(pack_key);
        self.session.remove_pack_name(pack_key);
        Ok(())
    }

    /// Closes all open packs without saving them.
    pub fn close_all_packs(&mut self) {
        for pack_key in self.packs.keys() {
            self.session.remove_pack_name(pack_key);
        }

        self.packs.clear();
        self.pack_modes.clear();
    }

    /// Returns the key and info of every open pack.
    pub fn open_packs_info(&self) -> Vec<(String, ContainerInfo)> {
        self.packs.iter()
            .map(|(key, pack)| (key.clone(), ContainerInfo::from(pack)))
            .collect()
    }

    /// Saves a pack to disk.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to save.
    /// * `path` - Path to save the pack to. If `None`, the pack is saved to its current path.
    /// * `options` - How to save the pack.
    ///
    /// # Returns
    ///
    /// The info of the saved pack.
    ///
    /// # Errors
    ///
    /// Fails if the pack is of a CA type and editing them is not allowed, if it has no path on disk yet, or if saving fails.
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

        if options.clean {
            pack.clean_undecoded();
        }

        let extra_data = encode_extra_data(&self.game, pack.compression_format(), options.disable_uuid_regeneration);
        pack.save(path, &self.game, &extra_data)
            .map_err(|error| anyhow!("Error while trying to save the currently open PackFile: {}", error))?;

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

    /// Returns the info of a pack and of every file in it.
    pub fn pack_tree_data(&self, pack_key: &str) -> Result<(ContainerInfo, Vec<RFileInfo>)> {
        let pack = pack(&self.packs, pack_key)?;
        Ok((ContainerInfo::from(pack), pack.files().par_iter().map(|(_, file)| From::from(file)).collect()))
    }

    /// Returns the info of a file in a pack, or `None` if it's not in the pack.
    pub fn file_info(&self, pack_key: &str, path: &str) -> Result<Option<RFileInfo>> {
        Ok(pack(&self.packs, pack_key)?.files().get(path).map(From::from))
    }

    /// Returns the info of the files at the provided paths of a pack.
    pub fn files_info(&self, pack_key: &str, paths: &[String]) -> Result<Vec<RFileInfo>> {
        let paths = paths.iter().map(|path| ContainerPath::File(path.to_owned())).collect::<Vec<_>>();
        Ok(pack(&self.packs, pack_key)?.files_by_paths(&paths, false).into_iter().map(From::from).collect())
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

    /// Changes the type of a pack.
    pub fn set_pack_file_type(&mut self, pack_key: &str, pack_type: PFHFileType) -> Result<()> {
        pack_mut(&mut self.packs, pack_key)?.set_pfh_file_type(pack_type);
        Ok(())
    }

    /// Sets or clears one of the flags of a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to change.
    /// * `flag` - Flag to change.
    /// * `state` - If the flag should be set or cleared.
    ///
    /// # Errors
    ///
    /// Fails if the flag is one of the encryption ones and the pack's version doesn't support encryption.
    pub fn set_pack_flag(&mut self, pack_key: &str, flag: PFHFlags, state: bool) -> Result<()> {
        let pack = pack_mut(&mut self.packs, pack_key)?;

        let is_encryption_flag = flag == PFHFlags::HAS_ENCRYPTED_INDEX || flag == PFHFlags::HAS_ENCRYPTED_DATA;
        if is_encryption_flag && state && !pack.pfh_version().supports_encryption() {
            return Err(anyhow!("Encryption is not supported in {} Packs.", pack.pfh_version().value()));
        }

        let mut bitmask = pack.bitmask();
        bitmask.set(flag, state);
        pack.set_bitmask(bitmask);
        Ok(())
    }

    /// Changes the compression format of a pack.
    ///
    /// # Returns
    ///
    /// The format actually set, which may differ from the requested one if the game doesn't support it.
    pub fn set_compression_format(&mut self, pack_key: &str, compression_format: CompressionFormat) -> Result<CompressionFormat> {
        Ok(pack_mut(&mut self.packs, pack_key)?.set_compression_format(compression_format, &self.game))
    }

    /// Returns the parent packs of a pack, and if each one is enabled.
    pub fn pack_dependencies(&self, pack_key: &str) -> Result<Vec<(bool, String)>> {
        Ok(pack(&self.packs, pack_key)?.dependencies().to_vec())
    }

    /// Replaces the parent packs of a pack.
    pub fn set_pack_dependencies(&mut self, pack_key: &str, dependencies: Vec<(bool, String)>) -> Result<()> {
        pack_mut(&mut self.packs, pack_key)?.set_dependencies(dependencies);
        Ok(())
    }

    /// Returns the settings of a pack.
    pub fn pack_settings(&self, pack_key: &str) -> Result<PackSettings> {
        Ok(pack(&self.packs, pack_key)?.settings().clone())
    }

    /// Replaces the settings of a pack.
    pub fn set_pack_settings(&mut self, pack_key: &str, settings: PackSettings) -> Result<()> {
        pack_mut(&mut self.packs, pack_key)?.set_settings(settings);
        Ok(())
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

    /// Changes the operational mode of a pack.
    pub fn set_pack_operational_mode(&mut self, pack_key: &str, mode: OperationalMode) -> Result<()> {
        pack(&self.packs, pack_key)?;
        self.pack_modes.insert(pack_key.to_owned(), mode);
        Ok(())
    }

    /// Returns the operational mode of a pack. Unknown packs are in normal mode.
    pub fn pack_operational_mode(&self, pack_key: &str) -> OperationalMode {
        self.pack_modes.get(pack_key).cloned().unwrap_or(OperationalMode::Normal)
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
    pub fn add_note(&mut self, pack_key: &str, note: Note) -> Result<Note> {
        Ok(pack_mut(&mut self.packs, pack_key)?.notes_mut().add_note(note))
    }

    /// Deletes a note from a pack.
    pub fn delete_note(&mut self, pack_key: &str, path: &str, id: u64) -> Result<()> {
        pack_mut(&mut self.packs, pack_key)?.notes_mut().delete_note(path, id);
        Ok(())
    }

    /// Saves a backup copy of a pack in the autosave folder, removing the oldest copies over the limit.
    ///
    /// Vanilla packs, packs with autosaves disabled and packs that are neither mods nor movies are skipped.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to back up.
    /// * `settings` - Settings, to find the game's install folder.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when encoded.
    /// * `autosave_amount` - Amount of backups to keep per pack.
    pub fn backup_autosave(&self, pack_key: &str, settings: &Settings, disable_uuid_regeneration: bool, autosave_amount: usize) -> Result<()> {
        let pack = pack(&self.packs, pack_key)?;
        let folder = backup_autosave_path()?.join(pack.disk_file_name());
        let _ = DirBuilder::new().recursive(true).create(&folder);

        let game_path = settings.path_buf(self.game.key());
        let ca_paths = self.game.ca_packs_paths(&game_path)
            .unwrap_or_default()
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();

        let pack_disable_autosaves = pack.settings().setting_bool("disable_autosaves").unwrap_or(&true);
        let pack_type = pack.pfh_file_type();
        let pack_path = pack.disk_file_path().replace('\\', "/");

        if folder.is_dir() &&
            !pack_disable_autosaves &&
            (pack_type == PFHFileType::Mod || pack_type == PFHFileType::Movie) &&
            (ca_paths.is_empty() || !ca_paths.contains(&pack_path))
        {
            let date = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
            let new_path = folder.join(format!("{date}.pack"));
            let extra_data = encode_extra_data(&self.game, pack.compression_format(), disable_uuid_regeneration);
            let _ = pack.clone().save(Some(&new_path), &self.game, &extra_data);

            if let Ok(files) = files_in_folder_from_newest_to_oldest(&folder) {
                for file in files.iter().skip(autosave_amount) {
                    let _ = std::fs::remove_file(file);
                }
            }
        }

        Ok(())
    }

    /// Writes the list of tables of a pack with no definition in the schema to `missing_table_definitions.txt`.
    ///
    /// This is slow, and only useful when a new patch lands and you want to know what tables need decoding.
    pub fn export_missing_definitions(&mut self, pack_key: &str) -> Result<()> {
        let pack = pack_mut(&mut self.packs, pack_key)?;

        let mut counter = 0;
        let mut table_list = String::new();
        if let Some(ref schema) = self.schema {
            let mut extra_data = DecodeableExtraData::default();
            extra_data.set_schema(Some(schema));
            let extra_data = Some(extra_data);

            let mut files = pack.files_by_type_mut(&[FileType::DB]);
            files.sort_by_key(|file| file.path_in_container_raw().to_lowercase());

            for file in files {
                if file.decode(&extra_data, false, false).is_err() && file.load().is_ok() {
                    if let Ok(raw_data) = file.cached() {
                        let mut reader = Cursor::new(raw_data);
                        if let Ok((_, _, _, entry_count)) = DB::read_header(&mut reader) {
                            if entry_count > 0 {
                                counter += 1;
                                table_list.push_str(&format!("{}, {:?}\n", counter, file.path_in_container_raw()))
                            }
                        }
                    }
                }
            }
        }

        if let Ok(file) = File::create(exe_path().join("missing_table_definitions.txt")) {
            let mut file = BufWriter::new(file);
            let _ = file.write_all(table_list.as_bytes());
        }

        Ok(())
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

    /// Opens the folder containing a pack in the system's file manager.
    ///
    /// # Errors
    ///
    /// Fails if the pack doesn't exist on disk.
    pub fn open_containing_folder(&self, pack_key: &str) -> Result<()> {
        let mut path_str = pack(&self.packs, pack_key)?.disk_file_path().to_owned();

        // Remove canonicalization, as it breaks the open thingy.
        if path_str.starts_with("//?/") || path_str.starts_with("\\\\?\\") {
            path_str = path_str[4..].to_string();
        }

        let mut path = PathBuf::from(path_str);
        if !path.exists() {
            return Err(anyhow!("This Pack doesn't exists as a file in the disk."));
        }

        path.pop();
        let _ = open::that(&path);
        Ok(())
    }

    /// Adds a pack to the open ones, in normal mode.
    fn insert_pack(&mut self, key: String, pack: Pack) {
        self.session.add_pack_name(&key);
        self.pack_modes.insert(key.clone(), OperationalMode::Normal);
        self.packs.insert(key, pack);
    }
}

/// Derives a unique pack name for new (unsaved) packs, like "new_pack.pack", "new_pack_2.pack", etc.
fn derive_new_pack_name(existing_keys: &BTreeMap<String, Pack>) -> String {
    let base = format!("{}{}", DEFAULT_PACK_STEM, DEFAULT_PACK_EXT);
    if !existing_keys.contains_key(&base) {
        return base;
    }

    (2..).map(|suffix| format!("{}_{}{}", DEFAULT_PACK_STEM, suffix, DEFAULT_PACK_EXT))
        .find(|candidate| !existing_keys.contains_key(candidate))
        .expect("an unbounded range always finds a free name")
}

/// Generates a pack key that doesn't conflict with any open pack, adding a " (2)"-like suffix if needed.
fn unique_pack_key(key: &str, packs: &BTreeMap<String, Pack>) -> String {
    if !packs.contains_key(key) {
        return key.to_string();
    }

    let path = Path::new(key);
    let parent = path.parent().map(|parent| parent.to_path_buf()).unwrap_or_default();
    let stem = path.file_stem().map(|stem| stem.to_string_lossy().to_string()).unwrap_or_else(|| key.to_string());
    let ext = path.extension().map(|ext| ext.to_string_lossy().to_string());

    (2..).map(|suffix| {
            let candidate_name = match &ext {
                Some(ext) => format!("{stem} ({suffix}).{ext}"),
                None => format!("{stem} ({suffix})"),
            };
            parent.join(candidate_name).to_string_lossy().to_string()
        })
        .find(|candidate| !packs.contains_key(candidate))
        .expect("an unbounded range always finds a free key")
}

/// In debug mode, this function returns the base folder of the repo.
/// In release mode, it returns the folder where the executable of the program is.
fn exe_path() -> PathBuf {
    if cfg!(debug_assertions) {
        std::env::current_dir().unwrap()
    } else {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path
    }
}

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! File operations: create, add, delete, move, extract, decode and save files of the open packs,
//! and read files from every data source.

use anyhow::{anyhow, Result};
use base64::{Engine, engine::general_purpose::STANDARD};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::slice::from_ref;

use rpfm_extensions::dependencies::Dependencies;
use rpfm_extensions::optimizer::{OptimizableContainer, OptimizerOptions};

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::files::{
    AddToAnimPack, DeleteFromAnimPack, ExtractFromAnimPack, FileContents, FileData, FileRef, ReadFile, ReadFormat, WriteFile,
    AddFilesFromDisk, ASSEMBLY_KIT_TABLE_FILE_NAME, CopyFiles, CreateFile, DEFAULT_FILES_LIMIT, DeleteFiles, DuplicateFiles, ExtractFiles,
    FileEntry, FileList, FileRename, FileSource, FilesAdded, FilesDeleted, FilesExtracted, FilesRenamed, ListFiles, NewFileKind, RenameFiles,
    FilesPasted, PasteFiles, ViewData,
};
use rpfm_ipc::api::tables::GetTableDefinition;
use rpfm_ipc::api::tools::FilesChanged;
use rpfm_ipc::helpers::{DataSource, NewFile, RFileInfo};

use rpfm_lib::files::{
    animpack::AnimPack, Container, ContainerPath, db::DB, DecodeableExtraData, FileType, loc::Loc,
    pack::{Pack, RESERVED_NAME_NOTES}, portrait_settings::PortraitSettings, RFile, RFileDecoded,
    text::{Text, TextFormat}, video::SupportedFormats,
};
use rpfm_lib::compression::CompressionFormat;
use rpfm_lib::games::{GameInfo, VanillaDBTableNameLogic};
use rpfm_lib::schema::Schema;

use super::{ExtractOptions, PasteEntry, SessionState, decode_tables, encode_extra_data, loaded_schema, pack, pack_mut};

/// File types the server never tries to decode.
const UNDECODEABLE_FILE_TYPES: [FileType; 9] = [
    FileType::Anim,
    FileType::BMD,
    FileType::BMDVegetation,
    FileType::Dat,
    FileType::Font,
    FileType::HlslCompiled,
    FileType::Pack,
    FileType::SoundBank,
    FileType::Unknown,
];

/// Paths of files, by the key of their pack.
pub type PathsByPack = BTreeMap<String, Vec<ContainerPath>>;

/// Result of decoding a file.
#[derive(Debug)]
pub enum DecodedFile {

    /// The decoded file, and its info.
    Decoded(Box<RFileDecoded>, RFileInfo),

    /// The notes of a pack, as markdown.
    Notes(Text),

    /// The file is of a type the server doesn't decode.
    Unsupported,

    /// The file is external to the server, so it's not decoded.
    External,
}

impl SessionState {

    /// Creates a new file in a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to create the file in.
    /// * `path` - Path of the new file in the pack.
    /// * `new_file` - Type and initial data of the new file.
    ///
    /// # Errors
    ///
    /// Fails if the file is a table with no definition in the schema, or if it can't be inserted.
    pub fn new_file(&mut self, pack_key: &str, path: &str, new_file: NewFile) -> Result<()> {
        let decoded = match new_file {
            NewFile::AnimPack(_) => RFileDecoded::AnimPack(AnimPack::default()),
            NewFile::DB(_, table, version) => {
                let schema = loaded_schema(&self.schema)?;
                let definition = schema.definition_by_name_and_version(&table, version)
                    .ok_or_else(|| anyhow!("No definitions found for the table `{}`, version `{}` in the currently loaded schema.", table, version))?;

                RFileDecoded::DB(DB::new(definition, schema.patches_for_table(&table), &table))
            },
            NewFile::Loc(_) => RFileDecoded::Loc(Loc::new()),
            NewFile::PortraitSettings(_, version, entries) => {
                let mut file = PortraitSettings::default();
                file.set_version(version);

                if !entries.is_empty() {
                    let mut vanilla_files = self.dependencies.files_by_types_mut(&[FileType::PortraitSettings], true, true);
                    let vanilla_files_decoded = vanilla_files.iter_mut()
                        .filter_map(|(_, file)| file.decode(&None, false, true).ok().flatten())
                        .filter_map(|file| if let RFileDecoded::PortraitSettings(file) = file { Some(file) } else { None })
                        .collect::<Vec<_>>();

                    let vanilla_values = vanilla_files_decoded.iter()
                        .flat_map(|file| file.entries())
                        .map(|entry| (entry.id(), entry))
                        .collect::<HashMap<_,_>>();

                    for (from_id, to_id) in entries {
                        if let Some(from_entry) = vanilla_values.get(&from_id) {
                            let mut new_entry = (*from_entry).clone();
                            new_entry.set_id(to_id);
                            file.entries_mut().push(new_entry);
                        }
                    }
                }

                RFileDecoded::PortraitSettings(file)
            },
            NewFile::Text(_, text_type) => {
                let mut file = Text::default();
                file.set_format(text_type);
                RFileDecoded::Text(file)
            },
            NewFile::VMD(_) => {
                let mut file = Text::default();
                file.set_format(TextFormat::Xml);
                RFileDecoded::VMD(file)
            },
            NewFile::WSModel(_) => {
                let mut file = Text::default();
                file.set_format(TextFormat::Xml);
                RFileDecoded::WSModel(file)
            },
        };

        let file = RFile::new_from_decoded(&decoded, 0, path);
        pack_mut(&mut self.packs, pack_key)?.insert(file)?;
        Ok(())
    }

    /// Returns a page of the files of a source matching the request, sorted by path.
    pub fn list_files(&self, request: &ListFiles) -> Result<FileList> {
        let keep = |path: &str, file_type: FileType| path.starts_with(&request.prefix) &&
            request.file_types.as_ref().is_none_or(|file_types| file_types.contains(&file_type));
        let entry = |file: &RFile| FileEntry { path: file.path_in_container_raw().to_owned(), file_type: file.file_type() };

        let mut files = match request.source {
            FileSource::Pack(ref pack_key) => pack(&self.packs, pack_key)?.files().values()
                .filter(|file| keep(file.path_in_container_raw(), file.file_type()))
                .map(entry)
                .collect::<Vec<_>>(),
            FileSource::GameFiles => self.dependencies.vanilla_loose_files().values()
                .chain(self.dependencies.vanilla_files().values())
                .filter(|file| keep(file.path_in_container_raw(), file.file_type()))
                .map(entry)
                .collect(),
            FileSource::ParentFiles => self.dependencies.parent_files().values()
                .filter(|file| keep(file.path_in_container_raw(), file.file_type()))
                .map(entry)
                .collect(),
            FileSource::AssemblyKit => self.dependencies.asskit_only_db_tables().keys()
                .map(|table_name| FileEntry { path: format!("db/{table_name}/{ASSEMBLY_KIT_TABLE_FILE_NAME}"), file_type: FileType::DB })
                .filter(|file| keep(&file.path, file.file_type))
                .collect(),
        };

        // In non-recursive listings, files in subfolders are replaced by their subfolder.
        let mut folders = BTreeSet::new();
        if !request.recursive {
            let base = if request.prefix.is_empty() || request.prefix.ends_with('/') { request.prefix.clone() } else { format!("{}/", request.prefix) };
            files.retain(|file| match file.path.strip_prefix(&base).and_then(|relative| relative.split_once('/')) {
                Some((folder, _)) => {
                    folders.insert(format!("{base}{folder}"));
                    false
                }
                None => file.path.starts_with(&base),
            });
        }

        files.sort_unstable_by(|a, b| a.path.cmp(&b.path));

        let total = files.len();
        let files = files.into_iter()
            .skip(request.offset)
            .take(request.limit.unwrap_or(DEFAULT_FILES_LIMIT))
            .collect();

        Ok(FileList { files, folders: folders.into_iter().collect(), total })
    }

    /// Adds files and folders from disk to a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to add the files to.
    /// * `source_paths` - Paths on disk of the files and folders to add.
    /// * `destination_paths` - Path in the pack of each source path.
    /// * `paths_to_ignore` - Source paths under any of these are skipped.
    /// * `include_base_folder` - If added folders keep their own name in the pack, or only their contents are added.
    ///
    /// # Returns
    ///
    /// The paths added, and the last error found while adding them, if any.
    pub fn add_files_from_disk(&mut self, pack_key: &str, source_paths: &[PathBuf], destination_paths: &[ContainerPath], paths_to_ignore: &Option<Vec<PathBuf>>, include_base_folder: bool) -> Result<(Vec<ContainerPath>, Option<String>)> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let mut added_paths = vec![];
        let mut last_error = None;

        for (source_path, destination_path) in source_paths.iter().zip(destination_paths.iter()) {
            if let Some(ref paths_to_ignore) = paths_to_ignore {
                if paths_to_ignore.iter().any(|path| source_path.starts_with(path)) {
                    continue;
                }
            }

            match destination_path {
                ContainerPath::File(destination_path) => match pack.insert_file(source_path, destination_path, &self.schema) {
                    Ok(path) => added_paths.extend(path),
                    Err(error) => last_error = Some(error.to_string()),
                },
                ContainerPath::Folder(destination_path) => match pack.insert_folder(source_path, destination_path, &None, &self.schema, include_base_folder) {
                    Ok(mut paths) => added_paths.append(&mut paths),
                    Err(error) => last_error = Some(error.to_string()),
                },
            }
        }

        decode_tables(&mut pack.files_by_paths_mut(&added_paths, false), &self.schema);
        Ok((added_paths, last_error))
    }

    /// Copies files from one open pack to another.
    ///
    /// # Arguments
    ///
    /// * `target_key` - Key of the pack to copy the files to.
    /// * `source_key` - Key of the pack to copy the files from.
    /// * `paths` - Paths of the files and folders to copy.
    ///
    /// # Returns
    ///
    /// The paths added to the target pack.
    pub fn add_files_from_pack(&mut self, target_key: &str, source_key: &str, paths: &[ContainerPath]) -> Result<Vec<ContainerPath>> {
        let files = loaded_files_by_paths(pack(&self.packs, source_key).map_err(|_| anyhow!("Source pack not found: {}", source_key))?, paths);
        let target_pack = pack_mut(&mut self.packs, target_key).map_err(|_| anyhow!("Target pack not found: {}", target_key))?;

        let added_paths = insert_files(target_pack, files);
        decode_tables(&mut target_pack.files_by_paths_mut(&added_paths, false), &self.schema);
        Ok(added_paths)
    }

    /// Copies files from an open pack into an AnimPack of another (or the same) open pack.
    ///
    /// # Arguments
    ///
    /// * `source_pack_key` - Key of the pack to copy the files from.
    /// * `anim_pack_key` - Key of the pack containing the AnimPack.
    /// * `anim_pack_path` - Path of the AnimPack in its pack.
    /// * `paths` - Paths of the files and folders to copy.
    ///
    /// # Returns
    ///
    /// The paths added to the AnimPack.
    pub fn add_files_to_animpack(&mut self, source_pack_key: &str, anim_pack_key: &str, anim_pack_path: &str, paths: &[ContainerPath]) -> Result<Vec<ContainerPath>> {
        let files = loaded_files_by_paths(pack(&self.packs, source_pack_key)?, paths);
        let anim_pack_file = pack_mut(&mut self.packs, anim_pack_key)?.files_mut()
            .get_mut(anim_pack_path)
            .ok_or_else(|| anyhow!("File not found in the Pack: {}.", anim_pack_path))?;

        let anim_pack = decoded_anim_pack(anim_pack_file, anim_pack_path)?;
        Ok(files.into_iter()
            .filter_map(|file| anim_pack.insert(file).ok().flatten())
            .collect())
    }

    /// Copies files from an AnimPack into an open pack.
    ///
    /// # Arguments
    ///
    /// * `anim_pack_key` - Key of the pack containing the AnimPack. Only used if `data_source` is a pack.
    /// * `dest_pack_key` - Key of the pack to copy the files to.
    /// * `data_source` - Where the AnimPack is.
    /// * `anim_pack_path` - Path of the AnimPack in its source.
    /// * `paths` - Paths in the AnimPack of the files and folders to copy.
    ///
    /// # Returns
    ///
    /// The paths of the copied files.
    pub fn add_files_from_animpack(&mut self, anim_pack_key: &str, dest_pack_key: &str, data_source: DataSource, anim_pack_path: &str, paths: &[ContainerPath]) -> Result<Vec<ContainerPath>> {
        let anim_pack_file = match data_source {
            DataSource::PackFile => self.packs.get_mut(anim_pack_key).and_then(|pack| pack.files_mut().get_mut(anim_pack_path)),
            DataSource::GameFiles => self.dependencies.file_mut(anim_pack_path, true, false).ok(),
            DataSource::ParentFiles => self.dependencies.file_mut(anim_pack_path, false, true).ok(),
            DataSource::AssKitFiles |
            DataSource::ExternalFile => return Err(anyhow!("You can't import files from this source.")),
        }.ok_or_else(|| anyhow!("The file with the path {} doesn't exists on the open Pack.", anim_pack_path))?;

        let files = decoded_anim_pack(anim_pack_file, anim_pack_path)?
            .files_by_paths(paths, false)
            .into_iter()
            .cloned()
            .collect::<Vec<RFile>>();

        let result_paths = files.iter().map(|file| file.path_in_container()).collect::<Vec<_>>();
        let pack = pack_mut(&mut self.packs, dest_pack_key)?;
        for mut file in files {
            let _ = file.guess_file_type();
            let _ = pack.insert(file);
        }

        Ok(result_paths)
    }

    /// Deletes files from an AnimPack in an open pack.
    pub fn delete_from_animpack(&mut self, pack_key: &str, anim_pack_path: &str, paths: &[ContainerPath]) -> Result<()> {
        let anim_pack_file = pack_mut(&mut self.packs, pack_key)?.files_mut()
            .get_mut(anim_pack_path)
            .ok_or_else(|| anyhow!("File not found in the Pack: {}.", anim_pack_path))?;

        let anim_pack = decoded_anim_pack(anim_pack_file, anim_pack_path)?;
        for path in paths {
            anim_pack.remove(path);
        }

        Ok(())
    }

    /// Decodes a file as a client opening it in a view needs it.
    ///
    /// # Arguments
    ///
    /// * `file` - The file.
    /// * `enable_esf_editor` - If ESF files are decoded.
    pub fn view_data(&mut self, file: &FileRef, enable_esf_editor: bool) -> Result<ViewData> {
        let (pack_key, data_source) = pack_key_and_data_source(&file.source);
        let pack_key = pack_key.to_owned();
        let (decoded, info) = match self.decode_file(&pack_key, &file.path, data_source, enable_esf_editor)? {
            DecodedFile::Decoded(decoded, info) => (decoded, info),
            DecodedFile::Notes(notes) => return Ok(ViewData::Notes(notes)),
            DecodedFile::Unsupported | DecodedFile::External => return Ok(ViewData::Unsupported),
        };

        Ok(match *decoded {
            RFileDecoded::AnimPack(data) => ViewData::AnimPack(data.files().values().map(From::from).collect(), info),
            RFileDecoded::Video(data) => ViewData::Video(From::from(&data), info),
            RFileDecoded::Anim(_) |
            RFileDecoded::BMD(_) |
            RFileDecoded::BMDVegetation(_) |
            RFileDecoded::Dat(_) |
            RFileDecoded::Font(_) |
            RFileDecoded::HlslCompiled(_) |
            RFileDecoded::Pack(_) |
            RFileDecoded::SoundBank(_) |
            RFileDecoded::Unknown(_) => ViewData::Unsupported,
            decoded @ (RFileDecoded::AnimFragmentBattle(_) |
                RFileDecoded::AnimsTable(_) |
                RFileDecoded::Atlas(_) |
                RFileDecoded::Audio(_) |
                RFileDecoded::DB(_) |
                RFileDecoded::ESF(_) |
                RFileDecoded::GroupFormations(_) |
                RFileDecoded::Image(_) |
                RFileDecoded::Loc(_) |
                RFileDecoded::MatchedCombat(_) |
                RFileDecoded::PortraitSettings(_) |
                RFileDecoded::RigidModel(_) |
                RFileDecoded::Text(_) |
                RFileDecoded::UIC(_) |
                RFileDecoded::UnitVariant(_) |
                RFileDecoded::VMD(_) |
                RFileDecoded::WSModel(_)) => ViewData::Decoded(decoded, info),
        })
    }

    /// Decodes a file.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack containing the file. Only used if `data_source` is a pack.
    /// * `path` - Path of the file in its source.
    /// * `data_source` - Where the file is.
    /// * `enable_esf_editor` - If ESF files are decoded.
    ///
    /// # Returns
    ///
    /// The decoded file, or why it wasn't decoded.
    pub fn decode_file(&mut self, pack_key: &str, path: &str, data_source: DataSource, enable_esf_editor: bool) -> Result<DecodedFile> {
        match data_source {
            DataSource::PackFile => {
                let pack = pack_mut(&mut self.packs, pack_key)?;
                if path == RESERVED_NAME_NOTES {
                    let mut notes = Text::default();
                    notes.set_format(TextFormat::Markdown);
                    notes.set_contents(pack.notes().pack_notes().to_owned());
                    return Ok(DecodedFile::Notes(notes));
                }

                let file = pack.files_mut().get_mut(path)
                    .ok_or_else(|| anyhow!("The file with the path {} hasn't been found on this Pack.", path))?;
                decode(file, &self.game, &self.schema, enable_esf_editor)
            }
            DataSource::ParentFiles => decode(self.dependencies.file_mut(path, false, true)?, &self.game, &self.schema, enable_esf_editor),
            DataSource::GameFiles => decode(self.dependencies.file_mut(path, true, false)?, &self.game, &self.schema, enable_esf_editor),
            DataSource::AssKitFiles => {
                let table_name = path.split('/').nth(1)
                    .filter(|_| path.split('/').count() > 2)
                    .ok_or_else(|| anyhow!("Path {} doesn't contain an identifiable table name.", path))?;

                let table = self.dependencies.asskit_only_db_tables().get(table_name)
                    .ok_or_else(|| anyhow!("Table {} not found on Assembly Kit files.", path))?;

                Ok(DecodedFile::Decoded(Box::new(RFileDecoded::DB(table.clone())), RFileInfo::default()))
            }
            DataSource::ExternalFile => Ok(DecodedFile::External),
        }
    }

    /// Replaces the data of a file of a pack with its edited version.
    pub fn save_file_from_view(&mut self, pack_key: &str, path: &str, decoded: RFileDecoded) -> Result<()> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        if path == RESERVED_NAME_NOTES {
            if let RFileDecoded::Text(data) = decoded {
                pack.notes_mut().set_pack_notes(data.contents().to_owned());
            }
        } else if let Some(file) = pack.files_mut().get_mut(path) {
            file.set_decoded(decoded)?;
        }

        Ok(())
    }

    /// Deletes files and folders from a pack.
    ///
    /// # Returns
    ///
    /// The paths deleted.
    pub fn delete_files(&mut self, pack_key: &str, paths: &[ContainerPath]) -> Result<Vec<ContainerPath>> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        Ok(paths.iter().flat_map(|path| pack.remove(path)).collect())
    }

    /// Copies or moves files and folders of open packs into a folder of an open pack. Cut files are removed from
    /// their packs before pasting, so they can be pasted where they were.
    ///
    /// # Returns
    ///
    /// The paths added to the target pack, and the paths removed from each source pack if the files were cut.
    pub fn paste_files(&mut self, request: &PasteFiles) -> Result<FilesPasted> {
        let target_key = &request.to_pack;
        pack(&self.packs, target_key)?;

        let mut entries = vec![];
        for (pack_key, paths) in &request.sources {
            let source_pack = pack(&self.packs, pack_key)?;
            let paths = paths.iter().map(|path| container_path(|path| source_pack.has_file(path), path)).collect::<Vec<_>>();
            entries.extend(paste_entries_from_paths(source_pack, &paths, pack_key));
        }

        if entries.is_empty() {
            return Err(ApiError::InvalidParams("There are no files to paste.".to_owned()).into());
        }

        // Clone the files first, so we don't hold borrows of their packs while mutating them.
        let destination_path = request.destination.trim_end_matches('/');
        let mut files_to_insert = Vec::with_capacity(entries.len());
        for entry in &entries {
            let Some(source_pack) = self.packs.get(&entry.source_pack_key) else { continue };
            let Some(file) = source_pack.files_by_paths(&[ContainerPath::File(entry.file_path.clone())], false).first().copied() else { continue };

            let mut new_file = file.clone();
            let _ = new_file.load();

            let relative_path = if !entry.base_path.is_empty() && entry.file_path.starts_with(&entry.base_path) {
                entry.file_path[entry.base_path.len()..].trim_start_matches('/')
            } else {
                &entry.file_path
            };

            let new_path = if destination_path.is_empty() {
                relative_path.to_string()
            } else {
                format!("{destination_path}/{relative_path}")
            };

            new_file.set_path_in_container_raw(&new_path);
            files_to_insert.push(new_file);
        }

        let mut deleted: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if request.cut {
            for entry in &entries {
                if let Some(source_pack) = self.packs.get_mut(&entry.source_pack_key) {
                    let removed = source_pack.remove(&ContainerPath::File(entry.file_path.clone()));
                    deleted.entry(entry.source_pack_key.clone()).or_default().extend(raw_paths(&removed));
                }
            }
        }

        let target_pack = pack_mut(&mut self.packs, target_key)?;
        let added_paths = insert_files(target_pack, files_to_insert);
        decode_tables(&mut target_pack.files_by_paths_mut(&added_paths, false), &self.schema);

        Ok(FilesPasted { added: raw_paths(&added_paths), deleted })
    }

    /// Duplicates files in the same pack, adding a numeric suffix to their names.
    ///
    /// # Returns
    ///
    /// The paths of the duplicates.
    pub fn duplicate_files(&mut self, pack_key: &str, paths: &[ContainerPath]) -> Result<Vec<ContainerPath>> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let files_to_duplicate = pack.files_by_paths(paths, false)
            .into_iter()
            .cloned()
            .collect::<Vec<RFile>>();

        let mut added_paths = Vec::with_capacity(files_to_duplicate.len());
        for mut file in files_to_duplicate {
            let new_path = duplicate_path(pack, file.path_in_container_raw());
            file.set_path_in_container_raw(&new_path);

            if let Ok(Some(path)) = pack.insert(file) {
                added_paths.push(path);
            }
        }

        decode_tables(&mut pack.files_by_paths_mut(&added_paths, false), &self.schema);
        Ok(added_paths)
    }

    /// Extracts files from a pack or from the dependencies to disk.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to extract from. Only used for paths from a pack.
    /// * `paths_by_source` - Paths to extract, by source. If there are paths from a pack, only those are extracted.
    /// * `destination` - Folder to extract the files to.
    /// * `as_tsv` - If tables are extracted as TSV.
    /// * `options` - Options for writing the files.
    ///
    /// # Returns
    ///
    /// The paths on disk of the extracted files.
    ///
    /// # Errors
    ///
    /// Fails if any file couldn't be extracted.
    pub fn extract_files(&mut self, pack_key: &str, paths_by_source: &BTreeMap<DataSource, Vec<ContainerPath>>, destination: &Path, as_tsv: bool, options: ExtractOptions) -> Result<Vec<PathBuf>> {
        let no_schema = None;
        let schema = if as_tsv { &self.schema } else { &no_schema };
        let mut errors = 0;
        let mut extracted_paths = vec![];

        if let Some(container_paths) = paths_by_source.get(&DataSource::PackFile) {
            let pack = pack_mut(&mut self.packs, pack_key)?;
            let extra_data = encode_extra_data(&self.game, pack.compression_format(), options.disable_uuid_regeneration);

            for container_path in container_paths {
                match pack.extract(container_path.clone(), destination, true, schema, false, options.tsv_keys_first, &extra_data) {
                    Ok(mut paths) => extracted_paths.append(&mut paths),
                    Err(_) => errors += 1,
                }
            }
        }

        else {
            let mut files = match paths_by_source.get(&DataSource::GameFiles) {
                Some(paths) => self.dependencies.files_by_path(paths, true, false, false),
                None => HashMap::new(),
            };

            if let Some(paths) = paths_by_source.get(&DataSource::ParentFiles) {
                files.extend(self.dependencies.files_by_path(paths, false, true, false));
            }

            let mut pack = Pack::default();
            let extra_data = encode_extra_data(&self.game, pack.compression_format(), options.disable_uuid_regeneration);
            for (path_raw, file) in files {
                if pack.insert(file.clone()).is_err() {
                    errors += 1;
                    continue;
                }

                let container_path = ContainerPath::File(path_raw);
                match pack.extract(container_path.clone(), destination, true, schema, false, options.tsv_keys_first, &extra_data) {
                    Ok(mut paths) => extracted_paths.append(&mut paths),
                    Err(_) => errors += 1,
                }

                // Drop the cloned file from the temp pack so memory doesn't grow with the batch.
                pack.remove(&container_path);
            }
        }

        if errors == 0 {
            Ok(extracted_paths)
        } else {
            Err(anyhow!("There were {} errors while extracting.", errors))
        }
    }

    /// Renames or moves files and folders in a pack.
    ///
    /// # Returns
    ///
    /// The old and new path of each moved file.
    pub fn rename_files(&mut self, pack_key: &str, renames: &[(ContainerPath, ContainerPath)]) -> Result<Vec<(ContainerPath, ContainerPath)>> {
        Ok(pack_mut(&mut self.packs, pack_key)?.move_paths(renames)?)
    }

    /// Returns if a file exists in a pack.
    pub fn file_exists(&self, pack_key: &str, path: &str) -> Result<bool> {
        Ok(pack(&self.packs, pack_key)?.has_file(path))
    }

    /// Extracts a file of a pack to a temp folder and opens it in the system's default program for its type.
    ///
    /// # Returns
    ///
    /// The path of the extracted file.
    ///
    /// # Errors
    ///
    /// Fails if the pack isn't open, or if the file can't be extracted.
    pub fn open_in_external_program(&mut self, pack_key: &str, path: &str, options: ExtractOptions) -> Result<PathBuf> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let folder = temp_dir().join(format!("rpfm_{}", pack.disk_file_name()));
        let extra_data = encode_extra_data(&self.game, pack.compression_format(), options.disable_uuid_regeneration);

        let extracted_paths = pack.extract(ContainerPath::File(path.to_owned()), &folder, true, &self.schema, false, options.tsv_keys_first, &extra_data)?;
        let extracted_path = extracted_paths.first()
            .ok_or_else(|| anyhow!("Nothing was extracted from {path}."))?;

        let _ = open::that(extracted_path);
        Ok(extracted_path.to_owned())
    }

    /// Replaces a file of a pack with the data of a file on disk, edited by an external program.
    pub fn save_file_from_external(&mut self, pack_key: &str, path: &str, external_path: &Path) -> Result<()> {
        let file = pack_mut(&mut self.packs, pack_key)?.file_mut(path, false)
            .ok_or_else(|| anyhow!("File not found"))?;

        file.encode_from_external_data(&self.schema, external_path)?;
        Ok(())
    }

    /// Adds files edited by the client to a pack, replacing existing ones, and optionally optimizes the pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to add the files to.
    /// * `files` - Files to add, already named with their path in the pack.
    /// * `optimizer_options` - If set, the pack is optimized with these options after adding the files.
    ///
    /// # Returns
    ///
    /// The paths added, and the paths deleted by the optimizer.
    pub fn save_files_and_optimize(&mut self, pack_key: &str, files: Vec<RFile>, optimizer_options: Option<OptimizerOptions>) -> Result<FilesChanged> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let schema = loaded_schema(&self.schema)?;

        let mut added_paths = insert_files(pack, files);
        added_paths.sort();
        added_paths.dedup();

        let mut added = raw_paths(&added_paths);
        let Some(options) = optimizer_options else {
            return Ok(FilesChanged { added, deleted: vec![] });
        };

        let (paths_to_delete, paths_to_add) = pack.optimize(None, &mut self.dependencies, schema, &self.game, &options)?;
        added.extend(paths_to_add);
        Ok(FilesChanged { added, deleted: paths_to_delete.into_iter().collect() })
    }

    /// Changes the format of a ca_vp8 video of a pack.
    pub fn set_video_format(&mut self, pack_key: &str, path: &str, format: SupportedFormats) -> Result<()> {
        let file = pack_mut(&mut self.packs, pack_key)?.files_mut()
            .get_mut(path)
            .ok_or_else(|| anyhow!("File not found in the Pack: {}.", path))?;

        match file.decoded_mut()? {
            RFileDecoded::Video(data) => {
                data.set_format(format);
                Ok(())
            }
            _ => Err(anyhow!("The file is not a video.")),
        }
    }

    /// Returns the files at the provided paths from the open packs, the parent packs and the game files.
    ///
    /// # Arguments
    ///
    /// * `paths` - Paths of the files and folders to get.
    /// * `lowercase_paths` - If the returned paths are lowercased.
    ///
    /// # Returns
    ///
    /// The files found, by source and path.
    pub fn files_from_all_sources(&self, paths: &[ContainerPath], lowercase_paths: bool) -> HashMap<DataSource, HashMap<String, RFile>> {
        let normalize = |path: String| if lowercase_paths { path.to_lowercase() } else { path };

        let parent_files = self.dependencies.files_by_path(paths, false, true, true).into_iter()
            .map(|(path, file)| (normalize(path), file.clone()))
            .collect();

        let game_files = self.dependencies.files_by_path(paths, true, false, true).into_iter()
            .map(|(path, file)| (normalize(path), file.clone()))
            .collect();

        let pack_files = self.packs.values()
            .flat_map(|pack| pack.files_by_paths(paths, true))
            .map(|file| (normalize(file.path_in_container_raw().to_owned()), file.clone()))
            .collect();

        HashMap::from([
            (DataSource::ParentFiles, parent_files),
            (DataSource::GameFiles, game_files),
            (DataSource::PackFile, pack_files),
        ])
    }

    /// Returns the paths of the animations using a skeleton, from the open packs, the parent packs and the game files.
    pub fn anim_paths_by_skeleton(&mut self, skeleton_name: &str) -> HashSet<String> {
        let mut paths = HashSet::new();

        for (game_files, parent_files) in [(true, false), (false, true)] {
            for (path, file) in self.dependencies.files_by_types_mut(&[FileType::Anim], game_files, parent_files) {
                if let Ok(Some(RFileDecoded::Anim(file))) = file.decode(&None, false, true) {
                    if file.skeleton_name() == skeleton_name {
                        paths.insert(path);
                    }
                }
            }
        }

        for pack in self.packs.values_mut() {
            for file in pack.files_by_type_mut(&[FileType::Anim]) {
                if let Ok(Some(RFileDecoded::Anim(anim_file))) = file.decode(&None, false, true) {
                    if anim_file.skeleton_name() == skeleton_name {
                        paths.insert(file.path_in_container_raw().to_owned());
                    }
                }
            }
        }

        paths
    }

    /// Copies files from the dependencies (game files, parent packs or Assembly Kit tables) into a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to copy the files into.
    /// * `paths_by_source` - Paths of the files and folders to copy, by source.
    ///
    /// # Returns
    ///
    /// The paths added, and the paths that couldn't be added.
    pub fn import_dependencies(&mut self, pack_key: &str, paths_by_source: &BTreeMap<DataSource, Vec<ContainerPath>>) -> Result<(Vec<ContainerPath>, Vec<String>)> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let mut added_paths = vec![];
        let mut not_added_paths = vec![];

        for (data_source, paths) in paths_by_source {
            let files = match data_source {
                DataSource::GameFiles => self.dependencies.files_by_path(paths, true, false, false),
                DataSource::ParentFiles => self.dependencies.files_by_path(paths, false, true, false),
                DataSource::AssKitFiles => continue,
                DataSource::PackFile |
                DataSource::ExternalFile => return Err(anyhow!("You can't import files from this source.")),
            };

            for file in files.into_values() {
                let mut file = file.clone();
                let _ = file.guess_file_type();
                let file_path = file.path_in_container_raw().to_owned();
                match pack.insert(file) {
                    Ok(Some(path)) => added_paths.push(path),
                    _ => not_added_paths.push(file_path),
                }
            }
        }

        // Once we're done with normal files, we process the Assembly Kit ones.
        if let Some(paths) = paths_by_source.get(&DataSource::AssKitFiles) {
            let schema = loaded_schema(&self.schema)?;
            let table_name_logic = self.game.vanilla_db_table_name_logic();
            let mut files = vec![];

            for path in paths {
                match path {

                    // We only have tables. If it's a folder, it's either a table folder, db or the root.
                    ContainerPath::Folder(path) => {
                        let path = path.strip_suffix('/').unwrap_or(path);
                        let path_split = path.split('/').collect::<Vec<_>>();

                        let table_names = match path_split.len() {
                            1 => self.dependencies.asskit_only_db_tables().keys().map(|name| name.as_str()).collect::<Vec<_>>(),
                            2 => vec![path_split[1]],
                            _ => return Err(anyhow!("No idea how you were able to trigger this.")),
                        };

                        for table_name in table_names {
                            let table_file_name = match table_name_logic {
                                VanillaDBTableNameLogic::DefaultName(name) => name.as_str(),
                                VanillaDBTableNameLogic::FolderName => table_name,
                            };

                            let mut file_path = path_split.clone();
                            file_path.push(table_file_name);
                            match import_asskit_table(&self.dependencies, schema, table_name, &file_path.join("/")) {
                                Some(file) => files.push(file),
                                None => not_added_paths.push(path.to_owned()),
                            }
                        }
                    }
                    ContainerPath::File(path) => {
                        let table_name = path.split('/').nth(1).unwrap_or_default();
                        match import_asskit_table(&self.dependencies, schema, table_name, path) {
                            Some(file) => files.push(file),
                            None => not_added_paths.push(path.clone()),
                        }
                    }
                }
            }

            added_paths.append(&mut insert_files(pack, files));
        }

        Ok((added_paths, not_added_paths))
    }
}

impl SessionState {

    /// Creates a new empty file in an open pack.
    ///
    /// # Returns
    ///
    /// The path and type of the new file.
    pub fn create_file(&mut self, request: &CreateFile) -> Result<FileEntry> {
        let new_file = match request.kind {
            NewFileKind::Db { ref table_name, version } => {
                let definition = self.table_definition(&GetTableDefinition { table_name: table_name.clone(), version })?;
                NewFile::DB(request.path.clone(), table_name.clone(), definition.version)
            }
            NewFileKind::Loc => NewFile::Loc(request.path.clone()),
            NewFileKind::Text { format } => NewFile::Text(request.path.clone(), format.unwrap_or(TextFormat::Plain)),
            NewFileKind::AnimPack => NewFile::AnimPack(request.path.clone()),
            NewFileKind::PortraitSettings { version, ref copy_entries } => {
                let entries = copy_entries.iter().map(|entry| (entry.from.clone(), entry.to.clone())).collect();
                NewFile::PortraitSettings(request.path.clone(), version, entries)
            }
            NewFileKind::Vmd => NewFile::VMD(request.path.clone()),
            NewFileKind::WsModel => NewFile::WSModel(request.path.clone()),
        };

        self.new_file(&request.pack, &request.path, new_file)?;

        let file = pack(&self.packs, &request.pack)?.files().get(&request.path)
            .ok_or_else(|| ApiError::FileNotFound(request.path.clone()))?;
        Ok(FileEntry { path: file.path_in_container_raw().to_owned(), file_type: file.file_type() })
    }

    /// Adds files and folders from disk to an open pack, under a folder of the pack.
    ///
    /// # Arguments
    ///
    /// * `request` - What to add, and where.
    /// * `include_base_folder` - If added folders keep their own name in the pack.
    pub fn add_disk_files(&mut self, request: &AddFilesFromDisk, include_base_folder: bool) -> Result<FilesAdded> {
        let destination_paths = match request.destinations {
            Some(ref destinations) => {
                if destinations.len() != request.paths.len() {
                    return Err(ApiError::InvalidParams("There must be one destination per path.".to_owned()).into());
                }

                request.paths.iter().zip(destinations)
                    .map(|(path, destination)| if path.is_file() {
                        ContainerPath::File(destination.to_owned())
                    } else {
                        ContainerPath::Folder(destination.trim_end_matches('/').to_owned())
                    })
                    .collect::<Vec<_>>()
            }
            None => {
                let destination = request.destination.trim_end_matches('/');
                request.paths.iter()
                    .map(|path| match path.file_name().filter(|_| path.is_file()) {
                        Some(name) if destination.is_empty() => ContainerPath::File(name.to_string_lossy().to_string()),
                        Some(name) => ContainerPath::File(format!("{destination}/{}", name.to_string_lossy())),
                        None => ContainerPath::Folder(destination.to_owned()),
                    })
                    .collect::<Vec<_>>()
            }
        };

        let ignore = if request.ignore.is_empty() { None } else { Some(request.ignore.clone()) };
        let (added, error) = self.add_files_from_disk(&request.pack, &request.paths, &destination_paths, &ignore, include_base_folder)?;
        Ok(FilesAdded { added: raw_paths(&added), not_added: vec![], error })
    }

    /// Copies files and folders from any source into an open pack, keeping their paths.
    ///
    /// Assembly Kit tables get the file name tables have in the game files.
    pub fn copy_files_to_pack(&mut self, request: &CopyFiles) -> Result<FilesAdded> {
        match request.from {
            FileSource::Pack(ref source_key) => {
                let source = pack(&self.packs, source_key)?;
                let (paths, not_found) = split_found(&request.paths, |path| container_path(|path| source.has_file(path), path), |path| !source.files_by_path(path, false).is_empty());

                let added = self.add_files_from_pack(&request.to_pack, source_key, &paths)?;
                Ok(FilesAdded { added: raw_paths(&added), not_added: not_found, error: None })
            }
            FileSource::GameFiles | FileSource::ParentFiles => {
                let data_source = DataSource::from(&request.from);
                let (include_vanilla, include_parent) = (data_source == DataSource::GameFiles, data_source == DataSource::ParentFiles);
                let (paths, mut not_found) = split_found(
                    &request.paths,
                    |path| container_path(|path| self.dependencies.file_exists(path, include_vanilla, include_parent, false), path),
                    |path| !self.dependencies.files_by_path(from_ref(path), include_vanilla, include_parent, false).is_empty(),
                );

                let (added, mut not_added) = self.import_dependencies(&request.to_pack, &BTreeMap::from([(data_source, paths)]))?;
                not_added.append(&mut not_found);
                Ok(FilesAdded { added: raw_paths(&added), not_added, error: None })
            }
            FileSource::AssemblyKit => {

                // Their listed paths end in a placeholder file name, so they're copied as their table folder instead.
                let paths = request.paths.iter()
                    .map(|path| ContainerPath::Folder(path.strip_suffix(ASSEMBLY_KIT_TABLE_FILE_NAME).unwrap_or(path).trim_end_matches('/').to_owned()))
                    .collect();

                let (added, not_added) = self.import_dependencies(&request.to_pack, &BTreeMap::from([(DataSource::AssKitFiles, paths)]))?;
                Ok(FilesAdded { added: raw_paths(&added), not_added, error: None })
            }
        }
    }

    /// Deletes files and folders from an open pack.
    pub fn delete_paths(&mut self, request: &DeleteFiles) -> Result<FilesDeleted> {
        let paths = self.pack_container_paths(&request.pack, &request.paths)?;
        let deleted = self.delete_files(&request.pack, &paths)?;
        Ok(FilesDeleted { deleted: raw_paths(&deleted) })
    }

    /// Renames or moves files and folders of an open pack.
    pub fn rename_paths(&mut self, request: &RenameFiles) -> Result<FilesRenamed> {
        let pack = pack(&self.packs, &request.pack)?;
        let renames = request.renames.iter()
            .map(|rename| match container_path(|path| pack.has_file(path), &rename.from) {
                ContainerPath::File(from) => (ContainerPath::File(from), ContainerPath::File(rename.to.clone())),
                ContainerPath::Folder(from) => (ContainerPath::Folder(from), ContainerPath::Folder(rename.to.clone())),
            })
            .collect::<Vec<_>>();

        let renamed = self.rename_files(&request.pack, &renames)?;
        Ok(FilesRenamed {
            renamed: renamed.iter()
                .map(|(from, to)| FileRename { from: from.path_raw().to_owned(), to: to.path_raw().to_owned() })
                .collect(),
        })
    }

    /// Duplicates files of an open pack in the same pack.
    pub fn duplicate_paths(&mut self, request: &DuplicateFiles) -> Result<FilesAdded> {
        let paths = self.pack_container_paths(&request.pack, &request.paths)?;
        let added = self.duplicate_files(&request.pack, &paths)?;
        Ok(FilesAdded { added: raw_paths(&added), ..FilesAdded::default() })
    }

    /// Extracts files and folders of an open pack, the game files or the parent packs to disk.
    ///
    /// # Arguments
    ///
    /// * `request` - What to extract, and where.
    /// * `options` - Options for writing the files.
    pub fn extract_paths(&mut self, request: &ExtractFiles, options: ExtractOptions) -> Result<FilesExtracted> {
        let (pack_key, data_source, paths) = match request.source {
            FileSource::Pack(ref pack_key) => (pack_key.as_str(), DataSource::PackFile, self.pack_container_paths(pack_key, &request.paths)?),
            FileSource::GameFiles | FileSource::ParentFiles => {
                let data_source = DataSource::from(&request.source);
                let (include_vanilla, include_parent) = (data_source == DataSource::GameFiles, data_source == DataSource::ParentFiles);
                let paths = request.paths.iter()
                    .map(|path| container_path(|path| self.dependencies.file_exists(path, include_vanilla, include_parent, false), path))
                    .collect();

                ("", data_source, paths)
            }
            FileSource::AssemblyKit => return Err(ApiError::InvalidParams("Assembly Kit tables can't be extracted.".to_owned()).into()),
        };

        let extracted = self.extract_files(pack_key, &BTreeMap::from([(data_source, paths)]), &request.destination, request.as_tsv, options)?;
        Ok(FilesExtracted { extracted })
    }

    /// Returns the contents of a file of any source.
    ///
    /// # Arguments
    ///
    /// * `request` - The file, and how to return it.
    /// * `enable_esf_editor` - If ESF files are decoded.
    /// * `disable_uuid_regeneration` - If tables keep their GUID when encoded to return their bytes.
    ///
    /// # Errors
    ///
    /// Fails if the file doesn't exist, or can't be returned in the requested format.
    pub fn read_file(&mut self, request: &ReadFile, enable_esf_editor: bool, disable_uuid_regeneration: bool) -> Result<FileContents> {
        if request.format == ReadFormat::Raw {
            let compression_format = match request.file.source {
                FileSource::Pack(ref pack_key) => pack(&self.packs, pack_key)?.compression_format(),
                _ => CompressionFormat::None,
            };

            let extra_data = encode_extra_data(&self.game, compression_format, disable_uuid_regeneration);
            let file = writable_file(&mut self.packs, &mut self.dependencies, &request.file)?;
            file.load()?;

            let file_type = file.file_type();
            let bytes = match file.cached() {
                Ok(data) => data.to_vec(),
                Err(_) => file.encode(&extra_data, false, false, true)?
                    .ok_or_else(|| anyhow!("The file {} returned no data when encoded.", request.file.path))?,
            };

            return Ok(FileContents { file_type, contents: FileData::Raw { base64: STANDARD.encode(bytes) } });
        }

        let (pack_key, data_source) = pack_key_and_data_source(&request.file.source);
        let pack_key = pack_key.to_owned();
        let (decoded, file_type) = match self.decode_file(&pack_key, &request.file.path, data_source, enable_esf_editor)? {
            DecodedFile::Decoded(decoded, info) => (*decoded, *info.file_type()),
            DecodedFile::Notes(notes) => (RFileDecoded::Text(notes), FileType::Text),
            DecodedFile::Unsupported | DecodedFile::External => {
                return Err(ApiError::InvalidParams(format!("The file {} can't be decoded. Read it as raw instead.", request.file.path)).into());
            }
        };

        let contents = match request.format {
            ReadFormat::Text => match decoded {
                RFileDecoded::Text(text) | RFileDecoded::VMD(text) | RFileDecoded::WSModel(text) => FileData::Text { text: text.contents().to_owned() },
                _ => return Err(ApiError::InvalidParams(format!("The file {} is not a text file.", request.file.path)).into()),
            },
            ReadFormat::Decoded | ReadFormat::Raw => FileData::Decoded { data: serde_json::to_value(decoded)? },
        };

        Ok(FileContents { file_type, contents })
    }

    /// Replaces the contents of a file of an open pack.
    ///
    /// With raw contents, the file is created if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Fails if the file doesn't exist (except for raw contents), or if the contents don't fit the file.
    pub fn write_file(&mut self, request: &WriteFile) -> Result<()> {
        let pack = pack_mut(&mut self.packs, &request.pack)?;
        match request.contents {
            FileData::Raw { ref base64 } => {
                let bytes = STANDARD.decode(base64).map_err(|error| ApiError::InvalidParams(format!("Invalid base64: {error}")))?;
                let file_type = pack.files().get(&request.path).map_or(FileType::Unknown, |file| file.file_type());
                let mut file = RFile::new_from_vec(&bytes, file_type, 0, &request.path);
                if file_type == FileType::Unknown {
                    let _ = file.guess_file_type();
                }

                pack.insert(file)?;
            }
            FileData::Text { ref text } => {
                let file = pack.files_mut().get_mut(&request.path).ok_or_else(|| ApiError::FileNotFound(request.path.clone()))?;
                let _ = file.decode(&Some(DecodeableExtraData::default()), true, false);
                match file.decoded_mut() {
                    Ok(RFileDecoded::Text(data)) | Ok(RFileDecoded::VMD(data)) | Ok(RFileDecoded::WSModel(data)) => { data.set_contents(text.clone()); }
                    _ => return Err(ApiError::InvalidParams(format!("The file {} is not a text file.", request.path)).into()),
                }
            }
            FileData::Decoded { ref data } => {
                if !pack.has_file(&request.path) && request.path != RESERVED_NAME_NOTES {
                    return Err(ApiError::FileNotFound(request.path.clone()).into());
                }

                let decoded = serde_json::from_value::<RFileDecoded>(data.clone())
                    .map_err(|error| ApiError::InvalidParams(format!("Invalid decoded file: {error}")))?;
                self.save_file_from_view(&request.pack, &request.path, decoded)?;
            }
        }

        Ok(())
    }

    /// Returns the files inside an AnimPack of any source, sorted by path.
    pub fn list_animpack(&mut self, file: &FileRef) -> Result<FileList> {
        let (pack_key, data_source) = pack_key_and_data_source(&file.source);
        let pack_key = pack_key.to_owned();
        let files = match self.decode_file(&pack_key, &file.path, data_source, false)? {
            DecodedFile::Decoded(decoded, _) => match *decoded {
                RFileDecoded::AnimPack(anim_pack) => {
                    let mut files = anim_pack.files().values()
                        .map(|file| FileEntry { path: file.path_in_container_raw().to_owned(), file_type: file.file_type() })
                        .collect::<Vec<_>>();
                    files.sort_unstable_by(|a, b| a.path.cmp(&b.path));
                    files
                }
                _ => return Err(ApiError::InvalidParams(format!("The file {} is not an AnimPack.", file.path)).into()),
            },
            _ => return Err(ApiError::InvalidParams(format!("The file {} is not an AnimPack.", file.path)).into()),
        };

        Ok(FileList { total: files.len(), files, folders: vec![] })
    }

    /// Copies files of an open pack into an AnimPack of an open pack.
    pub fn add_to_animpack(&mut self, request: &AddToAnimPack) -> Result<FilesAdded> {
        let paths = self.pack_container_paths(&request.from_pack, &request.paths)?;
        let added = self.add_files_to_animpack(&request.from_pack, &request.pack, &request.animpack, &paths)?;
        Ok(FilesAdded { added: raw_paths(&added), ..FilesAdded::default() })
    }

    /// Copies files of an AnimPack of any source into an open pack.
    pub fn extract_from_animpack(&mut self, request: &ExtractFromAnimPack) -> Result<FilesAdded> {
        let paths = self.animpack_container_paths(&request.file, &request.paths)?;
        let (pack_key, data_source) = pack_key_and_data_source(&request.file.source);
        let pack_key = pack_key.to_owned();
        let added = self.add_files_from_animpack(&pack_key, &request.to_pack, data_source, &request.file.path, &paths)?;
        Ok(FilesAdded { added: raw_paths(&added), ..FilesAdded::default() })
    }

    /// Deletes files from an AnimPack of an open pack.
    pub fn delete_in_animpack(&mut self, request: &DeleteFromAnimPack) -> Result<()> {
        let file = FileRef { source: FileSource::Pack(request.pack.clone()), path: request.animpack.clone() };
        let paths = self.animpack_container_paths(&file, &request.paths)?;
        self.delete_from_animpack(&request.pack, &request.animpack, &paths)
    }

    /// Resolves paths inside an AnimPack into file or folder paths, depending on if there's a file at each one.
    fn animpack_container_paths(&mut self, file: &FileRef, paths: &[String]) -> Result<Vec<ContainerPath>> {
        let files = self.list_animpack(file)?.files.into_iter().map(|file| file.path).collect::<HashSet<_>>();
        Ok(paths.iter().map(|path| container_path(|path| files.contains(path), path)).collect())
    }

    /// Resolves paths of an open pack into file or folder paths, depending on if there's a file at each one.
    pub(super) fn pack_container_paths(&self, pack_key: &str, paths: &[String]) -> Result<Vec<ContainerPath>> {
        let pack = pack(&self.packs, pack_key)?;
        Ok(paths.iter().map(|path| container_path(|path| pack.has_file(path), path)).collect())
    }
}

/// Returns a file of an open pack, the game files or the parent packs, mutably.
///
/// # Errors
///
/// Fails if the file doesn't exist, or if it's an Assembly Kit table, which isn't stored as a file.
fn writable_file<'a>(packs: &'a mut BTreeMap<String, Pack>, dependencies: &'a mut Dependencies, file: &FileRef) -> Result<&'a mut RFile> {
    let not_found = || ApiError::FileNotFound(file.path.clone());
    match file.source {
        FileSource::Pack(ref pack_key) => Ok(pack_mut(packs, pack_key)?.files_mut().get_mut(&file.path).ok_or_else(not_found)?),
        FileSource::GameFiles => dependencies.file_mut(&file.path, true, false).map_err(|_| not_found().into()),
        FileSource::ParentFiles => dependencies.file_mut(&file.path, false, true).map_err(|_| not_found().into()),
        FileSource::AssemblyKit => Err(ApiError::InvalidParams("Assembly Kit tables aren't files. Read them with table.rows.".to_owned()).into()),
    }
}

/// Returns a path as a file path if `has_file` says there's a file at it, or as a folder path otherwise.
pub(super) fn container_path(has_file: impl Fn(&str) -> bool, path: &str) -> ContainerPath {
    if has_file(path) {
        ContainerPath::File(path.to_owned())
    } else {
        ContainerPath::Folder(path.trim_end_matches('/').to_owned())
    }
}

/// Resolves paths into container paths, splitting out the ones that match no file.
///
/// # Arguments
///
/// * `paths` - Paths to resolve.
/// * `resolve` - Turns a path into a file or folder path.
/// * `exists` - Returns if a container path matches any file.
///
/// # Returns
///
/// The container paths matching files, and the paths matching none.
fn split_found(paths: &[String], resolve: impl Fn(&str) -> ContainerPath, exists: impl Fn(&ContainerPath) -> bool) -> (Vec<ContainerPath>, Vec<String>) {
    let mut found = Vec::with_capacity(paths.len());
    let mut not_found = vec![];
    for path in paths {
        let container_path = resolve(path);
        if exists(&container_path) {
            found.push(container_path);
        } else {
            not_found.push(path.to_owned());
        }
    }

    (found, not_found)
}

/// Returns the key of the pack of a file source (empty for other sources), and its data source.
pub(super) fn pack_key_and_data_source(source: &FileSource) -> (&str, DataSource) {
    let pack_key = match source {
        FileSource::Pack(pack_key) => pack_key.as_str(),
        _ => "",
    };

    (pack_key, DataSource::from(source))
}

/// Returns the raw paths of container paths.
pub(super) fn raw_paths(paths: &[ContainerPath]) -> Vec<String> {
    paths.iter().map(|path| path.path_raw().to_owned()).collect()
}

/// Decodes a file, skipping the types the server doesn't decode.
fn decode(file: &mut RFile, game: &GameInfo, schema: &Option<Schema>, enable_esf_editor: bool) -> Result<DecodedFile> {
    let file_type = file.file_type();
    if UNDECODEABLE_FILE_TYPES.contains(&file_type) || (file_type == FileType::ESF && !enable_esf_editor) {
        return Ok(DecodedFile::Unsupported);
    }

    let mut extra_data = DecodeableExtraData::default();
    extra_data.set_schema(schema.as_ref());
    extra_data.set_game_info(Some(game));

    let decoded = file.decode(&Some(extra_data), true, true)?
        .ok_or_else(|| anyhow!("The file {} returned no data when decoded.", file.path_in_container_raw()))?;

    Ok(DecodedFile::Decoded(Box::new(decoded), RFileInfo::from(&*file)))
}

/// Decodes an AnimPack file, returning its decoded data.
fn decoded_anim_pack<'a>(file: &'a mut RFile, anim_pack_path: &str) -> Result<&'a mut AnimPack> {
    let _ = file.decode(&Some(DecodeableExtraData::default()), true, false);
    match file.decoded_mut() {
        Ok(RFileDecoded::AnimPack(anim_pack)) => Ok(anim_pack),
        Ok(decoded) => Err(anyhow!("We expected {} to be of type {} but found {}. This is either a bug or you did weird things with the game selected.", anim_pack_path, FileType::AnimPack, FileType::from(&*decoded))),
        Err(_) => Err(anyhow!("Failed to decode the file at the following path: {}", anim_pack_path)),
    }
}

/// Returns copies of the files of a pack at the provided paths, with their data loaded in memory.
fn loaded_files_by_paths(pack: &Pack, paths: &[ContainerPath]) -> Vec<RFile> {
    pack.files_by_paths(paths, false)
        .into_iter()
        .map(|file| {
            let mut file = file.clone();
            let _ = file.load();
            file
        })
        .collect()
}

/// Inserts files into a pack, returning the paths of the ones inserted.
fn insert_files(pack: &mut Pack, files: Vec<RFile>) -> Vec<ContainerPath> {
    files.into_iter()
        .filter_map(|file| pack.insert(file).ok().flatten())
        .collect()
}

/// Returns a path for a copy of a file, incrementing the trailing number of its name until it's free.
///
/// For example, "name.ext" becomes "name1.ext", and "name1.ext" becomes "name2.ext".
fn duplicate_path(pack: &Pack, path: &str) -> String {
    let (base, ext) = match path.rfind('.') {
        Some(dot_pos) => path.split_at(dot_pos),
        None => (path, ""),
    };

    let base_trimmed = base.trim_end_matches(|c: char| c.is_ascii_digit());
    let first_counter = base[base_trimmed.len()..].parse::<u32>().unwrap_or(0) + 1;

    (first_counter..)
        .map(|counter| format!("{}{}{}", base_trimmed, counter, ext))
        .find(|candidate| !pack.has_file(candidate))
        .expect("an unbounded range always finds a free name")
}

/// Expands the selected paths of a pack into entries to paste, one per file.
///
/// - For a selected file `a/b/c`, the base path is `a/b` (parent folder), so pasting gives just `c`.
/// - For a selected folder `a/b`, the base path is `a` (parent of folder), so pasting preserves `b/...`.
fn paste_entries_from_paths(pack: &Pack, paths: &[ContainerPath], pack_key: &str) -> Vec<PasteEntry> {
    paths.iter()
        .flat_map(|path| {
            let base_path = path.path_raw().rfind('/')
                .map(|pos| path.path_raw()[..pos].to_string())
                .unwrap_or_default();

            pack.files_by_paths(from_ref(path), false)
                .into_iter()
                .map(move |file| PasteEntry {
                    file_path: file.path_in_container_raw().to_string(),
                    base_path: base_path.clone(),
                    source_pack_key: pack_key.to_string(),
                })
        })
        .collect()
}

/// Imports a table from the Assembly Kit as a new file, or `None` if it can't be imported.
///
/// CEO tables go under a `ceo_` prefixed folder.
fn import_asskit_table(dependencies: &Dependencies, schema: &Schema, table_name: &str, path: &str) -> Option<RFile> {
    let table = dependencies.import_from_ak(table_name, schema).ok()?;
    let path = if table_name.starts_with("ceo") {
        format!("ceo_{path}")
    } else {
        path.to_owned()
    };

    Some(RFile::new_from_decoded(&RFileDecoded::DB(table), 0, &path))
}

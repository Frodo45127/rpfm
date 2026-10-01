//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tool operations: optimizer, map packing, startpos and CEO building, translations, MyMods,
//! animation ids, glTF export and plugin scripts.

use anyhow::{anyhow, Result};

use serde_json::Value;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env::temp_dir;
use std::fs::{DirBuilder, File};
use std::io::{BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use rpfm_extensions::dependencies::{GAMES_NEEDING_VICTORY_OBJECTIVES, VICTORY_OBJECTIVES_FILE_NAME};
use rpfm_extensions::gltf::{gltf_from_rigid, save_gltf_to_disk};
use rpfm_extensions::optimizer::{OptimizableContainer, OptimizerOptions};
use rpfm_extensions::translator::PackTranslation;

use rpfm_ipc::messages::CeoEntryData;
use rpfm_ipc::settings_keys::ASSEMBLY_KIT_SUFFIX;

use rpfm_lib::files::{Container, ContainerPath, RFile, RFileDecoded, rigidmodel::RigidModel};
use rpfm_lib::games::supported_games::KEY_THREE_KINGDOMS;
use rpfm_lib::integrations::git::GitIntegration;

use rpfm_telemetry::{error, info, warn};

use crate::ceo_builder::{build_ceo, build_ceo_entries, build_ceo_post, get_trait_ceos};
use crate::settings::{lua_autogen_game_path, scripts_path, translations_local_path, translations_remote_path, Settings};

use rpfm_ipc::api::ApiError;
use rpfm_ipc::api::files::FileRef;
use rpfm_ipc::api::tables::FilesEdited;
use rpfm_ipc::api::translations::{DEFAULT_TRANSLATIONS_LIMIT, ListTranslations, TranslationEntry, TranslationFilter, Translations};
use rpfm_ipc::api::tools::{FilePaths, FilesChanged, InitMyMod, MyModCreated, SiegeAiPatched, StartStartpos, StartposCampaigns};

use super::{DecodedFile, ExtractOptions, SessionState, encode_extra_data, loaded_schema, pack, pack_mut};
use super::files::{legacy_source, raw_paths};

/// Filename prefix for community-maintained vanilla loc fix TSVs in the
/// [Total War Translation Hub][tlh] repo (e.g. `vanilla_fixes_es.tsv`).
/// Each one carries fixes for vanilla loc bugs in a specific language;
/// the suffix is the language code.
///
/// [tlh]: https://github.com/Frodo45127/total_war_translation_hub
const VANILLA_FIXES_NAME: &str = "vanilla_fixes_";

/// Sub-startpos each Three Kingdoms campaign has.
const THREE_KINGDOMS_SUB_STARTPOS: [&str; 2] = ["historical", "romance"];

/// Pack setting with the campaign the last startpos of a pack was built for.
const STARTPOS_LAST_CAMPAIGN_SETTING: &str = "starpos_last_campaign";

/// A startpos build started, waiting for the game to build it.
#[derive(Debug, Clone)]
pub(super) struct PendingStartpos {
    pack: String,
    campaign: String,
    process_hlp_spd_data: bool,
}

/// Editor support to set up in a new MyMod folder.
#[derive(Debug, Clone, Default)]
pub struct MyModOptions {

    /// If a Sublime Text project is created.
    pub sublime_support: bool,

    /// If a VSCode config is created.
    pub vscode_support: bool,

    /// If set, a git repository is created with this `.gitignore`.
    pub gitignore: Option<String>,
}

impl SessionState {

    /// Removes unchanged and duplicated data from a pack.
    ///
    /// # Returns
    ///
    /// The paths deleted and the paths added by the optimizer.
    pub fn optimize_pack(&mut self, pack_key: &str, options: &OptimizerOptions) -> Result<(HashSet<String>, HashSet<String>)> {
        let pack = pack_mut(&mut self.packs, pack_key)?;
        let schema = loaded_schema(&self.schema)?;
        Ok(pack.optimize(None, &mut self.dependencies, schema, &self.game, options)?)
    }

    /// Patches the SiegeAI of the siege maps of a pack, for Warhammer games.
    ///
    /// # Returns
    ///
    /// A message with the result, and the paths deleted.
    pub fn patch_siege_ai(&mut self, pack_key: &str) -> Result<(String, Vec<ContainerPath>)> {
        Ok(pack_mut(&mut self.packs, pack_key)?.patch_siege_ai()?)
    }

    /// Packs map tiles and tile maps into a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to add the maps to.
    /// * `tile_maps` - Paths on disk of the tile maps.
    /// * `tiles` - Paths on disk of the tiles, and their subfolder in the pack.
    /// * `options` - Options to optimize the added files.
    ///
    /// # Returns
    ///
    /// The paths added and the paths deleted.
    pub fn pack_map(&mut self, pack_key: &str, tile_maps: Vec<PathBuf>, tiles: Vec<(PathBuf, String)>, options: OptimizerOptions) -> Result<(Vec<ContainerPath>, Vec<ContainerPath>)> {
        let schema = loaded_schema(&self.schema)?;
        Ok(self.dependencies.add_tile_maps_and_tiles(&mut self.packs, Some(pack_key), &self.game, schema, options, tile_maps, tiles)?)
    }

    /// Adds the loc entries missing from the tables of the open packs.
    ///
    /// # Returns
    ///
    /// The paths of the files with the new entries.
    pub fn generate_missing_loc_data(&mut self) -> Result<Vec<ContainerPath>> {
        Ok(self.dependencies.generate_missing_loc_data(&mut self.packs)?)
    }

    /// Creates the folder of a new MyMod, with the editor configs requested.
    ///
    /// # Arguments
    ///
    /// * `mymod_base_path` - Folder containing the MyMods of all games.
    /// * `mod_game` - Folder name of the game of the MyMod.
    /// * `mod_name` - Name of the MyMod.
    /// * `options` - Editor support to set up.
    ///
    /// # Returns
    ///
    /// The path of the MyMod's pack.
    pub fn initialize_mymod_folder(&self, mymod_base_path: &Path, mod_game: &str, mod_name: &str, options: &MyModOptions) -> Result<PathBuf> {
        if !mymod_base_path.is_dir() {
            return Err(anyhow!("MyMod path is not configured. Configure it in the settings and try again."));
        }

        let mut mymod_path = mymod_base_path.join(mod_game);
        DirBuilder::new().recursive(true).create(&mymod_path)
            .map_err(|error| anyhow!("Error while creating the MyMod's Game folder: {}.", error))?;

        // We need to create another folder inside the game's folder with the name of the new "MyMod", to store extracted files.
        mymod_path.push(mod_name);
        DirBuilder::new().recursive(true).create(&mymod_path)
            .map_err(|error| anyhow!("Error while creating the MyMod's Assets folder: {}.", error))?;

        if let Some(ref gitignore) = options.gitignore {
            let git_integration = GitIntegration::new(&mymod_path, "", "", "");
            git_integration.init()?;
            git_integration.add_gitignore(gitignore)?;
        }

        // If the tw_autogen supports the game, create the vscode and sublime configs for lua mods.
        if options.sublime_support || options.vscode_support {
            if let Ok(lua_autogen_folder) = lua_autogen_game_path(&self.game) {
                let lua_autogen_folder = lua_autogen_folder.to_string_lossy().replace('\\', "/");
                write_lua_editor_configs(&mymod_path, mod_name, &lua_autogen_folder, options)?;
            }
        }

        mymod_path.set_extension("pack");
        Ok(mymod_path)
    }

    /// Returns the translation of a pack to a language, with the vanilla texts and their community fixes as base.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to translate.
    /// * `src_lang` - Language the pack's texts are written in.
    /// * `language` - Language to translate the pack to.
    pub fn pack_translation(&self, pack_key: &str, src_lang: &str, language: &str) -> Result<PackTranslation> {
        let game_key = self.game.key();
        let local_path = translations_local_path()?;
        let remote_path = translations_remote_path()?;

        let mut base_english = HashMap::new();
        let mut base_local_fixes = HashMap::new();

        // Vanilla texts generated from the game files live in the local folder. The Hub only ships English ones.
        let vanilla_loc_name = PackTranslation::vanilla_loc_file_name(src_lang);
        let vanilla_loc = [&local_path, &remote_path].iter()
            .find_map(|path| RFile::tsv_import_from_path(&path.join(game_key).join(&vanilla_loc_name), &None).ok());

        if let Some(mut vanilla_loc) = vanilla_loc {
            let _ = vanilla_loc.guess_file_type();
            if let Ok(RFileDecoded::Loc(vanilla_loc)) = vanilla_loc.decoded() {

                // If we have a fixes file for the vanilla translation, apply it before everything else.
                let fixes_loc_path = remote_path.join(format!("{}/{}{}.tsv", game_key, VANILLA_FIXES_NAME, language));
                if let Ok(mut fixes_loc) = RFile::tsv_import_from_path(&fixes_loc_path, &None) {
                    let _ = fixes_loc.guess_file_type();
                    if let Ok(RFileDecoded::Loc(fixes_loc)) = fixes_loc.decoded() {
                        base_local_fixes.extend(fixes_loc.data().iter().map(|row| (row[0].data_to_string().to_string(), row[1].data_to_string().to_string())));
                    }
                }

                base_english.extend(vanilla_loc.data().iter().map(|row| (row[0].data_to_string().to_string(), row[1].data_to_string().to_string())));
            }
        }

        let pack = pack(&self.packs, pack_key)?;
        Ok(PackTranslation::new(&[local_path, remote_path], pack, game_key, src_lang, language, &self.dependencies, &base_english, &base_local_fixes)?)
    }

    /// Generates the vanilla texts of a language from the game's locale packs.
    ///
    /// # Arguments
    ///
    /// * `src_lang` - Language to generate the texts of.
    /// * `settings` - Settings, to find the game's install folder.
    ///
    /// # Returns
    ///
    /// If vanilla texts for that language are available, generated now or before.
    pub fn generate_vanilla_translation_source(&self, src_lang: &str, settings: &Settings) -> Result<bool> {
        let local_path = translations_local_path()?;
        let remote_path = translations_remote_path()?;

        let game_path = settings.path_buf(self.game.key());
        if let Err(error) = PackTranslation::generate_vanilla_loc(&self.game, &game_path, src_lang, &local_path.join(self.game.key())) {
            warn!("Failed to generate the vanilla {src_lang} texts from the game files: {error}");
        }

        let vanilla_loc_name = PackTranslation::vanilla_loc_file_name(src_lang);
        Ok([local_path, remote_path].iter().any(|path| path.join(self.game.key()).join(&vanilla_loc_name).is_file()))
    }

    /// Checks that a pack has the victory objectives file, for the games that need it to build a startpos.
    pub fn check_starpos_victory_conditions(&self, pack_key: &str) -> Result<()> {
        let pack = pack(&self.packs, pack_key)?;
        if GAMES_NEEDING_VICTORY_OBJECTIVES.contains(&self.game.key()) && pack.file(VICTORY_OBJECTIVES_FILE_NAME, false).is_none() {
            return Err(anyhow!("Missing \"db/victory_objectives.txt\" file. Processing the startpos without this file will result in issues in campaign. Add the file to the pack and try again."));
        }

        Ok(())
    }

    /// Prepares the Assembly Kit to build the startpos of a campaign, with the tables of a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack with the startpos tables.
    /// * `campaign_id` - Campaign to build the startpos for.
    /// * `process_hlp_spd_data` - If the HLP and SPD data are also built.
    /// * `settings` - Settings, to find the game's install folder and Assembly Kit.
    pub fn build_starpos(&mut self, pack_key: &str, campaign_id: &str, process_hlp_spd_data: bool, settings: &Settings) -> Result<()> {
        let game_path = settings.path_buf(self.game.key());
        let asskit_path = Some(settings.path_buf(&format!("{}{}", self.game.key(), ASSEMBLY_KIT_SUFFIX)));

        // 3K needs two passes, one per startpos, and there are two per campaign. The HLP and SPD data is only built once.
        if self.game.key() == KEY_THREE_KINGDOMS {
            for (index, sub_start_pos) in THREE_KINGDOMS_SUB_STARTPOS.iter().enumerate() {
                self.dependencies.build_starpos_pre(&mut self.packs, Some(pack_key), &self.game, &game_path, asskit_path.clone(), campaign_id, process_hlp_spd_data && index == 0, sub_start_pos)?;
            }
        } else {
            self.dependencies.build_starpos_pre(&mut self.packs, Some(pack_key), &self.game, &game_path, asskit_path, campaign_id, process_hlp_spd_data, "")?;
        }

        Ok(())
    }

    /// Imports the startpos built by the Assembly Kit into a pack, or cleans up the files used to build it.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to import the startpos into.
    /// * `campaign_id` - Campaign the startpos was built for.
    /// * `process_hlp_spd_data` - If the HLP and SPD data were also built.
    /// * `cleanup` - If this only cleans up the build files, instead of importing the startpos.
    /// * `settings` - Settings, to find the game's install folder and Assembly Kit.
    ///
    /// # Returns
    ///
    /// The paths added to the pack.
    pub fn build_starpos_post(&mut self, pack_key: &str, campaign_id: &str, process_hlp_spd_data: bool, cleanup: bool, settings: &Settings) -> Result<Vec<ContainerPath>> {
        let game_path = settings.path_buf(self.game.key());
        let asskit_path = Some(settings.path_buf(&format!("{}{}", self.game.key(), ASSEMBLY_KIT_SUFFIX)));
        let sub_start_pos = if self.game.key() == KEY_THREE_KINGDOMS {
            THREE_KINGDOMS_SUB_STARTPOS.iter().map(|name| name.to_string()).collect()
        } else {
            vec![]
        };

        Ok(self.dependencies.build_starpos_post(&mut self.packs, Some(pack_key), &self.game, &game_path, asskit_path, campaign_id, process_hlp_spd_data, cleanup, &sub_start_pos)?)
    }

    /// Builds `ceo_data.ccd` in the Assembly Kit from the CEO tables of a pack. See [`build_ceo`].
    pub fn build_ceo(&mut self, pack_key: &str, akit_path: &Path, bob_exe_path: &Path) -> Result<()> {
        info!("[BuildCeo] handler entered: pack={pack_key}, akit={}, bob={}", akit_path.display(), bob_exe_path.display());
        let pack = pack_mut(&mut self.packs, pack_key)?;
        build_ceo(pack, &self.schema, akit_path, bob_exe_path)
    }

    /// Imports the `ceo_data.ccd` built by BOB into a pack.
    ///
    /// # Returns
    ///
    /// The paths added to the pack.
    pub fn build_ceo_post(&mut self, pack_key: &str, akit_path: &str) -> Result<Vec<ContainerPath>> {
        build_ceo_post(pack_mut(&mut self.packs, pack_key)?, akit_path)
    }

    /// Adds CEO entries (armour, career, traits and their locs) to a pack.
    ///
    /// # Returns
    ///
    /// The paths added to the pack.
    pub fn build_ceo_entries(&mut self, pack_key: &str, entries: &[CeoEntryData]) -> Result<Vec<ContainerPath>> {
        let schema = loaded_schema(&self.schema)?;
        build_ceo_entries(pack_mut(&mut self.packs, pack_key)?, schema, entries)
    }

    /// Returns the key and name of the trait CEOs in the Assembly Kit.
    pub fn trait_ceos(&self) -> Vec<(String, String)> {
        get_trait_ceos(&self.dependencies)
    }

    /// Offsets the animation ids of a pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack to edit.
    /// * `starting_id` - Only ids from this one are changed.
    /// * `offset` - Amount to add to each id.
    ///
    /// # Returns
    ///
    /// The paths of the edited files.
    pub fn update_anim_ids(&mut self, pack_key: &str, starting_id: i32, offset: i32) -> Result<Vec<ContainerPath>> {
        Ok(pack_mut(&mut self.packs, pack_key)?.update_anim_ids(&self.game, starting_id, offset)?)
    }

    /// Exports a RigidModel to a glTF file.
    pub fn export_rigid_to_gltf(&mut self, rigid_model: &RigidModel, path: &Path) -> Result<()> {
        let gltf = gltf_from_rigid(rigid_model, &mut self.dependencies)?;
        save_gltf_to_disk(&gltf, path)?;
        Ok(())
    }

    /// Runs a plugin script over files of a pack, reading back into the pack the files it changes.
    ///
    /// The files are extracted to a temp folder, keeping their paths in the pack, and passed to the script as arguments.
    /// Tables are extracted as TSV. Files the script deletes are left untouched in the pack.
    ///
    /// # Arguments
    ///
    /// * `pack_key` - Key of the pack with the files.
    /// * `script_path` - Path of the script.
    /// * `container_paths` - Paths of the files and folders to pass to the script.
    /// * `options` - Options for extracting the files.
    ///
    /// # Returns
    ///
    /// The paths read back into the pack, and an error message if the script failed.
    pub fn run_plugin_script(&mut self, pack_key: &str, script_path: &Path, container_paths: &[ContainerPath], options: ExtractOptions) -> Result<(Vec<ContainerPath>, Option<String>)> {
        let interpreter = plugin_script_interpreter(script_path)
            .ok_or_else(|| anyhow!("Unsupported plugin script type: {}", script_path.display()))?;

        let pack = pack_mut(&mut self.packs, pack_key)?;
        let base_folder = temp_dir().join("rpfm_plugins").join(pack.disk_file_name());
        let _ = std::fs::remove_dir_all(&base_folder);

        let extra_data = encode_extra_data(&self.game, pack.compression_format(), options.disable_uuid_regeneration);
        let mut extracted_paths = vec![];
        for container_path in container_paths {
            let mut paths = pack.extract(container_path.clone(), &base_folder, true, &self.schema, false, options.tsv_keys_first, &extra_data)
                .map_err(|error| anyhow!("Error extracting files for the plugin script: {}", error))?;
            extracted_paths.append(&mut paths);
        }

        let output = std::process::Command::new(interpreter)
            .arg(script_path)
            .args(&extracted_paths)
            .current_dir(&base_folder)
            .output()
            .map_err(|error| {
                error!("Failed to run the plugin script {}: {}", script_path.display(), error);
                anyhow!("Failed to run the plugin script: {}", error)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Record the run so users can debug their scripts: a `last_run.log` in the
        // scripts folder (overwritten each run), plus the standard terminal logger.
        let report = format!("Script: {}\nStatus: {}\n\n--- stdout ---\n{}\n--- stderr ---\n{}\n", script_path.display(), output.status, stdout, stderr);
        if let Ok(folder) = scripts_path() {
            let _ = std::fs::write(folder.join("last_run.log"), report.as_bytes());
        }

        info!("Plugin script {} finished with {}.", script_path.display(), output.status);
        if !stderr.trim().is_empty() {
            warn!("Plugin script stderr:\n{}", stderr.trim());
        }

        let message = if output.status.success() {
            None
        } else {
            Some(format!("The plugin script finished with errors. See last_run.log in the scripts folder.\n\n{}", stderr.trim()))
        };

        let mut reimported_paths = vec![];
        for disk_path in extracted_paths.iter().filter(|path| path.is_file()) {

            // TSV-exported DB/Loc files have a `.tsv` suffix appended to their in-pack name;
            // strip it to find the real file when the direct path doesn't match anything.
            let direct_path = container_path_from_disk_path(disk_path, &base_folder);
            let container_path = match direct_path.strip_suffix(".tsv") {
                Some(stripped) if pack.file_mut(&direct_path, false).is_none() => stripped.to_owned(),
                _ => direct_path,
            };

            if let Some(file) = pack.file_mut(&container_path, false) {
                if file.encode_from_external_data(&self.schema, disk_path).is_ok() {
                    reimported_paths.push(ContainerPath::File(container_path));
                }
            }
        }

        let _ = std::fs::remove_dir_all(&base_folder);
        Ok((reimported_paths, message))
    }
}

impl SessionState {

    /// Removes unneeded data from a pack, returning the paths changed.
    pub fn optimize_pack_files(&mut self, pack_key: &str, options: &OptimizerOptions) -> Result<FilesChanged> {
        let (deleted, added) = self.optimize_pack(pack_key, options)?;
        Ok(FilesChanged { added: sorted(added), deleted: sorted(deleted) })
    }

    /// Patches the siege maps of a pack, returning what was patched.
    pub fn patch_siege_ai_files(&mut self, pack_key: &str) -> Result<SiegeAiPatched> {
        let (message, deleted) = self.patch_siege_ai(pack_key)?;
        Ok(SiegeAiPatched { message, deleted: raw_paths(&deleted) })
    }

    /// Packs map tiles and tile maps into a pack, returning the paths changed.
    pub fn pack_map_files(&mut self, pack_key: &str, tile_maps: Vec<PathBuf>, tiles: Vec<(PathBuf, String)>, options: OptimizerOptions) -> Result<FilesChanged> {
        let (added, deleted) = self.pack_map(pack_key, tile_maps, tiles, options)?;
        Ok(FilesChanged { added: raw_paths(&added), deleted: raw_paths(&deleted) })
    }

    /// Adds the missing loc entries of the tables of the open packs, returning the paths edited.
    pub fn generate_missing_locs(&mut self) -> Result<FilesEdited> {
        Ok(FilesEdited { edited: raw_paths(&self.generate_missing_loc_data()?) })
    }

    /// Offsets the animation ids of a pack, returning the paths edited.
    pub fn update_anim_id_files(&mut self, pack_key: &str, starting_id: i32, offset: i32) -> Result<FilesEdited> {
        Ok(FilesEdited { edited: raw_paths(&self.update_anim_ids(pack_key, starting_id, offset)?) })
    }

    /// Returns the sorted paths of the animations using a skeleton.
    pub fn anims_by_skeleton(&mut self, skeleton: &str) -> FilePaths {
        FilePaths { paths: sorted(self.anim_paths_by_skeleton(skeleton)) }
    }

    /// Exports a RigidModel from any source to a glTF file.
    ///
    /// # Errors
    ///
    /// Fails if the file is not a RigidModel, or if it can't be exported.
    pub fn export_gltf(&mut self, file: &FileRef, destination: &Path) -> Result<()> {
        let (pack_key, data_source) = legacy_source(&file.source);
        let pack_key = pack_key.to_owned();
        match self.decode_file(&pack_key, &file.path, data_source, false)? {
            DecodedFile::Decoded(decoded, _) => match *decoded {
                RFileDecoded::RigidModel(rigid_model) => self.export_rigid_to_gltf(&rigid_model, destination),
                _ => Err(ApiError::InvalidParams(format!("The file {} is not a RigidModel.", file.path)).into()),
            },
            _ => Err(ApiError::InvalidParams(format!("The file {} is not a RigidModel.", file.path)).into()),
        }
    }

    /// Creates the folder of a new MyMod.
    ///
    /// # Arguments
    ///
    /// * `mymod_base_path` - Folder containing the MyMods of all games.
    /// * `request` - The MyMod to create.
    pub fn init_mymod(&self, mymod_base_path: &Path, request: &InitMyMod) -> Result<MyModCreated> {
        let options = MyModOptions {
            sublime_support: request.sublime_support,
            vscode_support: request.vscode_support,
            gitignore: request.gitignore.clone(),
        };

        Ok(MyModCreated { pack_path: self.initialize_mymod_folder(mymod_base_path, &request.game_folder, &request.name, &options)? })
    }

    /// Returns a page of the translation of the texts of a pack, sorted by key.
    pub fn list_translations(&self, request: &ListTranslations) -> Result<Translations> {
        let translation = self.pack_translation(&request.pack, &request.source_language, &request.language)?;
        let mut entries = translation.translations().iter()
            .filter(|(_, entry)| match request.filter {
                TranslationFilter::All => true,
                TranslationFilter::Untranslated => entry.dst().is_empty() && !*entry.rem(),
                TranslationFilter::NeedsReview => *entry.retr() || *entry.aut(),
            })
            .map(|(key, entry)| TranslationEntry {
                key: key.clone(),
                source: entry.src().clone(),
                translation: entry.dst().clone(),
                needs_retranslation: *entry.retr(),
                automatic: *entry.aut(),
                removed: *entry.rem(),
            })
            .collect::<Vec<_>>();
        entries.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        let total = entries.len();
        let entries = entries.into_iter()
            .skip(request.offset)
            .take(request.limit.unwrap_or(DEFAULT_TRANSLATIONS_LIMIT))
            .collect();

        Ok(Translations { entries, total })
    }

    /// Returns the campaigns a startpos can be built for, and the one the last startpos of a pack was built for.
    pub fn startpos_campaigns(&self, pack_key: &str) -> Result<StartposCampaigns> {
        let last_used = pack(&self.packs, pack_key)?.settings().settings_text().get(STARTPOS_LAST_CAMPAIGN_SETTING).cloned();
        let campaigns = sorted(self.column_values("campaigns_tables", "campaign_name", true, true));
        Ok(StartposCampaigns { campaigns, last_used })
    }

    /// Starts building a startpos: checks the pack, prepares the Assembly Kit and launches the game.
    ///
    /// The campaign is saved in the pack's settings, to suggest it next time.
    pub fn start_startpos(&mut self, request: &StartStartpos, settings: &Settings) -> Result<()> {
        self.check_starpos_victory_conditions(&request.pack)?;
        self.build_starpos(&request.pack, &request.campaign, request.process_hlp_spd_data, settings)?;

        pack_mut(&mut self.packs, &request.pack)?.settings_mut().settings_text_mut().insert(STARTPOS_LAST_CAMPAIGN_SETTING.to_owned(), request.campaign.clone());
        self.pending_startpos = Some(PendingStartpos {
            pack: request.pack.clone(),
            campaign: request.campaign.clone(),
            process_hlp_spd_data: request.process_hlp_spd_data,
        });

        Ok(())
    }

    /// Finishes building a startpos: imports it into its pack, or cancels the build, and cleans up the build files.
    ///
    /// # Errors
    ///
    /// Fails if there's no build started.
    pub fn finish_startpos(&mut self, cancel: bool, settings: &Settings) -> Result<FilesEdited> {
        let pending = self.pending_startpos.take()
            .ok_or_else(|| ApiError::InvalidParams("There's no startpos build started. Call startpos.start first.".to_owned()))?;

        let added = self.build_starpos_post(&pending.pack, &pending.campaign, pending.process_hlp_spd_data, cancel, settings)?;
        Ok(FilesEdited { edited: raw_paths(&added) })
    }
}

/// Returns the optimizer options of the settings, by name.
pub fn optimizer_option_values(options: &OptimizerOptions) -> BTreeMap<String, bool> {
    match serde_json::to_value(options) {
        Ok(Value::Object(values)) => values.into_iter().filter_map(|(name, value)| value.as_bool().map(|value| (name, value))).collect(),
        _ => BTreeMap::new(),
    }
}

/// Returns optimizer options with some of them changed, by name.
///
/// # Errors
///
/// Fails if a name isn't an optimizer option.
pub fn optimizer_options_with(base: &OptimizerOptions, changes: &BTreeMap<String, bool>) -> Result<OptimizerOptions, ApiError> {

    // The options are set by name through serde, as the optimizer has a field per option.
    let mut values = optimizer_option_values(base);
    for (name, value) in changes {
        match values.get_mut(name) {
            Some(option) => *option = *value,
            None => return Err(ApiError::InvalidParams(format!("Unknown optimizer option: {name}. Valid ones: {}.", values.keys().cloned().collect::<Vec<_>>().join(", ")))),
        }
    }

    serde_json::from_value(serde_json::to_value(values).map_err(|error| ApiError::Internal(error.to_string()))?)
        .map_err(|error| ApiError::Internal(error.to_string()))
}

/// Returns the items of a collection, sorted.
fn sorted(items: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut items = items.into_iter().collect::<Vec<_>>();
    items.sort_unstable();
    items
}

/// Returns the paths of the plugin scripts in the config's scripts folder.
pub fn plugin_scripts() -> Result<Vec<String>> {
    let mut scripts = std::fs::read_dir(scripts_path()?)
        .map(|entries| entries.flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && plugin_script_interpreter(path).is_some())
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>())
        .unwrap_or_default();

    scripts.sort();
    Ok(scripts)
}

/// Returns the interpreter command for a plugin script, based on its extension.
///
/// Returns `None` for unsupported extensions, which is also how we filter the scripts folder.
fn plugin_script_interpreter(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("py") => Some("python"),
        Some("lua") => Some("lua"),
        _ => None,
    }
}

/// Rebuilds the in-pack container path of a file extracted under `base_folder`.
///
/// The extraction keeps the in-pack structure, so the container path is just the file's path
/// relative to `base_folder` with forward slashes (the separator container paths use).
fn container_path_from_disk_path(disk_path: &Path, base_folder: &Path) -> String {
    disk_path.strip_prefix(base_folder)
        .unwrap_or(disk_path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Writes the editor configs for Lua scripting in a MyMod folder.
///
/// # Arguments
///
/// * `mymod_path` - Folder of the MyMod.
/// * `mod_name` - Name of the MyMod.
/// * `lua_autogen_folder` - Folder with the Lua type definitions of the game, with forward slashes.
/// * `options` - Editors to set up.
fn write_lua_editor_configs(mymod_path: &Path, mod_name: &str, lua_autogen_folder: &str, options: &MyModOptions) -> Result<()> {
    if options.vscode_support {
        let vscode_config_path = mymod_path.join(".vscode");
        DirBuilder::new().recursive(true).create(&vscode_config_path)
            .map_err(|error| anyhow!("Error while creating the VSCode Config folder: {}.", error))?;

        if let Ok(file) = File::create(vscode_config_path.join("extensions.json")) {
            let mut file = BufWriter::new(file);
            let _ = file.write_all("
{
    \"recommendations\": [
        \"sumneko.lua\",
        \"formulahendry.code-runner\"
    ],
}".as_bytes());
        }
    }

    if options.sublime_support {
        if let Ok(file) = File::create(mymod_path.join(format!("{mod_name}.sublime-project"))) {
            let mut file = BufWriter::new(file);
            let _ = file.write_all("
{
    \"folders\":
    [
        {
            \"path\": \".\"
        }
    ]
}".as_bytes());
        }
    }

    // Generic lua support.
    if let Ok(file) = File::create(mymod_path.join(".luarc.json")) {
        let mut file = BufWriter::new(file);
        let _ = file.write_all(format!("
{{
    \"workspace.library\": [
        \"{lua_autogen_folder}/global/\",
        \"{lua_autogen_folder}/campaign/\",
        \"{lua_autogen_folder}/frontend/\",
        \"{lua_autogen_folder}/battle/\"
    ],
    \"runtime.version\": \"Lua 5.1\",
    \"completion.autoRequire\": false,
    \"workspace.preloadFileSize\": 1500,
    \"workspace.ignoreSubmodules\": false,
    \"diagnostics.workspaceDelay\": 500,
    \"diagnostics.workspaceRate\": 40,
    \"diagnostics.disable\": [
        \"lowercase-global\",
        \"trailing-space\"
    ],
    \"hint.setType\": true,
    \"workspace.ignoreDir\": [
        \".vscode\",
        \".git\"
    ]
}}").as_bytes());
    }

    Ok(())
}

//-------------------------------------------------------------------------------//
//                                   Tests
//-------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimizer_options_change_only_the_named_ones() {
        let base = OptimizerOptions::default();
        let base_values = optimizer_option_values(&base);
        assert!(base_values.contains_key("table_remove_itm_entries"));

        let flipped = !base_values["table_remove_itm_entries"];
        let changed = optimizer_options_with(&base, &BTreeMap::from([("table_remove_itm_entries".to_owned(), flipped)])).unwrap();
        let changed_values = optimizer_option_values(&changed);

        assert_eq!(changed_values["table_remove_itm_entries"], flipped);
        assert!(changed_values.iter().filter(|(name, _)| *name != "table_remove_itm_entries").all(|(name, value)| base_values[name] == *value));

        assert!(matches!(optimizer_options_with(&base, &BTreeMap::from([("nope".to_owned(), true)])), Err(ApiError::InvalidParams(_))));
    }
}

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the `GlobalSearch`.

This module contains the code needed to get a `GlobalSearch` over an entire `PackFile`.
!*/

use getset::*;
use regex::{RegexBuilder, Regex};
use rayon::prelude::*;

use rpfm_lib::error::{Result, RLibError};
use rpfm_lib::files::{Container, ContainerPath, FileType, pack::Pack, RFile, RFileDecoded};
use rpfm_lib::games::{GameInfo, VanillaDBTableNameLogic};
use rpfm_lib::schema::Schema;

use crate::dependencies::Dependencies;

//use self::anim::AnimMatches;
//use self::anim_fragment::AnimFragmentMatches;
//use self::anim_pack::AnimPackMatches;
//use self::anims_table::AnimsTableMatches;
use self::atlas::AtlasMatches;
//use self::audio::AudioMatches;
//use self::bmd::BmdMatches;
//use self::esf::EsfMatches;
//use self::group_formations::GroupFormationsMatches;
//use self::image::ImageMatches;
//use self::matched_combat::MatchedCombatMatches;
//use self::pack::PackMatches;
use self::portrait_settings::PortraitSettingsMatches;
use self::rigid_model::RigidModelMatches;
//use self::sound_bank::SoundBankMatches;
use self::table::TableMatches;
use self::text::TextMatches;
//use self::uic::UicMatches;
use self::unit_variant::UnitVariantMatches;
use self::unknown::UnknownMatches;
//use self::video::VideoMatches;
use self::schema::SchemaMatches;

//pub mod anim;
//pub mod anim_fragment;
//pub mod anim_pack;
//pub mod anims_table;
pub mod atlas;
//pub mod audio;
//pub mod bmd;
//pub mod esf;
//pub mod group_formations;
//pub mod image;
//pub mod matched_combat;
//pub mod pack;
pub mod portrait_settings;
pub mod rigid_model;
//pub mod sound_bank;
pub mod table;
pub mod text;
//pub mod uic;
pub mod unit_variant;
pub mod unknown;
//pub mod video;
pub mod schema;

//-------------------------------------------------------------------------------//
//                             Trait definitions
//-------------------------------------------------------------------------------//

/// This trait marks an struct (mainly structs representing decoded files) as `Searchable`, meaning they can be scanned for specific string matches.
pub trait Searchable {
    type SearchMatches;

    /// This function performs a search over a Searchable type, and returns the results.
    fn search(&self, file_path: &str, pattern_to_search: &str, case_sensitive: bool, matching_mode: &MatchingMode) -> Self::SearchMatches;
}

/// This trait marks a Searchable struct as `Replaceable`, meaning their matches can be replaced.
pub trait Replaceable: Searchable {

    /// This function performs a replace over search matches, returning true if the replacement was done.
    ///
    /// Replacements can fail due to outdated search matches or if the replacement is the same as the search match.
    fn replace(&mut self, pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_matches: &Self::SearchMatches) -> bool;
}

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the information needed to perform a global search, and the results of said search.
#[derive(Default, Debug, Clone, Getters, MutGetters, Setters)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
pub struct GlobalSearch {

    /// Pattern to search.
    pattern: String,

    /// Pattern to use when replacing. This is a hard pattern, which means regex is not allowed here.
    replace_text: String,

    /// Should the global search be *Case Sensitive*?
    case_sensitive: bool,

    /// If the search must be done using regex instead basic matching.
    use_regex: bool,

    /// Where should we search.
    source: SearchSource,

    /// In which files we should search on.
    search_on: SearchOn,

    /// Matches returned by this search.
    matches: Matches
}

/// This enum defines the matching mode of the search. We use `Pattern` by default, and fall back to it
/// if we try to use `Regex` and the provided regex expression is invalid.
#[derive(Debug, Clone)]
pub enum MatchingMode {
    Regex(Regex),
    Pattern(Option<Regex>),
}

/// This enum is a way to put together all kind of matches.
#[derive(Debug, Clone)]
pub enum MatchHolder {
    Anim(UnknownMatches),
    AnimFragment(UnknownMatches),
    AnimPack(UnknownMatches),
    AnimsTable(UnknownMatches),
    Atlas(AtlasMatches),
    Audio(UnknownMatches),
    Bmd(UnknownMatches),
    Db(TableMatches),
    Esf(UnknownMatches),
    GroupFormations(UnknownMatches),
    Image(UnknownMatches),
    Loc(TableMatches),
    MatchedCombat(UnknownMatches),
    Pack(UnknownMatches),
    PortraitSettings(PortraitSettingsMatches),
    RigidModel(RigidModelMatches),
    SoundBank(UnknownMatches),
    Text(TextMatches),
    Uic(UnknownMatches),
    UnitVariant(UnitVariantMatches),
    Unknown(UnknownMatches),
    Video(UnknownMatches),
    Schema(SchemaMatches),
}

/// This enum is specifies the source where the search should be performed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum SearchSource {
    #[default] Pack,
    ParentFiles,
    GameFiles,
    AssKitFiles,
}

/// This struct specifies in what file types is the search going to be performed.
#[derive(Default, Debug, Clone, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct SearchOn {
    anim: bool,
    anim_fragment: bool,
    anim_pack: bool,
    anims_table: bool,
    atlas: bool,
    audio: bool,
    bmd: bool,
    db: bool,
    esf: bool,
    group_formations: bool,
    image: bool,
    loc: bool,
    matched_combat: bool,
    pack: bool,
    portrait_settings: bool,
    rigid_model: bool,
    sound_bank: bool,
    text: bool,
    uic: bool,
    unit_variant: bool,
    unknown: bool,
    video: bool,
    schema: bool,
}

/// This struct stores the search matches, separated by file type.
#[derive(Default, Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Matches {
    anim: Vec<UnknownMatches>,
    anim_fragment: Vec<UnknownMatches>,
    anim_pack: Vec<UnknownMatches>,
    anims_table: Vec<UnknownMatches>,
    atlas: Vec<AtlasMatches>,
    audio: Vec<UnknownMatches>,
    bmd: Vec<UnknownMatches>,
    db: Vec<TableMatches>,
    esf: Vec<UnknownMatches>,
    group_formations: Vec<UnknownMatches>,
    image: Vec<UnknownMatches>,
    loc: Vec<TableMatches>,
    matched_combat: Vec<UnknownMatches>,
    pack: Vec<UnknownMatches>,
    portrait_settings: Vec<PortraitSettingsMatches>,
    rigid_model: Vec<RigidModelMatches>,
    sound_bank: Vec<UnknownMatches>,
    text: Vec<TextMatches>,
    uic: Vec<UnknownMatches>,
    unit_variant: Vec<UnitVariantMatches>,
    unknown: Vec<UnknownMatches>,
    video: Vec<UnknownMatches>,
    schema: SchemaMatches,
}

//---------------------------------------------------------------p----------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GlobalSearch {

    /// This function performs a search over the parts of a `PackFile` you specify it, storing his results.
    pub fn search(&mut self, game_info: &GameInfo, schema: &Schema, pack: &mut Pack, dependencies: &mut Dependencies, update_paths: &[ContainerPath]) {

        // Don't do anything if we have no pattern to search.
        if self.pattern.is_empty() { return }

        // If we want to use regex and the pattern is invalid, don't search.
        let matching_mode = if self.use_regex {
            match RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                Ok(regex) => MatchingMode::Regex(regex),
                Err(_) => MatchingMode::Pattern(RegexBuilder::new(&format!("(?i){}", regex::escape(&self.pattern)))
                    .case_insensitive(!self.case_sensitive)
                    .build()
                    .ok()
                ),
            }
        } else {
            match RegexBuilder::new(&format!("(?i){}", regex::escape(&self.pattern))).case_insensitive(!self.case_sensitive).build() {
                Ok(regex) => MatchingMode::Pattern(Some(regex)),
                Err(_) => MatchingMode::Pattern(None),
            }
        };

        // If we're updating, make sure to dedup and get the raw paths of each file to update.
        let update_paths = if !update_paths.is_empty() && self.source == SearchSource::Pack {
            let container_paths = ContainerPath::dedup(update_paths);
            let raw_paths = container_paths.par_iter()
                .map(|container_path| pack.paths_raw_from_container_path(container_path))
                .flatten()
                .collect::<Vec<_>>();

            self.matches_mut().retain_paths(&raw_paths);

            container_paths
        }

        // Otherwise, ensure we don't store results from previous searches.
        else {
            self.matches = Matches::default();

            vec![]
        };

        // Schema matches do not support "update search".
        self.matches.schema = SchemaMatches::default();

        let pattern_original = self.pattern.to_owned();
        if !self.case_sensitive {
            self.pattern = self.pattern.to_lowercase();
        }

        let pattern = self.pattern.to_owned();
        let case_sensitive = self.case_sensitive;
        let search_on = self.search_on().clone();

        match self.source {
            SearchSource::Pack => {

                let files_to_search = self.search_on().types_to_search();
                let mut files = if !update_paths.is_empty() {
                    pack.files_by_type_and_paths_mut(&files_to_search, &update_paths, false)
                } else {
                    pack.files_by_type_mut(&files_to_search)
                };

                self.matches_mut().find_matches(&pattern, case_sensitive, &matching_mode, &search_on, &mut files, schema);
            }
            SearchSource::ParentFiles => {

                let files_to_search = self.search_on().types_to_search();
                let files = dependencies.files_by_types_mut(&files_to_search, false, true);

                self.matches_mut().find_matches(&pattern, case_sensitive, &matching_mode, &search_on, &mut files.into_values().collect::<Vec<_>>(), schema);
            },
            SearchSource::GameFiles => {

                let files_to_search = self.search_on().types_to_search();
                let files = dependencies.files_by_types_mut(&files_to_search, true, false);

                self.matches_mut().find_matches(&pattern, case_sensitive, &matching_mode, &search_on, &mut files.into_values().collect::<Vec<_>>(), schema);
            },

            // Asskit files are only tables.
            SearchSource::AssKitFiles => {
                if self.search_on.db {
                    self.matches.db = dependencies.asskit_only_db_tables()
                        .par_iter()
                        .filter_map(|(table_name, table)| {
                            let file_name = match game_info.vanilla_db_table_name_logic() {
                                VanillaDBTableNameLogic::FolderName => table_name.to_owned(),
                                VanillaDBTableNameLogic::DefaultName(ref default_name) => default_name.to_owned()
                            };

                            let path = format!("db/{table_name}/{file_name}");
                            let result = table.search(&path, &self.pattern, self.case_sensitive, &matching_mode);
                            if !result.matches().is_empty() {
                                Some(result)
                            } else {
                                None
                            }
                        }
                    ).collect();
                }
            },
        }

        // Restore the pattern to what it was before searching.
        self.pattern = pattern_original;
    }

    /// This function clears the Global Search result's data, and reset the UI for it.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// This function checks if it's possible to replace the provided matches.
    pub fn replace_possible(&self, matches: &[MatchHolder]) -> Result<()> {
        let patterns_same_lenght = self.pattern.len() == self.replace_text.len();

        // Error out if at least one of the matches requires special conditions.
        if matches.iter().any(|m| match m {
            MatchHolder::Anim(_) => false,
            MatchHolder::AnimFragment(_) => false,
            MatchHolder::AnimPack(_) => false,
            MatchHolder::AnimsTable(_) => false,
            MatchHolder::Atlas(_) => false,
            MatchHolder::Audio(_) => false,
            MatchHolder::Bmd(_) => false,
            MatchHolder::Db(_) => false,
            MatchHolder::Esf(_) => false,
            MatchHolder::GroupFormations(_) => false,
            MatchHolder::Image(_) => false,
            MatchHolder::Loc(_) => false,
            MatchHolder::MatchedCombat(_) => false,
            MatchHolder::Pack(_) => false,
            MatchHolder::PortraitSettings(_) => false,
            MatchHolder::RigidModel(_) => self.use_regex || !patterns_same_lenght,
            MatchHolder::Schema(_) => false,
            MatchHolder::SoundBank(_) => false,
            MatchHolder::Text(_) => false,
            MatchHolder::Uic(_) => false,
            MatchHolder::UnitVariant(_) => false,
            MatchHolder::Unknown(_) => self.use_regex || !patterns_same_lenght,
            MatchHolder::Video(_) => false,
        }) {
            Err(RLibError::GlobalSearchReplaceRequiresSameLenghtAndNotRegex)
        } else {
            Ok(())
        }
    }

    /// This function performs a replace operation over the provided matches.
    ///
    /// NOTE: Schema matches are always ignored.
    pub fn replace(&mut self, game_info: &GameInfo, schema: &Schema, pack: &mut Pack, dependencies: &mut Dependencies, matches: &[MatchHolder]) -> Result<Vec<ContainerPath>> {
        let mut edited_paths = vec![];

        // Don't do anything if we have no pattern to search.
        if self.pattern.is_empty() {
            return Ok(edited_paths)
        }

        // This is only useful for Packs, not for dependencies.
        if self.source != SearchSource::Pack {
            return Ok(edited_paths)
        }

        // Make sure we can actually do the replacements.
        self.replace_possible(matches)?;

        // If we want to use regex and the pattern is invalid, use normal pattern instead of Regex.
        let matching_mode = if self.use_regex {
            match RegexBuilder::new(&self.pattern).case_insensitive(!self.case_sensitive).build() {
                Ok(regex) => MatchingMode::Regex(regex),
                Err(_) => MatchingMode::Pattern(RegexBuilder::new(&format!("(?i){}", regex::escape(&self.pattern)))
                    .case_insensitive(!self.case_sensitive)
                    .build()
                    .ok()
                ),
            }
        } else {
            match RegexBuilder::new(&format!("(?i){}", regex::escape(&self.pattern))).case_insensitive(!self.case_sensitive).build() {
                Ok(regex) => MatchingMode::Pattern(Some(regex)),
                Err(_) => MatchingMode::Pattern(None),
            }
        };

        // Just replace all the provided matches, one by one.
        for match_file in matches {
            match match_file {
                MatchHolder::Anim(_) => continue,
                MatchHolder::AnimFragment(_) => continue,
                MatchHolder::AnimPack(_) => continue,
                MatchHolder::AnimsTable(_) => continue,
                MatchHolder::Atlas(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::Atlas(table) => table.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::Audio(_) => continue,
                MatchHolder::Bmd(_) => continue,

                MatchHolder::Db(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::DB(table) => table.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::Esf(_) => continue,
                MatchHolder::GroupFormations(_) => continue,
                MatchHolder::Image(_) => continue,
                MatchHolder::Loc(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::Loc(table) => table.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::MatchedCombat(_) => continue,
                MatchHolder::Pack(_) => continue,
                MatchHolder::PortraitSettings(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {

                        // Make sure it has been decoded.
                        let _ = file.decode(&None, true, false);
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::PortraitSettings(data) => data.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::RigidModel(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {

                        // Make sure it has been decoded.
                        let _ = file.decode(&None, true, false);
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::RigidModel(data) => data.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::SoundBank(_) => continue,
                MatchHolder::Text(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {

                        // Make sure it has been decoded.
                        let _ = file.decode(&None, true, false);
                        if let Ok(decoded) = file.decoded_mut() {

                            // NOTE: Make freaking sure this is sorted properly. Otherwise the replace logic will break when changing the lenght of the string.
                            let mut search_matches = search_matches.clone();
                            search_matches.matches_mut().par_sort_unstable_by(|a, b| {
                                if a.row() == b.row() {
                                    a.start().cmp(b.start())
                                } else {
                                    a.row().cmp(b.row())
                                }
                            });

                            let edited = match decoded {
                                RFileDecoded::Text(text) => text.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, &search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::Uic(_) => continue,
                MatchHolder::UnitVariant(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {

                        // Make sure it has been decoded.
                        let _ = file.decode(&None, true, false);
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::UnitVariant(data) => data.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },

                MatchHolder::Unknown(search_matches) => {
                    let container_path = ContainerPath::File(search_matches.path().to_string());
                    let mut file = pack.files_by_path_mut(&container_path, false);
                    if let Some(file) = file.get_mut(0) {

                        // Make sure it has been decoded.
                        let _ = file.decode(&None, true, false);
                        if let Ok(decoded) = file.decoded_mut() {
                            let edited = match decoded {
                                RFileDecoded::Unknown(data) => data.replace(&self.pattern, &self.replace_text, self.case_sensitive, &matching_mode, search_matches),
                                _ => unimplemented!(),
                            };

                            if edited {
                                edited_paths.push(container_path);
                            }
                        }
                    }
                },
                MatchHolder::Video(_) => continue,

                // We cannot edit schemas here.
                MatchHolder::Schema(_) => continue,
            }
        }

        // Update the current search over the edited files.
        self.search(game_info, schema, pack, dependencies, &edited_paths);

        // Return the changed paths.
        Ok(edited_paths)
    }

    pub fn replace_all(&mut self, game_info: &GameInfo, schema: &Schema, pack: &mut Pack, dependencies: &mut Dependencies) -> Result<Vec<ContainerPath>> {
        let mut matches = vec![];

        matches.extend(self.matches.anim.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.anim_fragment.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.anim_pack.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.anims_table.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.atlas.iter().map(|x| MatchHolder::Atlas(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.audio.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.bmd.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.db.iter().map(|x| MatchHolder::Db(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.esf.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.group_formations.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.image.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.loc.iter().map(|x| MatchHolder::Loc(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.matched_combat.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.pack.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.portrait_settings.iter().map(|x| MatchHolder::PortraitSettings(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.rigid_model.iter().map(|x| MatchHolder::RigidModel(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.sound_bank.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.text.iter().map(|x| MatchHolder::Text(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.uic.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.unit_variant.iter().map(|x| MatchHolder::UnitVariant(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.unknown.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());
        matches.extend(self.matches.video.iter().map(|x| MatchHolder::Unknown(x.clone())).collect::<Vec<_>>());

        self.replace(game_info, schema, pack, dependencies, &matches)
    }
}

impl SearchOn {
    pub fn types_to_search(&self) -> Vec<FileType> {
        let mut types = vec![];

        if *self.anim() { types.push(FileType::Anim); }
        if *self.anim_fragment() { types.push(FileType::AnimFragment); }
        if *self.anim_pack() { types.push(FileType::AnimPack); }
        if *self.anims_table() { types.push(FileType::AnimsTable); }
        if *self.atlas() { types.push(FileType::Atlas); }
        if *self.audio() { types.push(FileType::Audio); }
        if *self.bmd() { types.push(FileType::BMD); }
        if *self.db() { types.push(FileType::DB); }
        if *self.esf() { types.push(FileType::ESF); }
        if *self.group_formations() { types.push(FileType::GroupFormations); }
        if *self.image() { types.push(FileType::Image); }
        if *self.loc() { types.push(FileType::Loc); }
        if *self.matched_combat() { types.push(FileType::MatchedCombat); }
        if *self.pack() { types.push(FileType::Pack); }
        if *self.portrait_settings() { types.push(FileType::PortraitSettings); }
        if *self.rigid_model() { types.push(FileType::RigidModel); }
        if *self.sound_bank() { types.push(FileType::SoundBank); }
        if *self.text() { types.push(FileType::Text); }
        if *self.uic() { types.push(FileType::UIC); }
        if *self.unit_variant() { types.push(FileType::UnitVariant); }
        if *self.unknown() { types.push(FileType::Unknown); }
        if *self.video() { types.push(FileType::Video); }

        types
    }
}

impl Matches {
    pub fn retain_paths(&mut self, paths: &[String]) {
        for path in paths {
            self.anim.retain(|x| x.path() != path);
            self.anim_fragment.retain(|x| x.path() != path);
            self.anim_pack.retain(|x| x.path() != path);
            self.anims_table.retain(|x| x.path() != path);
            self.atlas.retain(|x| x.path() != path);
            self.audio.retain(|x| x.path() != path);
            self.bmd.retain(|x| x.path() != path);
            self.db.retain(|x| x.path() != path);
            self.esf.retain(|x| x.path() != path);
            self.group_formations.retain(|x| x.path() != path);
            self.image.retain(|x| x.path() != path);
            self.loc.retain(|x| x.path() != path);
            self.matched_combat.retain(|x| x.path() != path);
            self.pack.retain(|x| x.path() != path);
            self.portrait_settings.retain(|x| x.path() != path);
            self.rigid_model.retain(|x| x.path() != path);
            self.sound_bank.retain(|x| x.path() != path);
            self.text.retain(|x| x.path() != path);
            self.uic.retain(|x| x.path() != path);
            self.unit_variant.retain(|x| x.path() != path);
            self.unknown.retain(|x| x.path() != path);
            self.video.retain(|x| x.path() != path);
        }
    }

    pub fn find_matches(&mut self, pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, search_on: &SearchOn, files: &mut Vec<&mut RFile>, schema: &Schema) {
        let matches = files.par_iter_mut()
            .filter_map(|file| {
                if search_on.anim && file.file_type() == FileType::Anim {
                    /*
                    if let Ok(RFileDecoded::Anim(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.anim_fragment && file.file_type() == FileType::AnimFragment {
                    /*
                    if let Ok(RFileDecoded::AnimFragment(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.anim_pack && file.file_type() == FileType::AnimPack {
                    /*
                    if let Ok(RFileDecoded::AnimPack(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.anims_table && file.file_type() == FileType::AnimsTable {
                    /*
                    if let Ok(RFileDecoded::AnimsTable(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.atlas && file.file_type() == FileType::Atlas {
                    if let Ok(RFileDecoded::Atlas(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.audio && file.file_type() == FileType::Audio {
                    /*
                    if let Ok(RFileDecoded::Audio(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.bmd && file.file_type() == FileType::BMD {
                    /*
                    if let Ok(RFileDecoded::BMD(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.db && file.file_type() == FileType::DB {
                    if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                        let result = table.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.esf && file.file_type() == FileType::ESF {
                    /*
                    if let Ok(RFileDecoded::ESF(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.group_formations && file.file_type() == FileType::GroupFormations {
                    /*
                    if let Ok(RFileDecoded::GroupFormations(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.image && file.file_type() == FileType::Image {
                    /*
                    if let Ok(RFileDecoded::Image(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.loc && file.file_type() == FileType::Loc {
                    if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                        let result = table.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.matched_combat && file.file_type() == FileType::MatchedCombat {
                    /*
                    if let Ok(RFileDecoded::MatchedCombat(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.pack && file.file_type() == FileType::Pack {
                    /*
                    if let Ok(RFileDecoded::Pack(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.portrait_settings && file.file_type() == FileType::PortraitSettings {
                    if let Ok(RFileDecoded::PortraitSettings(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.rigid_model && file.file_type() == FileType::RigidModel {
                    if let Ok(RFileDecoded::RigidModel(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.sound_bank && file.file_type() == FileType::SoundBank {
                    /*
                    if let Ok(RFileDecoded::SoundBank(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.text && file.file_type() == FileType::Text {
                    if let Ok(RFileDecoded::Text(table)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = table.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.uic && file.file_type() == FileType::UIC {
                    /*
                    if let Ok(RFileDecoded::UIC(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else if search_on.unit_variant && file.file_type() == FileType::UnitVariant {
                    if let Ok(RFileDecoded::UnitVariant(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None, None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.unknown && file.file_type() == FileType::Unknown {
                    if let Ok(RFileDecoded::Unknown(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result), None))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if search_on.video && file.file_type() == FileType::Video {
                    /*
                    if let Ok(RFileDecoded::Video(data)) = file.decode(&None, false, true).transpose().unwrap() {
                        let result = data.search(file.path_in_container_raw(), pattern, case_sensitive, &matching_mode);
                        if !result.matches().is_empty() {
                            Some((None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(result)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }*/
                    None
                } else {
                    None
                }
            }
        ).collect::<Vec<(
            Option<UnknownMatches>, Option<UnknownMatches>, Option<UnknownMatches>, Option<UnknownMatches>, Option<AtlasMatches>, Option<UnknownMatches>, Option<UnknownMatches>, Option<TableMatches>,
            Option<UnknownMatches>, Option<UnknownMatches>, Option<UnknownMatches>, Option<TableMatches>, Option<UnknownMatches>, Option<UnknownMatches>, Option<PortraitSettingsMatches>,
            Option<RigidModelMatches>, Option<UnknownMatches>, Option<TextMatches>, Option<UnknownMatches>, Option<UnitVariantMatches>, Option<UnknownMatches>, Option<UnknownMatches>
        )>>();

        self.anim = matches.iter().filter_map(|x| x.0.clone()).collect::<Vec<_>>();
        self.anim_fragment = matches.iter().filter_map(|x| x.1.clone()).collect::<Vec<_>>();
        self.anim_pack = matches.iter().filter_map(|x| x.2.clone()).collect::<Vec<_>>();
        self.anims_table = matches.iter().filter_map(|x| x.3.clone()).collect::<Vec<_>>();
        self.atlas = matches.iter().filter_map(|x| x.4.clone()).collect::<Vec<_>>();
        self.audio = matches.iter().filter_map(|x| x.5.clone()).collect::<Vec<_>>();
        self.bmd = matches.iter().filter_map(|x| x.6.clone()).collect::<Vec<_>>();
        self.db = matches.iter().filter_map(|x| x.7.clone()).collect::<Vec<_>>();
        self.esf = matches.iter().filter_map(|x| x.8.clone()).collect::<Vec<_>>();
        self.group_formations = matches.iter().filter_map(|x| x.9.clone()).collect::<Vec<_>>();
        self.image = matches.iter().filter_map(|x| x.10.clone()).collect::<Vec<_>>();
        self.loc = matches.iter().filter_map(|x| x.11.clone()).collect::<Vec<_>>();
        self.matched_combat = matches.iter().filter_map(|x| x.12.clone()).collect::<Vec<_>>();
        self.pack = matches.iter().filter_map(|x| x.13.clone()).collect::<Vec<_>>();
        self.portrait_settings = matches.iter().filter_map(|x| x.14.clone()).collect::<Vec<_>>();
        self.rigid_model = matches.iter().filter_map(|x| x.15.clone()).collect::<Vec<_>>();
        self.sound_bank = matches.iter().filter_map(|x| x.16.clone()).collect::<Vec<_>>();
        self.text = matches.iter().filter_map(|x| x.17.clone()).collect::<Vec<_>>();
        self.uic = matches.iter().filter_map(|x| x.18.clone()).collect::<Vec<_>>();
        self.unit_variant = matches.iter().filter_map(|x| x.19.clone()).collect::<Vec<_>>();
        self.unknown = matches.iter().filter_map(|x| x.20.clone()).collect::<Vec<_>>();
        self.video = matches.iter().filter_map(|x| x.21.clone()).collect::<Vec<_>>();

        // Schema searches are a bit independant from the rest, so they're done after the full search.
        if search_on.schema {
            self.schema = schema.search("", pattern, case_sensitive, matching_mode);
        }
    }
}

impl Default for MatchingMode {
    fn default() -> Self {
        Self::Pattern(None)
    }
}

//-------------------------------------------------------------------------------//
//                              Util functions
//-------------------------------------------------------------------------------//

fn replace_match_string(pattern: &str, replace_pattern: &str, case_sensitive: bool, matching_mode: &MatchingMode, start: usize, end: usize, previous_data: &str, current_data: &mut String) -> bool {

    // Only replace if the substring is actually a valid one.
    if current_data.get(start..end).is_some() {
        match matching_mode {
            MatchingMode::Regex(regex) => {
                if let Some(match_regex) = regex.find(&current_data[start..end]) {
                    if match_regex.start() == 0 && match_regex.end() == end - start {
                        current_data.replace_range(start..end, replace_pattern);
                    }
                }
            },

            MatchingMode::Pattern(regex) => {
                let pattern = if case_sensitive || regex.is_some() {
                    pattern.to_owned()
                } else {
                    pattern.to_lowercase()
                };

                if let Some((start_new, end_new, _)) = find_in_string(&current_data[start..end], &pattern, case_sensitive, regex).get(0) {
                    if *start_new == 0 && *end_new == end - start {
                        current_data.replace_range(start..end, replace_pattern);
                    }
                }
            }
        }
    }

    previous_data != *current_data
}

fn replace_match_bytes(replace_pattern: &str, start: usize, len: usize, data: &mut Vec<u8>) -> bool {
    let old_data = data[start..start + len].to_vec();
    data.splice(start..start + len, replace_pattern.as_bytes().to_vec());
    old_data != data[start..start + len]
}

fn find_in_string(value: &str, pattern: &str, case_sensitive: bool, case_insensitive_regex: &Option<Regex>) -> Vec<(usize, usize, String)> {
    if case_sensitive {
        value.match_indices(&pattern).map(|(start, pat)| (start, start + pat.len(), pat.to_owned())).collect()
    } else if let Some(regex) = case_insensitive_regex {
        regex.find_iter(value).map(|m| (m.start(), m.end(), m.as_str().to_string())).collect()
    } else {
        value.to_lowercase().match_indices(&pattern).map(|(start, pat)| (start, start + pat.len(), value[start..start + pat.len()].to_string())).collect()
    }
}

fn find_in_bytes(value: &[u8], pattern: &str, case_sensitive: bool, case_insensitive_regex: &Option<regex::bytes::Regex>) -> Vec<(usize, usize)> {
    if case_sensitive {
        let length = pattern.len();
        (0..value.len() - length)
            .filter_map(|index| if &value[index..index + length] == pattern.as_bytes() { Some((index, length)) } else { None })
            .collect()

    } else if let Some(regex) = case_insensitive_regex {
        regex.find_iter(value).map(|m| (m.start(), m.len())).collect()
    } else {
        let pattern = pattern.as_bytes().to_ascii_lowercase();
        let value = value.to_ascii_lowercase();
        let length = pattern.len();
        (0..value.len() - length)
            .filter_map(|index| if value[index..index + length] == pattern { Some((index, length)) } else { None })
            .collect()    }
}

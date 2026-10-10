//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Mission scripts of a campaign (`victory_objectives.txt`, `missions.txt`) and the victory objectives they define.
//!
//! A script is a list of statements: words ended by `;` (`total 80;`), or words followed by a block of statements
//! (`objective { ... }`). `//` starts a comment that runs to the end of the line.

use getset::{CopyGetters, Getters};

use rpfm_lib::error::{RLibError, Result};

#[cfg(test)] #[path = "missions_test.rs"] mod missions_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// A statement of a mission script.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters)]
#[getset(get = "pub")]
pub struct ScriptItem {

    /// Words of the statement: its key, then its values.
    words: Vec<String>,

    /// Statements of its block, if it has one.
    block: Option<Vec<ScriptItem>>,
}

/// The victory missions of a faction in a kind of campaign (a `victory_objective` block).
#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct VictoryObjectives {

    /// Faction key.
    #[getset(get = "pub")]
    faction: String,

    /// Kind of campaign the missions are for.
    #[getset(get_copy = "pub")]
    campaign_type: CampaignType,

    /// Missions, in script order.
    #[getset(get = "pub")]
    missions: Vec<VictoryMission>,
}

/// Kinds of campaign with their own victory missions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignType {

    /// Single player campaigns (`sp_normal`).
    SinglePlayer,

    /// Multiplayer campaigns against other players (`mp_vs_normal`).
    MultiplayerVersus,

    /// Cooperative multiplayer campaigns (`mp_co_op_normal`).
    MultiplayerCoop,
}

/// A victory mission. Completing its objectives wins the game.
#[derive(Debug, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct VictoryMission {

    /// Mission key.
    key: String,

    /// Kind of victory (`att_vc_1_short`...).
    victory_type: String,

    /// Who gives the mission (`CLAN_ELDERS`...).
    issuer: String,

    /// Objectives, in script order.
    objectives: Vec<MissionObjective>,
}

/// An objective of a mission, by its script `type`. `total` is the amount the objective asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum MissionObjective {

    /// Reach a date (`year`, `week_of_year`).
    ReachSpecifiedDate { year: u32, week: u32 },

    /// Control a number of regions.
    ControlNRegionsIncluding { total: u32 },

    /// Control a number of provinces among some (`province`).
    ControlNProvincesIncluding { total: u32, provinces: Vec<String> },

    /// Research a number of technologies of some categories (`technology_category`), with an optional text
    /// (`override_text`).
    ResearchNTechsOfTypeX { total: u32, categories: Vec<String>, text: Option<String> },

    /// Earn an amount of money from buildings.
    EarnXAmountFromBuildingWealth { total: u32 },

    /// Earn an amount of money from raiding.
    EarnXAmountFromRaiding { total: u32 },

    /// Own a number of units (`OWN_N_UNITS`), or of naval units (`OWN_N_NAVAL_UNITS`).
    OwnNUnits { total: u32, naval: bool },

    /// Own a number of provinces where the faction's culture dominates.
    OwnNProvincesWithCulturalDominance { total: u32 },

    /// Construct a number of buildings among some (`building_level`), for a faction.
    ConstructNBuildingsIncluding { total: u32, buildings: Vec<String>, faction: String },

    /// Subjugate a number of factions.
    SubjugateFactions { total: u32 },

    /// Keep a number of client states.
    MaintainNClientStates { total: u32 },

    /// Loot or sack a number of settlements (`region` lists some; the game doesn't keep them).
    LootOrSackNDifferentSettlementsIncluding { total: u32, regions: Vec<String> },

    /// Raze or sack a number of settlements (`region` lists some; the game doesn't keep them).
    RazeOrSackNDifferentSettlementsIncluding { total: u32, regions: Vec<String> },

    /// Have a religion at a percentage in every province.
    AtLeastXReligionInAllProvinces { total: f32, religion: String },
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl ScriptItem {

    /// Returns the key of the statement (its first word).
    pub fn key(&self) -> &str {
        self.words.first().map(String::as_str).unwrap_or_default()
    }

    /// Returns the values of the statement (the words after its key).
    pub fn values(&self) -> &[String] {
        self.words.get(1..).unwrap_or_default()
    }

    /// Returns the statements of the block, or none for statements without one.
    pub fn children(&self) -> &[ScriptItem] {
        self.block.as_deref().unwrap_or_default()
    }

    /// Returns the first value of each child statement with a key, in order.
    pub fn child_values(&self, key: &str) -> Vec<String> {
        self.children().iter()
            .filter(|child| child.key() == key)
            .filter_map(|child| child.values().first().cloned())
            .collect()
    }

    /// Returns the first value of the first child statement with a key.
    ///
    /// # Errors
    ///
    /// Returns an error if there's no such statement, or it has no value.
    pub fn child_value(&self, key: &str) -> Result<String> {
        self.child_values(key).into_iter().next().ok_or_else(|| script_error(format!("{} lacks {key}", self.key())))
    }

    /// Returns the first value of the first child statement with a key, as a number.
    ///
    /// # Errors
    ///
    /// Returns an error if there's no such statement, or its value isn't a number.
    pub fn child_number<T: std::str::FromStr>(&self, key: &str) -> Result<T> {
        let value = self.child_value(key)?;
        value.parse().map_err(|_| script_error(format!("{key} of {} isn't a number: {value}", self.key())))
    }
}

impl CampaignType {

    /// Returns the id of the campaign type, as `VICTORY_MISSIONS_BY_TYPE_BLOCK` stores it.
    pub fn id(&self) -> u32 {
        match self {
            Self::SinglePlayer => 0,
            Self::MultiplayerVersus => 2,
            Self::MultiplayerCoop => 4,
        }
    }

    /// Returns the campaign type of a script key (`sp_normal`, `mp_vs_normal`, `mp_co_op_normal`).
    ///
    /// # Errors
    ///
    /// Returns an error for unknown keys.
    pub fn from_key(key: &str) -> Result<Self> {
        match key {
            "sp_normal" => Ok(Self::SinglePlayer),
            "mp_vs_normal" => Ok(Self::MultiplayerVersus),
            "mp_co_op_normal" => Ok(Self::MultiplayerCoop),
            _ => Err(script_error(format!("unknown campaign type {key}"))),
        }
    }
}

impl MissionObjective {

    /// Reads an objective from its script block.
    ///
    /// # Errors
    ///
    /// Returns an error if the objective's type is unsupported, or a value it needs is missing or malformed.
    pub fn from_item(item: &ScriptItem) -> Result<Self> {
        let total = || item.child_number::<u32>("total");
        Ok(match item.child_value("type")?.as_str() {
            "REACH_SPECIFIED_DATE" => Self::ReachSpecifiedDate { year: item.child_number("year")?, week: item.child_number("week_of_year")? },
            "CONTROL_N_REGIONS_INCLUDING" => Self::ControlNRegionsIncluding { total: total()? },
            "CONTROL_N_PROVINCES_INCLUDING" => Self::ControlNProvincesIncluding { total: total()?, provinces: item.child_values("province") },
            "RESEARCH_N_TECHS_OF_TYPE_X" => Self::ResearchNTechsOfTypeX {
                total: total()?,
                categories: item.child_values("technology_category"),
                text: item.child_values("override_text").into_iter().next(),
            },
            "EARN_X_AMOUNT_FROM_BUILDING_WEALTH" => Self::EarnXAmountFromBuildingWealth { total: total()? },
            "EARN_X_AMOUNT_FROM_RAIDING" => Self::EarnXAmountFromRaiding { total: total()? },
            "OWN_N_UNITS" => Self::OwnNUnits { total: total()?, naval: false },
            "OWN_N_NAVAL_UNITS" => Self::OwnNUnits { total: total()?, naval: true },
            "OWN_N_PROVINCES_WITH_CULTURAL_DOMINANCE" => Self::OwnNProvincesWithCulturalDominance { total: total()? },
            "CONSTRUCT_N_BUILDINGS_INCLUDING" => Self::ConstructNBuildingsIncluding {
                total: total()?,
                buildings: item.child_values("building_level"),
                faction: item.child_value("faction")?,
            },
            "SUBJUGATE_FACTIONS" => Self::SubjugateFactions { total: total()? },
            "MAINTAIN_N_CLIENT_STATES" => Self::MaintainNClientStates { total: total()? },
            "LOOT_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING" => Self::LootOrSackNDifferentSettlementsIncluding { total: total()?, regions: item.child_values("region") },
            "RAZE_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING" => Self::RazeOrSackNDifferentSettlementsIncluding { total: total()?, regions: item.child_values("region") },
            "AT_LEAST_X_RELIGION_IN_ALL_PROVINCES" => Self::AtLeastXReligionInAllProvinces { total: item.child_number("total")?, religion: item.child_value("religion")? },
            other => return Err(script_error(format!("unsupported objective type {other}"))),
        })
    }
}

/// Parses a mission script into its statements.
///
/// # Errors
///
/// Returns an error if a block isn't closed, or a `}` closes nothing.
pub fn parse_script(text: &str) -> Result<Vec<ScriptItem>> {
    let mut tokens = vec![];
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or_default();
        for word in line.replace('{', " { ").replace('}', " } ").replace(';', " ; ").split_whitespace() {
            tokens.push(word.to_owned());
        }
    }

    parse_block(&mut tokens.into_iter(), false)
}

/// Parses the victory objectives of a `victory_objectives.txt` script.
///
/// # Errors
///
/// Returns an error if the script is malformed, or uses an objective or payload that isn't supported.
pub fn parse_victory_objectives(text: &str) -> Result<Vec<VictoryObjectives>> {
    let script = parse_script(text)?;
    let mut objectives = vec![];
    for block in script.iter().filter(|item| item.key() == "victory_objectives").flat_map(ScriptItem::children) {
        if block.key() != "victory_objective" {
            continue;
        }

        let mut missions = vec![];
        for mission in block.children().iter().filter(|item| item.key() == "mission") {
            let primary = mission.children().iter()
                .find(|item| item.key() == "primary_objectives_and_payload")
                .ok_or_else(|| script_error(format!("mission {} has no primary objectives", mission.child_value("key").unwrap_or_default())))?;
            for payload in primary.children().iter().filter(|item| item.key() == "payload").flat_map(ScriptItem::children) {
                if payload.key() != "game_victory" {
                    return Err(script_error(format!("unsupported victory payload {}", payload.key())));
                }
            }

            missions.push(VictoryMission {
                key: mission.child_value("key")?,
                victory_type: mission.child_value("victory_type")?,
                issuer: mission.child_value("issuer")?,
                objectives: primary.children().iter().filter(|item| item.key() == "objective").map(MissionObjective::from_item).collect::<Result<_>>()?,
            });
        }

        objectives.push(VictoryObjectives {
            faction: block.child_value("faction")?,
            campaign_type: CampaignType::from_key(&block.child_value("campaign_type")?)?,
            missions,
        });
    }
    Ok(objectives)
}

/// Parses statements until the end of the current block (or of the script, at the top level).
fn parse_block(tokens: &mut impl Iterator<Item = String>, nested: bool) -> Result<Vec<ScriptItem>> {
    let mut items = vec![];
    let mut words = vec![];
    while let Some(token) = tokens.next() {
        match token.as_str() {
            ";" => items.push(ScriptItem { words: std::mem::take(&mut words), block: None }),
            "{" => {
                let block = parse_block(tokens, true)?;
                items.push(ScriptItem { words: std::mem::take(&mut words), block: Some(block) });
            },
            "}" if nested => return Ok(items),
            "}" => return Err(script_error("a } closes no block".to_owned())),
            _ => words.push(token),
        }
    }
    if nested {
        return Err(script_error("a block isn't closed".to_owned()));
    }
    Ok(items)
}

/// Builds a script error.
fn script_error(message: String) -> RLibError {
    RLibError::StartposScriptError(message)
}

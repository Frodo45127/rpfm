//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the AI faction managers (`CAI_INTERFACE_MANAGERS`).
//!
//! Each faction owning regions or mobiles, and the rebels, has a manager: a pool of desires (analysers and behaviour
//! managers, of fixed types), two beliefs about the faction's finances and strategic context, and intentions (one for
//! the faction, one per character and one per mobile, to manage their skills). Every faction gets the AI level 7
//! manager (28 desires); the rebels get the level 2 one (the first 18 desires, nothing else).
//!
//! Each object starts from the layout of its type, with its ids and references set. State the game computes on its
//! first update is written as its constructors leave it: the finance manager, the stance manager's empire data and
//! the components' update flags.

use std::collections::HashMap;

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{hash_map_order, NodeType, RecordNode, VecI32Node, VecU32Node};

use super::plan::{CaiPlan, ManagerIds};
use super::stances::StartingStances;
use super::CAI_PATH;
use crate::startpos::layout::{child_path, variant_path, GroupBuilder, RecordBuilder, StartposLayout};
use crate::startpos::writer::{bool_node, i32_node, u32_node};

#[cfg(test)] #[path = "managers_test.rs"] mod managers_test;

/// Type codes of a faction manager's desires, in creation order.
pub const FACTION_DESIRES: [u32; 28] = [39, 44, 37, 41, 38, 40, 34, 94, 108, 150, 23, 82, 152, 144, 114, 141, 140, 25, 171, 98, 175, 185, 84, 173, 183, 90, 88, 159];

/// Type codes of a human manager's desires, in creation order: the rebels' ones, then two more.
pub const HUMAN_DESIRES: [u32; 20] = [39, 44, 37, 41, 38, 40, 34, 94, 108, 150, 23, 82, 152, 144, 114, 141, 140, 25, 159, 90];

/// Desires of the rebels' manager: the first ones of [`FACTION_DESIRES`].
pub const REBEL_DESIRES: usize = 18;

/// AI level of the AI factions' managers.
pub(crate) const FACTION_LEVEL: u32 = 7;

/// AI level of the playable factions' managers.
pub(crate) const HUMAN_LEVEL: u32 = 6;

/// AI level of the rebels' manager.
pub(crate) const REBEL_LEVEL: u32 = 2;

/// Value of `CAI_BDI_POOL_FAILED+1` in the human and rebel pools (the AI pools keep the layout's).
const NON_AI_POOL_VALUE: u32 = 3;

/// Priorities of the human managers' desires that differ from the AI managers' ones.
const HUMAN_PRIORITIES: [(u32, f32); 2] = [(TAXATION, 500.0), (CONSTRUCTION, 100.0)];

/// Slots of `CAI_FACTION_BDI_POOL` holding a desire, with the desire's type. The other slots hold 0.
const POOL_SLOTS: [(u32, u32); 17] = [
    (3, 23), (4, 82), (5, 152), (6, 144), (7, 114), (8, 141), (9, 140), (20, 39), (23, 44), (25, 37), (26, 41), (27, 38),
    (30, 40), (32, 34), (36, 94), (37, 108), (38, 150),
];

/// Last numbered slot of `CAI_FACTION_BDI_POOL`.
const LAST_POOL_SLOT: u32 = 38;

/// Seed the game mixes into AI component ids to hash them.
const COMPONENT_HASH_SEED: u32 = 0x4a54_5eed;

/// Desires refering to the faction's AI faction record.
const CAI_FACTION_USERS: [u32; 3] = [114, 141, 144];

/// Desire types.
const TARGET_LIST: u32 = 25;
const FINANCE_MANAGER: u32 = 82;
const RESEARCH: u32 = 88;
const TAXATION: u32 = 90;
const FINANCIAL_ANALYSER: u32 = 94;
const TMS_COORDINATION: u32 = 140;
const STANCE_MANAGER: u32 = 144;
const STRATEGIC_ANALYSER: u32 = 150;
const DIPLOMACY_MANAGER: u32 = 152;
const TASK_GENERATOR: u32 = 114;
const CONSTRUCTION: u32 = 159;
const FORCE_SKILLS: u32 = 171;
const CHARACTER_SKILLS: u32 = 173;

/// Belief types.
const FINANCIAL_ANALYSIS: u32 = 95;
const STRATEGIC_CONTEXT_ANALYSIS: u32 = 151;

/// Intention types.
const FACTION_INTENTION: u32 = 79;
const FORCE_SKILLS_INTENTION: u32 = 172;
const CHARACTER_SKILLS_INTENTION: u32 = 174;

/// Values of an ownership entry linking an object to one it works for (analysers, intentions).
pub(crate) const OWNER_LINK: (f32, f32, u32) = (0.0, 0.0, 1);

/// Values of an ownership entry linking a skill desire to one of its intentions.
const INTENTION_LINK: (f32, f32, u32) = (0.0, 1.0, 0);

/// Update flags of the components of objects that haven't been updated yet.
const FRESH_FLAGS: u32 = 97;

/// Update flags of the diplomacy manager with and without war entries.
const DIPLOMACY_FLAGS: (u32, u32) = (96, 369);

/// Value of the diplomacy manager's first field without war entries.
const NO_WARS_VALUE: i32 = -100;

/// Pairs of values of each diplomacy manager entry after its first ones.
const DIPLOMACY_ENTRY_PAIRS: u32 = 41;

/// State the diplomacy manager keeps after its war entries, when it has any (the game writes 3 or 5).
const DIPLOMACY_WAR_STATE: u32 = 3;

/// Strategic context of the factions (0 to 4; most factions are at 4 in the game's startpos). The constructor's 5 is
/// out of range: the finance manager's load fixup uses it as a row of the personality's parameter table and crashes.
const STRATEGIC_CONTEXT: u32 = 4;

/// Stances whose scores each stance re-evaluation mark can adjust.
const STANCE_SCORES: usize = 7;

/// Budget categories of the finance manager.
const BUDGET_CATEGORIES: usize = 6;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// Kinds of AI faction managers, by the AI level the game gives them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ManagerKind {

    /// Level 7: a faction the AI plays.
    #[default]
    Ai,

    /// Level 6: a faction players can pick. When a campaign starts, the game turns the ones nobody picked into AI
    /// managers (it never turns AI managers into human ones in its own startposes).
    Human,

    /// Level 2: the rebels.
    Rebels,
}

/// What the managers are written from.
#[derive(Debug, Clone, Copy)]
pub struct ManagerInputs<'a> {

    /// Record layouts.
    pub layout: &'a StartposLayout,

    /// Component ids.
    pub plan: &'a CaiPlan,

    /// Starting wars and stance re-evaluation marks, by faction position.
    pub stances: &'a StartingStances,

    /// Components of each faction's characters, by faction position.
    pub characters: &'a [Vec<u32>],

    /// Components of each faction's mobiles, by faction position.
    pub mobiles: &'a [Vec<u32>],
}

/// Fields of a pool object's component that differ from its type's defaults.
#[derive(Debug, Clone, Default)]
pub(crate) struct ComponentFields {

    /// Update flags.
    pub(crate) flags: Option<u32>,

    /// Priority (`^+2`).
    pub(crate) priority: Option<f32>,

    /// Ids of the objects the component lists (`^+5`).
    pub(crate) users: Option<Vec<u32>>,

    /// Ownership entries: the linked object, with the entry's values.
    pub(crate) links: Vec<(u32, (f32, f32, u32))>,

    /// Ids of the objects it refers to (`CAI_BDI_COMPONENT_BLOCK_OWNS+1`).
    pub(crate) references: Vec<u32>,
}

/// A manager being written: its ids and paths.
struct ManagerContext<'a> {
    inputs: &'a ManagerInputs<'a>,
    ids: &'a ManagerIds,
    kind: ManagerKind,

    /// The faction's component and AI faction record.
    faction: u32,
    cai_faction: u32,

    /// Desire ids by type.
    desires: HashMap<u32, u32>,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl ManagerKind {

    /// Returns the type codes of the manager's desires, in creation order.
    pub fn desires(self) -> &'static [u32] {
        match self {
            Self::Ai => &FACTION_DESIRES,
            Self::Human => &HUMAN_DESIRES,
            Self::Rebels => &FACTION_DESIRES[..REBEL_DESIRES],
        }
    }

    /// Returns the AI level of the manager.
    pub fn level(self) -> u32 {
        match self {
            Self::Ai => FACTION_LEVEL,
            Self::Human => HUMAN_LEVEL,
            Self::Rebels => REBEL_LEVEL,
        }
    }
}

/// Writes the AI faction managers.
///
/// # Arguments
///
/// * `inputs` - What the managers are written from.
///
/// # Errors
///
/// Returns an error if the layout lacks one of the records or object types.
pub fn write_managers(inputs: &ManagerInputs) -> Result<RecordNode> {
    let path = managers_path();
    let mut list = inputs.layout.builder(&path)?;
    for ids in inputs.plan.managers() {
        let faction = &inputs.plan.factions()[ids.faction()];
        let context = ManagerContext {
            inputs,
            ids,
            faction: faction.faction(),
            cai_faction: faction.cai_faction(),
            kind: inputs.plan.manager_kinds()[ids.faction()],
            desires: inputs.plan.manager_kinds()[ids.faction()].desires().iter().copied().zip(ids.desires().iter().copied()).collect(),
        };
        list.push_group()?
            .set_record("CAI_BDI_POOL", Some(context.write_pool()?))?
            .set_record("CAI_FACTION_BDI_POOL", Some(context.write_pool_slots()?))?
            .set_record("CAI_FACTION_MANAGER", Some(context.write_faction_manager()?))?;
    }
    list.build()
}

/// Writes the component of a pool object from the defaults of its type.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `variant` - Path of the object's type.
/// * `id` - Component id.
/// * `fields` - Fields that differ from the type's defaults.
///
/// # Errors
///
/// Returns an error if the layout lacks the component.
pub(crate) fn pool_component(layout: &StartposLayout, variant: &str, id: u32, fields: ComponentFields) -> Result<RecordNode> {
    let path = child_path(variant, "CAI_BDI_COMPONENT");
    let mut ownership = layout.builder(&child_path(&path, "CAI_BDI_COMPONENT_BLOCK_OWNS"))?;
    for (linked, (first, second, flag)) in &fields.links {
        ownership.push_group()?
            .set_u32("^+1", *linked)?
            .set_u32("^+2", id)?
            .set_f32("^+3", *first)?
            .set_f32("^+4", *second)?
            .set_u32("^+5", *flag)?;
    }

    let mut component = layout.builder(&path)?;
    let group = component.group()?;
    group.set_u32("^+1", id)?
        .set_record("CAI_BDI_COMPONENT_BLOCK_OWNS", Some(ownership.build()?))?
        .set_u32s("CAI_BDI_COMPONENT_BLOCK_OWNS+1", fields.references)?;
    if let Some(users) = fields.users {
        group.set_u32s("^+5", users)?;
    }
    if let Some(flags) = fields.flags {
        group.set_u32("^+4", flags)?;
    }
    if let Some(priority) = fields.priority {
        group.set_f32("^+2", priority)?;
    }
    component.build()
}

/// Writes the analyser of a pool object type with its maps empty, if the type has one. The type's defaults can hold
/// the maps of the game's objects, whose ids mean other objects in a generated startpos.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `variant` - Path of the object's type.
///
/// # Errors
///
/// Returns an error if the analyser's layout lacks its maps.
pub(crate) fn empty_analyser(layout: &StartposLayout, variant: &str) -> Result<Option<RecordNode>> {
    let path = child_path(variant, "CAI_ANALYSER");
    if layout.record(&path).is_none() {
        return Ok(None);
    }
    let mut analyser = layout.builder(&path)?;
    analyser.group()?
        .set_u32s("^+1", vec![])?
        .set("^+2", NodeType::U64Array(vec![]))?;
    analyser.build().map(Some)
}

/// Returns the path of the managers list.
fn managers_path() -> String {
    child_path(CAI_PATH, "CAI_INTERFACE_MANAGERS")
}

/// Returns the order the game lists a map of AI objects in, from their components in insertion order.
pub(crate) fn component_map_order(components: &[u32]) -> Vec<usize> {
    hash_map_order(&components.iter().map(|component| component ^ COMPONENT_HASH_SEED).collect::<Vec<_>>())
}

/// Returns an empty unsigned array leaf, as the game writes them.
fn empty_u32s() -> NodeType {
    NodeType::U32Array(VecU32Node::default())
}

impl ManagerContext<'_> {


    /// Returns the id of a desire of the manager.
    fn desire(&self, code: u32) -> u32 {
        self.desires.get(&code).copied().unwrap_or_default()
    }

    /// Writes the pool of desires, beliefs and intentions.
    fn write_pool(&self) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let path = child_path(&managers_path(), "CAI_BDI_POOL");

        let mut desires = layout.builder(&child_path(&path, "CAI_BDI_POOL_DESIRES"))?;
        let ids = self.ids.desires();
        for index in component_map_order(ids) {
            self.write_desire(&mut desires, self.kind.desires()[index], ids[index])?;
        }

        let mut beliefs = layout.builder(&child_path(&path, "CAI_BDI_POOL_BELIEFS"))?;
        if let Some((financial, strategic)) = self.ids.beliefs() {
            let list = [(FINANCIAL_ANALYSIS, financial), (STRATEGIC_CONTEXT_ANALYSIS, strategic)];
            for index in component_map_order(&[financial, strategic]) {
                self.write_belief(&mut beliefs, list[index].0, list[index].1)?;
            }
        }

        let mut intentions = layout.builder(&child_path(&path, "CAI_BDI_POOL_INTENTIONS"))?;
        let position = self.ids.faction();
        let characters = self.inputs.characters.get(position).map(Vec::as_slice).unwrap_or_default();
        let mobiles = self.inputs.mobiles.get(position).map(Vec::as_slice).unwrap_or_default();
        let list = std::iter::once((FACTION_INTENTION, self.ids.intention(), None))
            .chain(self.ids.character_intentions().iter().zip(characters).map(|(id, target)| (CHARACTER_SKILLS_INTENTION, *id, Some(*target))))
            .chain(self.ids.mobile_intentions().iter().zip(mobiles).map(|(id, target)| (FORCE_SKILLS_INTENTION, *id, Some(*target))))
            .collect::<Vec<_>>();
        for index in component_map_order(&list.iter().map(|(_, id, _)| *id).collect::<Vec<_>>()) {
            let (code, id, target) = list[index];
            self.write_intention(&mut intentions, code, id, target)?;
        }

        let mut pool = layout.builder(&path)?;
        let group = pool.group()?;
        group.set_record("CAI_BDI_POOL_BELIEFS", Some(beliefs.build()?))?
            .set_record("CAI_BDI_POOL_DESIRES", Some(desires.build()?))?
            .set_record("CAI_BDI_POOL_INTENTIONS", Some(intentions.build()?))?;
        if self.kind != ManagerKind::Ai {
            group.set_u32("CAI_BDI_POOL_FAILED+1", NON_AI_POOL_VALUE)?;
        }
        pool.build()
    }

    /// Writes a desire, with the references of its type.
    fn write_desire(&self, desires: &mut RecordBuilder, code: u32, id: u32) -> Result<()> {
        let layout = self.inputs.layout;
        let variant = variant_path(&child_path(&managers_path(), "CAI_BDI_POOL/CAI_BDI_POOL_DESIRES"), code);
        let analysed_belief = match code {
            FINANCIAL_ANALYSER => self.ids.beliefs().map(|(financial, _)| financial),
            STRATEGIC_ANALYSER => self.ids.beliefs().map(|(_, strategic)| strategic),
            _ => None,
        };
        let links = match code {
            TARGET_LIST => vec![(self.desire(TMS_COORDINATION), OWNER_LINK)],
            FORCE_SKILLS => self.ids.mobile_intentions().iter().map(|intention| (*intention, INTENTION_LINK)).collect(),
            CHARACTER_SKILLS => self.ids.character_intentions().iter().map(|intention| (*intention, INTENTION_LINK)).collect(),
            _ => analysed_belief.map(|belief| (belief, OWNER_LINK)).into_iter().collect(),
        };
        let wars = self.wars();
        let flags = match code {
            FINANCE_MANAGER => Some(FRESH_FLAGS),
            DIPLOMACY_MANAGER => Some(if wars.is_empty() { DIPLOMACY_FLAGS.1 } else { DIPLOMACY_FLAGS.0 }),
            _ => None,
        };
        let users = CAI_FACTION_USERS.contains(&code).then(|| vec![self.cai_faction]);
        let human = self.kind == ManagerKind::Human;
        let priority = HUMAN_PRIORITIES.iter().find(|(human_code, _)| human && *human_code == code).map(|(_, priority)| *priority);
        let component = pool_component(layout, &variant, id, ComponentFields { flags, priority, users, links, references: vec![] })?;

        let group = desires.push_variant(code)?;
        group.set_record("CAI_BDI_COMPONENT", Some(component))?;
        match code {
            TARGET_LIST => self.set_single(group, &variant, "CAI_MANOEUVRE_AND_TARGET_LIST", "^+1", self.desire(TMS_COORDINATION))?,
            FINANCE_MANAGER => { group.set_record("CAI_FACTION_BDI_POOL_FINANCE_MANAGER_BASIC", Some(self.write_finance_manager(&variant)?))?; },
            RESEARCH => self.set_single(group, &variant, "CAI_BDI_DESIRE_RESEARCH_TECHNOLOGY", "^+1", self.faction)?,
            TAXATION => self.set_single(group, &variant, "CAI_BDI_DESIRE_TAXATION", "^+1", self.faction)?,
            FINANCIAL_ANALYSER | STRATEGIC_ANALYSER => {
                let mut analyser = layout.builder(&child_path(&variant, "CAI_ANALYSER"))?;
                analyser.group()?
                    .set_u32s("^+1", analysed_belief.map(|belief| vec![self.faction, belief]).unwrap_or_default())?
                    .set("^+2", NodeType::U64Array(vec![]))?;
                group.set_record("CAI_ANALYSER", Some(analyser.build()?))?;
            },
            TMS_COORDINATION => {
                let mut properties = layout.builder(&child_path(&variant, "CAI_BDI_COMPONENT_PROPERTY_SET"))?;
                properties.group()?.set_u32("^+12", self.faction)?;
                let mut coordination = layout.builder(&child_path(&variant, "CAI_TMS_COORDINATION_MANAGER"))?;
                coordination.group()?
                    .set_u32("^+1", self.faction)?
                    .set_u32("^+2", self.desire(TARGET_LIST))?;
                group.set_record("CAI_BDI_COMPONENT_PROPERTY_SET", Some(properties.build()?))?
                    .set_record("CAI_TMS_COORDINATION_MANAGER", Some(coordination.build()?))?;
            },
            STANCE_MANAGER => { group.set_record("CAI_STRATEGIC_STANCE_MANAGER", Some(self.write_stance_manager(&variant)?))?; },
            TASK_GENERATOR => {
                let mut generator = layout.builder(&child_path(&variant, "CAI_TASK_GENERATOR_MANAGER"))?;
                generator.group()?.set_bool("^+4", human)?;
                group.set_record("CAI_TASK_GENERATOR_MANAGER", Some(generator.build()?))?;
            },
            DIPLOMACY_MANAGER => { group.set_record("CAI_DIPLOMACY_MANAGER", Some(self.write_diplomacy_manager(&variant, &wars)?))?; },
            _ => if let Some(analyser) = empty_analyser(layout, &variant)? {
                group.set_record("CAI_ANALYSER", Some(analyser))?;
            },
        }
        Ok(())
    }

    /// Writes a belief about the faction: the financial one in its constructor state, the strategic context one in
    /// the most common state of the game's.
    fn write_belief(&self, beliefs: &mut RecordBuilder, code: u32, id: u32) -> Result<()> {
        let layout = self.inputs.layout;
        let variant = variant_path(&child_path(&managers_path(), "CAI_BDI_POOL/CAI_BDI_POOL_BELIEFS"), code);
        let key = if code == FINANCIAL_ANALYSIS { "CAI_FINANCIAL_ANALYSIS" } else { "CAI_STRATEGIC_CONTEXT_ANALYSIS" };
        let references = if code == FINANCIAL_ANALYSIS { vec![self.desire(FINANCE_MANAGER)] } else { vec![] };
        let component = pool_component(layout, &variant, id, ComponentFields { flags: Some(FRESH_FLAGS), references, ..Default::default() })?;

        let mut properties = layout.builder(&child_path(&variant, "CAI_BDI_COMPONENT_PROPERTY_SET"))?;
        properties.group()?.set_u32("^+9", self.faction)?;
        let mut analysis = layout.builder(&child_path(&variant, key))?;
        let group = analysis.group()?;
        group.set_u32("^+1", self.faction)?;
        if code == FINANCIAL_ANALYSIS {
            group.set_f32("^+2", 0.0)?.set_f32("^+3", 0.0)?;
        } else {
            group.set_u32("^+2", STRATEGIC_CONTEXT)?.set_u32("^+3", 0)?;
        }

        beliefs.push_variant(code)?
            .set_record("CAI_BDI_COMPONENT_PROPERTY_SET", Some(properties.build()?))?
            .set_record("CAI_BDI_COMPONENT", Some(component))?
            .set_record(key, Some(analysis.build()?))?;
        Ok(())
    }

    /// Writes an intention: the faction's one (no target), or the skill management of a character or mobile.
    fn write_intention(&self, intentions: &mut RecordBuilder, code: u32, id: u32, target: Option<u32>) -> Result<()> {
        let layout = self.inputs.layout;
        let variant = variant_path(&child_path(&managers_path(), "CAI_BDI_POOL/CAI_BDI_POOL_INTENTIONS"), code);
        let links = target.map(|target| vec![(target, OWNER_LINK)]).unwrap_or_default();
        let component = pool_component(layout, &variant, id, ComponentFields { links, ..Default::default() })?;
        let group = intentions.push_variant(code)?;
        group.set_record("CAI_BDI_COMPONENT", Some(component))?;
        if let Some(target) = target {
            let mut properties = layout.builder(&child_path(&variant, "CAI_BDI_COMPONENT_PROPERTY_SET"))?;
            properties.group()?.set_u32("^+12", target)?;
            group.set_record("CAI_BDI_COMPONENT_PROPERTY_SET", Some(properties.build()?))?;

            let key = if code == CHARACTER_SKILLS_INTENTION { "CAI_BDI_CHARACTER_SKILL_MANAGEMENT_FOR_CHARACTER" } else { "CAI_BDI_FORCE_LEGACY_SKILL_MANAGEMENT_FOR_FORCE" };
            self.set_single(group, &variant, key, "^+1", target)?;
        }
        Ok(())
    }

    /// Sets a child record of a pool object holding one reference.
    fn set_single(&self, group: &mut GroupBuilder, variant: &str, key: &str, field: &str, value: u32) -> Result<()> {
        let mut record = self.inputs.layout.builder(&child_path(variant, key))?;
        record.group()?.set_u32(field, value)?;
        group.set_record(key, Some(record.build()?))?;
        Ok(())
    }

    /// Returns the components of the factions the manager's faction is at war with, sorted.
    fn wars(&self) -> Vec<u32> {
        let position = self.ids.faction();
        self.inputs.plan.factions().iter().enumerate()
            .filter(|(other, _)| self.inputs.stances.at_war(position, *other))
            .map(|(_, ids)| ids.faction())
            .collect()
    }

    /// Writes the finance manager in its constructor state: the first update recomputes it from the faction's
    /// personality and treasury.
    fn write_finance_manager(&self, variant: &str) -> Result<RecordNode> {
        let mut record = self.inputs.layout.builder(&child_path(variant, "CAI_FACTION_BDI_POOL_FINANCE_MANAGER_BASIC"))?;
        let mut repeated = VecI32Node::default();
        repeated.set_value(vec![2; BUDGET_CATEGORIES]).set_optimized(true);
        let mut never = VecU32Node::default();
        never.set_value(vec![0; BUDGET_CATEGORIES]).set_optimized(true);
        record.group()?
            .set_u32s("^+1", vec![])?
            .set_u32("^+2", u32::MAX)?
            .set_u32s("^+3", vec![])?
            .set_f32("^+4", 0.0)?
            .set_i32("^+5", -1)?
            .set_f32("^+6", 0.0)?
            .set_bool("^+7", false)?
            .set_f32("^+8", -1.0)?
            .set_f32("^+9", -1.0)?
            .set_f32("^+10", 0.0)?
            .set_f32("^+11", 0.0)?
            .set_f32("^+12", 0.0)?
            .set("^+13", NodeType::F32Array(vec![1.0 / BUDGET_CATEGORIES as f32; BUDGET_CATEGORIES]))?
            .set("^+14", NodeType::F32Array(vec![0.0; BUDGET_CATEGORIES]))?
            .set("^+15", NodeType::BoolArray(vec![false; BUDGET_CATEGORIES]))?
            .set("^+16", NodeType::U32Array(never))?
            .set("^+17", NodeType::F32Array(vec![1000.0; BUDGET_CATEGORIES]))?
            .set("^+18", NodeType::F32Array(vec![0.0; BUDGET_CATEGORIES]))?
            .set("^+19", NodeType::F32Array(vec![0.0; BUDGET_CATEGORIES]))?
            .set("^+20", NodeType::I32Array(repeated))?
            .set_u32("^+21", 5)?
            .set_u32("^+22", 5)?;
        record.build()
    }

    /// Writes the strategic stance manager: the factions marked for re-evaluation, and the empire data in its
    /// constructor state (the game computes it after loading).
    fn write_stance_manager(&self, variant: &str) -> Result<RecordNode> {
        let layout = self.inputs.layout;
        let path = child_path(variant, "CAI_STRATEGIC_STANCE_MANAGER");
        let factions = self.inputs.plan.factions();
        let marks = self.inputs.stances.marks(self.ids.faction()).iter().map(|position| factions[*position].faction()).collect::<Vec<_>>();
        let mut blocking = layout.builder(&child_path(&path, "CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION"))?;
        let adjustments_path = child_path(&child_path(&path, "CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION"), "CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION_SCORE_ADJUSTMENT");
        for index in component_map_order(&marks) {
            let mut adjustments = layout.builder(&adjustments_path)?;
            for _ in 0..STANCE_SCORES {
                adjustments.push_group()?;
            }
            blocking.push_group()?
                .set_u32("^+1", marks[index])?
                .set_record("CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION_SCORE_ADJUSTMENT", Some(adjustments.build()?))?;
        }

        let mut empire = layout.builder(&child_path(&path, "CAI_STRATEGIC_STANCE_MANAGER_EMPIRE_DATA"))?;
        let group = empire.group()?;
        for field in 1..=6 {
            group.set_u32(&format!("^+{field}"), 0)?;
        }
        for field in 7..=12 {
            group.set_f32(&format!("^+{field}"), 0.5)?;
        }

        let mut manager = layout.builder(&path)?;
        manager.group()?
            .set_u32("^+1", self.faction)?
            .set_record("CAI_FACTION_STANCE_BLOCKING_AND_PROMOTION", Some(blocking.build()?))?
            .set_record("CAI_STRATEGIC_STANCE_MANAGER_EMPIRE_DATA", Some(empire.build()?))?
            .set_u32("CAI_STRATEGIC_STANCE_MANAGER_EMPIRE_DATA+1", u32::MAX)?;
        manager.build()
    }

    /// Writes the diplomacy manager: an entry per faction at war with the manager's one.
    fn write_diplomacy_manager(&self, variant: &str, wars: &[u32]) -> Result<RecordNode> {
        let mut manager = self.inputs.layout.builder(&child_path(variant, "CAI_DIPLOMACY_MANAGER"))?;
        let group = manager.group()?;
        group.push_child(i32_node(if wars.is_empty() { NO_WARS_VALUE } else { 0 }))?
            .push_child(u32_node(wars.len() as u32))?
            .push_child(u32_node(DIPLOMACY_ENTRY_PAIRS + 1))?;
        for enemy in wars {
            group.push_child(u32_node(*enemy))?
                .push_child(i32_node(3))?
                .push_child(i32_node(0))?;
            for _ in 0..DIPLOMACY_ENTRY_PAIRS {
                group.push_child(i32_node(-1))?.push_child(i32_node(i32::MAX))?;
            }
        }
        group.push_child(u32_node(if wars.is_empty() { 0 } else { DIPLOMACY_WAR_STATE }))?
            .push_child(bool_node(false))?
            .push_child(empty_u32s())?
            .push_child(empty_u32s())?
            .push_child(u32_node(0))?;
        manager.build()
    }

    /// Writes the slots of the pool holding its main desires.
    fn write_pool_slots(&self) -> Result<RecordNode> {
        let mut record = self.inputs.layout.builder(&child_path(&managers_path(), "CAI_FACTION_BDI_POOL"))?;
        let group = record.group()?;
        for slot in 3..=LAST_POOL_SLOT {
            let desire = POOL_SLOTS.iter().find(|(pool_slot, _)| *pool_slot == slot).map(|(_, code)| self.desire(*code)).unwrap_or_default();
            group.set_u32(&format!("^+{slot}"), desire)?;
        }
        record.build()
    }

    /// Writes the manager's faction and AI level.
    fn write_faction_manager(&self) -> Result<RecordNode> {
        let mut record = self.inputs.layout.builder(&child_path(&managers_path(), "CAI_FACTION_MANAGER"))?;
        record.group()?
            .set_u32("^+1", self.faction)?
            .set_u32("^+2", self.kind.level())?
            .set_bool("CAI_FACTION_LEARNT_PARAMETERS+1", self.kind != ManagerKind::Rebels)?
            .set_u32("CAI_FACTION_LEARNT_PARAMETERS+2", 0)?
            .set_u32("CAI_FACTION_LEARNT_PARAMETERS+3", 0)?;
        record.build()
    }
}

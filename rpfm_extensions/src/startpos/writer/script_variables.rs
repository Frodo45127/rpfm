//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Writer of the restrictions the campaign's scripts manage (`EPISODIC_RESTRICTIONS`). Only the variables the scripts
//! save (`SCRIPT_VARIABLES`) are generated, from the values they save when the game builds the startpos. Each variable
//! is a record named after it, holding the type code of its value and the value.

use rpfm_lib::error::Result;
use rpfm_lib::files::esf::{NodeType, RecordNode, RecordNodeFlags};

use super::{bool_node, i32_node, u32_node, MODEL_PATH};
use crate::lua::harness::LuaSavedValue;
use crate::startpos::layout::{child_path, StartposLayout};

#[cfg(test)] #[path = "script_variables_test.rs"] mod script_variables_test;

/// Name of the record with the script variables.
const SCRIPT_VARIABLES: &str = "SCRIPT_VARIABLES";

/// Type codes of the saved values.
const BOOL_TYPE: u32 = 0;
const INTEGER_TYPE: u32 = 1;
const STRING_TYPE: u32 = 2;

/// Writes the episodic restrictions.
///
/// # Arguments
///
/// * `layout` - Record layouts.
/// * `variables` - Values saved by the campaign's scripts, in order, or `None` to keep the layout's.
///
/// # Errors
///
/// Returns an error if the layout lacks the records.
pub(super) fn write_episodic_restrictions(layout: &StartposLayout, variables: Option<&[(String, LuaSavedValue)]>) -> Result<RecordNode> {
    let path = child_path(MODEL_PATH, "EPISODIC_RESTRICTIONS");
    let mut restrictions = layout.builder(&path)?;
    if let Some(variables) = variables {
        let children = variables.iter().map(|(name, value)| NodeType::Record(Box::new(variable_record(name, value)))).collect();
        let record = layout.raw_record(&child_path(&path, SCRIPT_VARIABLES), children)?;
        restrictions.group()?.set_record(SCRIPT_VARIABLES, Some(record))?;
    }
    restrictions.build()
}

/// Returns the record of a script variable.
fn variable_record(name: &str, value: &LuaSavedValue) -> RecordNode {
    let (type_code, value) = match value {
        LuaSavedValue::Bool(value) => (BOOL_TYPE, bool_node(*value)),
        LuaSavedValue::Integer(value) => (INTEGER_TYPE, i32_node(*value)),
        LuaSavedValue::String(value) => (STRING_TYPE, NodeType::Ascii(value.to_owned())),
    };

    let mut record = RecordNode::default();
    record.set_name(name.to_owned());
    record.set_record_flags(RecordNodeFlags::IS_RECORD_NODE);
    record.set_children(vec![vec![u32_node(type_code), value]]);
    record
}

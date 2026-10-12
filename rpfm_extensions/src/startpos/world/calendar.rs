//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! The calendar of a campaign (`start_pos_calendars`).

use getset::{CopyGetters, Getters};

use rpfm_lib::error::{RLibError, Result};

use crate::startpos::tables::StartposTables;

#[cfg(test)] #[path = "calendar_test.rs"] mod calendar_test;

//---------------------------------------------------------------------------//
//                              Types
//---------------------------------------------------------------------------//

/// When a campaign starts and how many turns its years have.
#[derive(Debug, Clone, PartialEq, Eq, Default, Getters, CopyGetters)]
pub struct Calendar {

    /// Year of the first turn, as the game counts them (Attila counts from the founding of Rome).
    #[getset(get_copy = "pub")]
    start_year: u32,

    /// Turns per year.
    #[getset(get_copy = "pub")]
    turns_per_year: u32,

    /// Season of the first turn (`seasons` key).
    #[getset(get = "pub")]
    start_season: String,
}

//---------------------------------------------------------------------------//
//                              Implementations
//---------------------------------------------------------------------------//

impl Calendar {

    /// Reads the calendar of a campaign.
    ///
    /// # Errors
    ///
    /// Returns an error if the campaign has no calendar, or the table lacks a column.
    pub fn new(tables: &StartposTables, campaign: &str) -> Result<Self> {
        for row in tables.table("start_pos_calendars")?.rows() {
            if row.str("campaign")? == campaign {
                return Ok(Self {
                    start_year: u32::try_from(row.int("start_year")?).unwrap_or_default(),
                    turns_per_year: u32::try_from(row.int("turns_per_year")?).unwrap_or_default(),
                    start_season: row.str("start_season")?.into_owned(),
                });
            }
        }
        Err(RLibError::StartposInvalidValue("start_pos_calendars".to_owned(), "campaign".to_owned(), campaign.to_owned()))
    }
}

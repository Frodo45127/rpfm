//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Generation of campaign startpos files (`startpos.esf`) without the game.
//!
//! The game builds a startpos by creating a new campaign from the `start_pos_*` tables, the DB and the campaign
//! map files, running its normal campaign initialisation, and saving the result with its save game code. This
//! module reproduces that process piece by piece, so startpos generation can report clear errors instead of
//! failing silently inside the game.
//!
//! Each submodule builds one part of the startpos and is verified against startposes built by the game
//! (Attila's main campaign for now). Parts not covered yet are listed in each submodule's docs.
//!
//! # Submodules
//!
//! - [`coords`]: coordinate systems of the campaign map (world, raw map units, hex cells).
//! - [`trade`]: the trade network (`CAMPAIGN_TRADE_MANAGER` splines and routes), from `trade_routes.ptd`.
//! - [`settlements`]: settlement layouts (slots and sprawl cells of each region).

pub mod coords;
mod esf_nav;
pub mod settlements;
pub mod trade;

#[cfg(test)] mod startpos_test;

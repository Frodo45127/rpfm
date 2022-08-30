//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains integrations of this crate with another tools.
//!
//! The following integrations are included:
//! - **Assembly Kit**: enables importing tables from the Assembly Kit.
//!   Requires the feature `integration_assembly_kit` to be enabled.
//! - **Git**: enables basic management of git repositories. Requires the feature
//!   `integration_git` to be enabled.
//! - **Log**: enables logging and automatic upload crash reports. Requires the
//!   feature `integration_log` to be enabled.
//!
//! Each integration is opt-in, so you can ignore them unless you really want to use them.

#[cfg(feature = "integration_assembly_kit")] pub mod assembly_kit;
#[cfg(feature = "integration_git")] pub mod git;
#[cfg(feature = "integration_log")] pub mod log;
#[cfg(feature = "integration_sqlite")] pub mod sqlite;

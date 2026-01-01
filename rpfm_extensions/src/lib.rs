//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This crate contains certain functionality extensions that, for one reason or another, didn't fit in the main RPFM lib crate.

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::too_many_arguments,             // Disabled because it gets annoying really quick.
    clippy::field_reassign_with_default,    // Disabled because it gets annoying on tests.
    clippy::assigning_clones,
    clippy::type_complexity,
)]

use std::{sync::{mpsc::Sender, Arc, LazyLock, RwLock}, thread::JoinHandle};

pub mod dependencies;
pub mod diagnostics;
pub mod gltf;
pub mod optimizer;
pub mod search;
pub mod translator;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Variable to keep the background thread for the startpos generation working.
static START_POS_WORKAROUND_THREAD: LazyLock<Arc<RwLock<Option<Vec<(Sender<bool>, JoinHandle<()>)>>>>> = LazyLock::new(|| Arc::new(RwLock::new(None)));

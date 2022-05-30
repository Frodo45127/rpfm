//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// These consts are used for dealing with Time-related operations.
const WINDOWS_TICK: u64 = 10_000_000;
const SEC_TO_UNIX_EPOCH: u64 = 11_644_473_600;

mod pfh6;
mod pfh5;
mod pfh4;
mod pfh3;
mod pfh2;
mod pfh0;

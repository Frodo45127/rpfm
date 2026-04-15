//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

/// Shared data types for v1 (Shogun 2) group formations.
pub mod v1;

/// Shared data types for v2 (Rome 2 and later) group formations.
pub mod v2;

/// Per-game decode/encode implementations.
pub mod troy;
pub mod warhammer_3;
pub mod rome_2;
pub mod shogun_2;

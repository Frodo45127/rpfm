//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Version-specific decoders and encoders for animation fragment battle files.
//!
//! This module contains the implementation details for different format versions:
//! - `v2_3k`: Version 2 format for Three Kingdoms (with locomotion graph support)
//! - `v2_wh2`: Version 2 format for Warhammer 2, Troy, and Pharaoh
//! - `v4`: Version 4 format for Warhammer 3 (with animation references)

mod v2_3k;
mod v2_wh2;
mod v4;

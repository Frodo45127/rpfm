//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the [Optimizable] and [OptimizableContainer] trait.

use rpfm_lib::error::Result;
use rpfm_lib::files::{Container, db::DB, loc::Loc, pack::Pack};

use crate::dependencies::Dependencies;

//-------------------------------------------------------------------------------//
//                             Trait definitions
//-------------------------------------------------------------------------------//

/// This trait marks an struct (mainly structs representing decoded files) as `Optimizable`, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait Optimizable {

    /// This function optimizes the provided struct to reduce its size and improve compatibility.
    ///
    /// It returns if the struct has been left in an state where it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies) -> bool;
}

/// This trait marks a [Container](rpfm_lib::files::Container) as an `Optimizable` container, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait OptimizableContainer: Container {

    /// This function optimizes the provided [Container](rpfm_lib::files::Container) to reduce its size and improve compatibility.
    ///
    /// It returns the list of files that has been safetly deleted during the optimization process.
    fn optimize(&mut self, dependencies: &mut Dependencies) -> Result<Vec<String>>;
}

//-------------------------------------------------------------------------------//
//                           Trait implementations
//-------------------------------------------------------------------------------//

impl OptimizableContainer for Pack {
    fn optimize(&mut self, dependencies: &mut Dependencies) -> Result<Vec<String>> {
        todo!()
    }
}

impl Optimizable for DB {
    fn optimize(&mut self, dependencies: &mut Dependencies) -> bool {
        todo!()
    }
}

impl Optimizable for Loc {
    fn optimize(&mut self, dependencies: &mut Dependencies) -> bool {
        todo!()
    }
}

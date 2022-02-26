//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code to setup the tips (in the `StatusBar`) for the actions in `PackFileContentsUI`.
!*/

use crate::locale::qtr;
use std::rc::Rc;

use super::PackFileContentsUI;

/// This function sets the status bar tip for all the actions in the provided `PackFileContentsUI`.
pub unsafe fn set_tips(ui: &Rc<PackFileContentsUI>) {

    //---------------------------------------------------//
    // PackFile Contents panel tips.
    //---------------------------------------------------//
    ui.filter_autoexpand_matches_button.set_status_tip(&qtr("tt_filter_autoexpand_matches_button"));
    ui.filter_case_sensitive_button.set_status_tip(&qtr("tt_filter_case_sensitive_button"));
}

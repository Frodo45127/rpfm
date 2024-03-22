//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains all kind of errors caused by tools.
//!
//! Not much to say appart of that, really.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolsError {

    #[error("<p>This tool is not supported for the currently selected game.</p>")]
    GameSelectedNotSupportedForTool,

    #[error("<p>There is no Schema for the Game Selected.</p>")]
    SchemaNotFound,

    #[error("<p>The dependencies cache for the Game Selected is either missing, outdated, or it was generated without the Assembly Kit. Please, re-generate it and try again.</p>")]
    DependenciesCacheNotGeneratedorOutOfDate,

    #[error("<p>One of the columns we need is not in the table we're searching for. This means either the tool needs updating, or you have some weird tables there.</p>")]
    ToolTableColumnNotFound,

    #[error("<p>TBRemoved</p>")]
    Impossibru,

    #[error("<p>One of the widgets of this view has not been found in the UI Template. This means either the code is wrong, or the template is incomplete/outdated.</p><p>The missing widgets are: {0}</p>")]
    TemplateUIWidgetNotFound(String),

    #[error("<p>The following values hasn't been found for this entry: {0}.</p>")]
    ToolEntryDataNotFound(String),

    #[error("<p>One of the columns we need is not of the type we expected. This means either the tool needs updating, or you have some weird tables there.</p>")]
    ToolTableColumnNotOfTypeWeExpected,

    #[error("<p>The following tool variable hasn't been found for the current Game Selected: {0}.</p>")]
    ToolVarNotFoundForGame(String),

    #[error("<p>Missing column {1} in table {0}. This is caused either by an outdated definition, or a bug.</p>")]
    MissingColumnInTable(String, String),

    #[error("<p>{0}</p>")]
    GenericError(String),
}

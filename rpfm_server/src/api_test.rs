//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the dispatch of API methods.

use super::*;

#[test]
fn job_methods_are_the_ones_marked_as_jobs() {
    const { assert!(SetGame::IS_JOB && GenerateDependenciesCache::IS_JOB && RebuildDependencies::IS_JOB && RunDiagnostics::IS_JOB && RunSearch::IS_JOB) };
    const { assert!(UpdateSchemas::IS_JOB && UpdateSchemaFromAssemblyKit::IS_JOB && OptimizePack::IS_JOB && RunLuaTests::IS_JOB) };
    assert!(JOB_METHODS.iter().all(|method| is_job_method(method)));
    assert!(!is_job_method(GetSessionStatus::METHOD) && !GetSessionStatus::IS_JOB);
}

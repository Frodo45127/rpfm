//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for utility functions.

use crate::utils::*;

#[test]
fn test_sanitize_filename() {
    // Test invalid Windows characters.
    assert_eq!(sanitize_filename("*file_name"), "_file_name");
    assert_eq!(sanitize_filename("file<name>"), "file_name_");
    assert_eq!(sanitize_filename("file:name"), "file_name");
    assert_eq!(sanitize_filename("file\"name"), "file_name");
    assert_eq!(sanitize_filename("file/name"), "file_name");
    assert_eq!(sanitize_filename("file\\name"), "file_name");
    assert_eq!(sanitize_filename("file|name"), "file_name");
    assert_eq!(sanitize_filename("file?name"), "file_name");

    // Test leading/trailing spaces and dots.
    assert_eq!(sanitize_filename("  filename  "), "filename");
    assert_eq!(sanitize_filename("...filename..."), "filename");

    // Test empty result after sanitization.
    assert_eq!(sanitize_filename("***"), "___");
    assert_eq!(sanitize_filename(""), "unnamed_file");
    assert_eq!(sanitize_filename("   "), "unnamed_file");
    assert_eq!(sanitize_filename("..."), "unnamed_file");

    // Test valid filenames (should remain unchanged).
    assert_eq!(sanitize_filename("valid_filename"), "valid_filename");
    assert_eq!(sanitize_filename("file.name"), "file.name");
}

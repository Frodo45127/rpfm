//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, &which can be &found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::type_complexity,
    clippy::missing_safety_doc,
    clippy::arc_with_non_send_sync,
)]

use qt_core::QCoreApplication;

use lazy_static::lazy_static;
use time::format_description::{parse, FormatItem};

use std::path::PathBuf;

use crate::locale::Locale;
use crate::settings::*;

pub mod icons;
pub mod locale;
pub mod settings;
pub mod tools;
pub mod utils;

/// This macro is used to clone the variables into the closures without the compiler complaining.
///
/// Mainly for use with UI stuff, but you can use it with anything clonable.
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($y:ident $n:ident),+ => move || $body:expr) => (
        {
            $( #[allow(unused_mut)] let mut $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
    ($($y:ident $n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( #[allow(unused_mut)] let mut $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

lazy_static!{

    /// Path were the stuff used by RPFM (settings, schemas,...) is. In debug mode, we just take the current path
    /// (so we don't break debug builds). In Release mode, we take the `.exe` path.
    #[derive(Debug)]
    pub static ref PROGRAM_PATH: PathBuf = if cfg!(debug_assertions) {
        std::env::current_dir().unwrap()
    } else {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path
    };

    /// Path that contains the extra assets we need, like images.
    #[derive(Debug)]
    pub static ref ASSETS_PATH: PathBuf = if cfg!(debug_assertions) {
        PROGRAM_PATH.to_path_buf()
    } else {
        // For release builds:
        // - Windows: Same as RFPM exe.
        // - Linux: /usr/share/rpfm.
        // - MacOs: Who knows?
        if cfg!(target_os = "linux") {
            PathBuf::from("/usr/share/".to_owned() + unsafe { &QCoreApplication::application_name().to_std_string() })
        } else {
            PROGRAM_PATH.to_path_buf()
        }
    };

    /// Variable to keep the locale fallback data (english locales) used by the UI loaded and available.
    static ref LOCALE_FALLBACK: Locale = {
        match Locale::initialize_fallback() {
            Ok(locale) => locale,
            Err(_) => Locale::initialize_empty(),
        }
    };

    /// Variable to keep the locale data used by the UI loaded and available. If we fail to load the selected locale data, copy the english one instead.
    static ref LOCALE: Locale = {
        let language = setting_string("language");
        if !language.is_empty() {
            Locale::initialize(&language).unwrap_or_else(|_| LOCALE_FALLBACK.clone())
        } else {
            LOCALE_FALLBACK.clone()
        }
    };

    /// Formatted date, so we can reuse it instead of re-parsing it on each use.
    pub static ref FULL_DATE_FORMAT: Vec<FormatItem<'static>> = parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
    pub static ref SLASH_DMY_DATE_FORMAT: Vec<FormatItem<'static>> = parse("[day]/[month]/[year]").unwrap();
    pub static ref SLASH_MDY_DATE_FORMAT: Vec<FormatItem<'static>> = parse("[month]/[day]/[year]").unwrap();

}

pub const ROOT_NODE_TYPE: i32 = 23;
pub const ROOT_NODE_TYPE_EDITABLE_PACKFILE: i32 = 0;

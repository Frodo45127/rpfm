//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

// Disabled `Clippy` linters, with the reasons why they were disabled.
#![allow(
    clippy::type_complexity,
    clippy::missing_safety_doc,
    clippy::arc_with_non_send_sync,
)]

use time::format_description::{parse, FormatItem};

use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock};

pub mod icons;
pub mod locale;
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

/// Path were the stuff used by RPFM (settings, schemas,...) is. In debug mode, we just take the current path
/// (so we don't break debug builds). In Release mode, we take the `.exe` path.
pub static PROGRAM_PATH: LazyLock<PathBuf> = LazyLock::new(|| if cfg!(debug_assertions) {
    std::env::current_dir().unwrap()
} else {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path
});

/// Path that contains the extra assets we need, like images.
pub static ASSETS_PATH: LazyLock<PathBuf> = LazyLock::new(|| if cfg!(debug_assertions) {
    PROGRAM_PATH.to_path_buf()
} else {
    // For release builds:
    // - Windows: Same as RFPM exe.
    // - Linux: /usr/share/rpfm.
    // - MacOs: Who knows?
    if cfg!(target_os = "linux") {
        PathBuf::from("/usr/share/".to_owned() + &APP_NAME.read().unwrap())
    } else {
        PROGRAM_PATH.to_path_buf()
    }
});

/// Formatted date, so we can reuse it instead of re-parsing it on each use.
pub static FULL_DATE_FORMAT: LazyLock<Vec<FormatItem<'static>>> = LazyLock::new(|| parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap());
pub static SLASH_DMY_DATE_FORMAT: LazyLock<Vec<FormatItem<'static>>> = LazyLock::new(|| parse("[day]/[month]/[year]").unwrap());
pub static SLASH_MDY_DATE_FORMAT: LazyLock<Vec<FormatItem<'static>>> = LazyLock::new(|| parse("[month]/[day]/[year]").unwrap());

pub static ORG_DOMAIN: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::from("com"))));
pub static ORG_NAME: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::from("FrodoWazEre"))));
pub static APP_NAME: LazyLock<Arc<RwLock<String>>> = LazyLock::new(|| Arc::new(RwLock::new(String::from("rpfm"))));

pub const ROOT_NODE_TYPE: i32 = 23;
pub const ROOT_NODE_TYPE_EDITABLE_PACKFILE: i32 = 0;

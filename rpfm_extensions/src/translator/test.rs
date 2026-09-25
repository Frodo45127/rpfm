//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Round-trip tests for the on-disk translation format.
//!
//! These cover both supported wire versions (v0 legacy and v1 current) and the
//! conversions between them. The fixtures are tiny — three entries is enough
//! to exercise every code path in [`PackTranslation::save`] / [`load_json`]
//! without making the test brittle.
//!
//! The `dst` character substitutions [`PackTranslation::load`] performs
//! (`\n||\n` → `||`, then `\r`/`\n`/`\t` → escaped two-char forms) are covered
//! by [`load_substitutes_special_chars_in_dst`] and
//! [`save_writes_dst_verbatim`]. These document the asymmetric round-trip:
//! load rewrites the bytes, save doesn't, so values containing those
//! characters change shape after the first load. Every other test in this
//! file uses plain ASCII so the substitutions are no-ops.
//!
//! What we don't cover here:
//!
//! - [`PackTranslation::new`], which depends on `Pack` and `Dependencies`
//!   instances we'd need to mock heavily for an integration-level test.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use tempfile::TempDir;

use super::*;

const GAME: &str = "warhammer_3";
const PACK: &str = "test_pack.pack";
const SRC_LANG: &str = "EN";
const LANG: &str = "ES";

/// Build an in-memory [`PackTranslation`] populated with a small but realistic
/// fixture covering every field the v1 format carries.
fn sample_v1() -> PackTranslation {
    let mut translations = HashMap::new();
    translations.insert("greeting".to_owned(), Translation {
        src: "Hello".to_owned(),
        dst: "Hola".to_owned(),
        retr: false,
        rem: false,
        aut: false,
    });
    translations.insert("farewell".to_owned(), Translation {
        src: "Goodbye".to_owned(),
        dst: "Adios".to_owned(),
        retr: true,
        rem: false,
        aut: true,
    });
    translations.insert("legacy_string".to_owned(), Translation {
        src: "Old".to_owned(),
        dst: "Viejo".to_owned(),
        retr: false,
        rem: true,
        aut: false,
    });

    let mut glossary = BTreeMap::new();
    glossary.insert("Empire".to_owned(), "Imperio".to_owned());
    glossary.insert("Faction".to_owned(), "Faccion".to_owned());

    PackTranslation {
        version: CURRENT_VERSION,
        language: LANG.to_owned(),
        src_lang: DEFAULT_SRC_LANG.to_owned(),
        pack_name: PACK.to_owned(),
        authors: vec!["Alice".to_owned(), "Bob".to_owned()],
        glossary,
        translations,
    }
}

/// Resolve where [`PackTranslation::save`] will drop the JSON for a given
/// base directory, so tests can read the raw on-disk bytes back.
///
/// v1 files live under `{base}/{game}/{pack}/{src_lang}-{lang}.json`; v0 files keep the
/// legacy `{base}/{game}/{pack}/{lang}.json` layout because the old format had no source
/// language in the filename.
fn translation_path(base: &Path, pack: &str, version: u32, src_lang: &str, lang: &str) -> PathBuf {
    let filename = if version == 0 {
        format!("{lang}.json")
    } else {
        format!("{src_lang}-{lang}.json")
    };
    base.join(format!("{GAME}/{pack}/{filename}"))
}

/// Roundtripping a v1 translation through disk should preserve every field,
/// including the per-entry `aut` flag that doesn't exist in v0.
#[test]
fn roundtrip_v1() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();

    original.save(tmp.path(), GAME).unwrap();
    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();

    assert_eq!(loaded.version, CURRENT_VERSION);
    assert_eq!(loaded.language, original.language);
    assert_eq!(loaded.src_lang, original.src_lang);
    assert_eq!(loaded.pack_name, original.pack_name);
    assert_eq!(loaded.authors, original.authors);
    assert_eq!(loaded.glossary, original.glossary);
    assert_eq!(loaded.translations.len(), original.translations.len());
    for (key, tr) in &original.translations {
        let loaded_tr = loaded.translations.get(key).expect("entry missing after roundtrip");
        assert_eq!(loaded_tr.src, tr.src, "src mismatch for {key}");
        assert_eq!(loaded_tr.dst, tr.dst, "dst mismatch for {key}");
        assert_eq!(loaded_tr.retr, tr.retr, "retr mismatch for {key}");
        assert_eq!(loaded_tr.rem, tr.rem, "rem mismatch for {key}");
        assert_eq!(loaded_tr.aut, tr.aut, "aut mismatch for {key}");
    }
}

/// Roundtripping with `version = 0` should write the legacy wire shape and
/// reload it without losing translation data. v0 has no per-entry `aut`, so
/// it collapses to `false`.
#[test]
fn roundtrip_v0() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();
    original.version = 0;

    original.save(tmp.path(), GAME).unwrap();
    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();

    // Version stays at 0 so the next save keeps writing the legacy shape.
    assert_eq!(loaded.version, 0);
    assert_eq!(loaded.language, original.language);
    assert_eq!(loaded.pack_name, original.pack_name);
    assert!(loaded.authors.is_empty(), "v0 has no authors");
    assert!(loaded.glossary.is_empty(), "v0 has no glossary");

    // The core translation data must survive intact, modulo the dropped `aut` flag.
    assert_eq!(loaded.translations.len(), original.translations.len());
    for (key, tr) in &original.translations {
        let loaded_tr = loaded.translations.get(key).expect("entry missing after roundtrip");
        assert_eq!(loaded_tr.src, tr.src, "src mismatch for {key}");
        assert_eq!(loaded_tr.dst, tr.dst, "dst mismatch for {key}");
        assert_eq!(loaded_tr.retr, tr.retr, "retr mismatch for {key}");
        assert_eq!(loaded_tr.rem, tr.rem, "rem mismatch for {key}");
        assert!(!loaded_tr.aut, "aut should default to false when read from v0");
    }
}

/// Saving as v0 must produce the legacy JSON shape (no `version` key, the old
/// long field names, no `aut`). Asserting on the raw JSON guards against
/// accidentally writing v1 fields into a v0 file.
#[test]
fn v0_wire_shape_is_legacy() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();
    original.version = 0;
    original.save(tmp.path(), GAME).unwrap();

    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, 0, SRC_LANG, LANG)).unwrap()).unwrap();
    assert!(json.get("version").is_none(), "v0 must not write a version field");
    assert!(json.get("src_lang").is_none(), "v0 must not write src_lang");
    assert!(json.get("authors").is_none(), "v0 must not write authors");
    assert!(json.get("glossary").is_none(), "v0 must not write glossary");

    let entry = json.pointer("/translations/greeting").expect("entry missing");
    assert_eq!(entry.get("key").and_then(Value::as_str), Some("greeting"));
    assert_eq!(entry.get("value_original").and_then(Value::as_str), Some("Hello"));
    assert_eq!(entry.get("value_translated").and_then(Value::as_str), Some("Hola"));
    assert!(entry.get("src").is_none(), "v0 must not use the v1 short field names");
    assert!(entry.get("aut").is_none(), "v0 must not carry the per-entry aut flag");
}

/// Saving as v1 must produce the current JSON shape with the short field names.
/// Counterpart to [`v0_wire_shape_is_legacy`].
#[test]
fn v1_wire_shape_is_current() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();
    original.save(tmp.path(), GAME).unwrap();

    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, original.version, &original.src_lang, LANG)).unwrap()).unwrap();
    assert_eq!(json.get("version").and_then(Value::as_u64), Some(CURRENT_VERSION as u64));
    assert_eq!(json.get("src_lang").and_then(Value::as_str), Some(DEFAULT_SRC_LANG));
    assert_eq!(json.pointer("/glossary/Empire").and_then(Value::as_str), Some("Imperio"));
    assert_eq!(json.pointer("/authors/0").and_then(Value::as_str), Some("Alice"));

    let entry = json.pointer("/translations/greeting").expect("entry missing");
    assert_eq!(entry.get("src").and_then(Value::as_str), Some("Hello"));
    assert_eq!(entry.get("dst").and_then(Value::as_str), Some("Hola"));
    assert!(entry.get("value_original").is_none(), "v1 must not use the v0 long field names");
}

/// A hand-written legacy file (no `version`, old field names, no `aut`) must
/// deserialize into the current in-memory shape with the right defaults.
/// This is what we promise existing translation hubs.
#[test]
fn load_handwritten_legacy_v0() {
    let tmp = TempDir::new().unwrap();
    let path = translation_path(tmp.path(), PACK, 0, SRC_LANG, LANG);
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    let legacy = json!({
        "language": LANG,
        "pack_name": PACK,
        "translations": {
            "greeting": {
                "key": "greeting",
                "value_original": "Hello",
                "value_translated": "Hola",
                "needs_retranslation": false,
                "removed": false,
            },
            "obsolete": {
                "key": "obsolete",
                "value_original": "Gone",
                "value_translated": "Eliminado",
                "needs_retranslation": false,
                "removed": true,
            },
        },
    });
    fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();
    assert_eq!(loaded.version, 0, "missing version field must be treated as v0");
    assert_eq!(loaded.src_lang, default_src_lang());

    let greeting = loaded.translations.get("greeting").expect("greeting missing");
    assert_eq!(greeting.src, "Hello");
    assert_eq!(greeting.dst, "Hola");
    assert!(!greeting.retr);
    assert!(!greeting.rem);
    assert!(!greeting.aut, "aut must default to false on legacy entries");

    let obsolete = loaded.translations.get("obsolete").expect("obsolete missing");
    assert!(obsolete.rem, "the `removed` alias must populate `rem`");
}

/// Converting v0 → v1: load a legacy file, flip the version, save, reload.
/// The legacy data must survive and the new file must carry the v1 shape.
#[test]
fn convert_v0_to_v1() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();
    original.version = 0;
    original.save(tmp.path(), GAME).unwrap();

    let mut upgraded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();
    assert_eq!(upgraded.version, 0);
    upgraded.version = CURRENT_VERSION;
    upgraded.save(tmp.path(), GAME).unwrap();

    // After saving as v1 the file moves from `{lang}.json` to `{src_lang}-{lang}.json`.
    assert!(!translation_path(tmp.path(), PACK, 0, SRC_LANG, LANG).is_file(), "the v0 file must be removed");
    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, CURRENT_VERSION, &upgraded.src_lang, LANG)).unwrap()).unwrap();
    assert_eq!(json.get("version").and_then(Value::as_u64), Some(CURRENT_VERSION as u64));

    let reloaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();
    assert_eq!(reloaded.version, CURRENT_VERSION);
    assert_eq!(reloaded.translations.len(), original.translations.len());
    for (key, tr) in &original.translations {
        let r = reloaded.translations.get(key).expect("entry missing after upgrade");
        assert_eq!(r.src, tr.src);
        assert_eq!(r.dst, tr.dst);
        assert_eq!(r.retr, tr.retr);
        assert_eq!(r.rem, tr.rem);
    }
}

/// Converting v1 → v0: take a v1 in-memory translation, downgrade it, save,
/// reload. The `aut` flag is expected to be dropped; the core translation
/// rows must round-trip.
#[test]
fn convert_v1_to_v0() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();

    // Mark one entry as auto so we can confirm the flag is dropped on downgrade.
    original.translations.get_mut("greeting").unwrap().aut = true;
    original.save(tmp.path(), GAME).unwrap();

    let mut downgraded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();
    assert_eq!(downgraded.version, CURRENT_VERSION);
    downgraded.version = 0;
    downgraded.save(tmp.path(), GAME).unwrap();

    // Wire-level: the on-disk file must look like v0 — no `version`, no `aut`/`src_lang`, old
    // field names. Downgrading also moves the file back to the legacy `{lang}.json` location.
    assert!(!translation_path(tmp.path(), PACK, CURRENT_VERSION, SRC_LANG, LANG).is_file(), "the v1 file must be removed");
    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, 0, SRC_LANG, LANG)).unwrap()).unwrap();
    assert!(json.get("version").is_none());
    assert!(json.get("src_lang").is_none());
    let entry = json.pointer("/translations/greeting").expect("entry missing");
    assert!(entry.get("aut").is_none(), "aut must be dropped when downgrading to v0");
    assert_eq!(entry.get("value_original").and_then(Value::as_str), Some("Hello"));

    // Reload-level: data still round-trips, just without `aut`.
    let reloaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();
    assert_eq!(reloaded.version, 0);
    for (key, tr) in &original.translations {
        let r = reloaded.translations.get(key).expect("entry missing after downgrade");
        assert_eq!(r.src, tr.src);
        assert_eq!(r.dst, tr.dst);
        assert_eq!(r.retr, tr.retr);
        assert_eq!(r.rem, tr.rem);
        assert!(!r.aut, "aut must default to false on v0 reload");
    }
}

/// Saving a v1 translation from a non-EN source must not delete the legacy
/// `{lang}.json`, because that file is the EN-sourced translation, not a stale copy.
#[test]
fn save_non_en_source_keeps_en_v0_file() {
    let tmp = TempDir::new().unwrap();
    let mut en_v0 = sample_v1();
    en_v0.version = 0;
    en_v0.save(tmp.path(), GAME).unwrap();

    let mut de_v1 = sample_v1();
    de_v1.src_lang = "DE".to_owned();
    de_v1.save(tmp.path(), GAME).unwrap();

    assert!(translation_path(tmp.path(), PACK, 0, SRC_LANG, LANG).is_file());
    assert!(translation_path(tmp.path(), PACK, CURRENT_VERSION, "DE", LANG).is_file());
}

/// A v0 file is only a valid fallback for EN sources, since v0 predates non-EN sources.
#[test]
fn load_v0_only_for_en_source() {
    let tmp = TempDir::new().unwrap();
    let mut en_v0 = sample_v1();
    en_v0.version = 0;
    en_v0.save(tmp.path(), GAME).unwrap();

    assert!(PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).is_ok());
    assert!(PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, "DE", LANG).is_err());
}

/// [`PackTranslation::Default`] must produce a fresh-but-current translation.
/// This is what [`PackTranslation::new`] uses when no prior translation file exists.
#[test]
fn default_is_current_version() {
    let pt = PackTranslation::default();
    assert_eq!(pt.version, CURRENT_VERSION);
    assert_eq!(pt.src_lang, DEFAULT_SRC_LANG);
    assert!(pt.authors.is_empty());
    assert!(pt.glossary.is_empty());
    assert!(pt.language.is_empty());
    assert!(pt.pack_name.is_empty());
    assert!(pt.translations.is_empty());
}

/// `load` rewrites whitespace and `||` markers in `dst` so the loaded value
/// shows them in escaped form. Order matters: the `\n||\n` collapse must run
/// first, otherwise the surrounding `\n`s would already be escaped into
/// `\\n` and the marker wouldn't match.
#[test]
fn load_substitutes_special_chars_in_dst() {
    let tmp = TempDir::new().unwrap();
    let path = translation_path(tmp.path(), PACK, CURRENT_VERSION, SRC_LANG, LANG);
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // Hand-built so we control the on-disk bytes exactly. Each entry exercises
    // one substitution rule the loader is supposed to apply.
    let on_disk = json!({
        "version": CURRENT_VERSION,
        "language": LANG,
        "src_lang": DEFAULT_SRC_LANG,
        "pack_name": PACK,
        "translations": {
            "newline": {
                "src": "line\nbreak",
                "dst": "linea\nrota",
                "retr": false, "rem": false, "aut": false,
            },
            "carriage": {
                "src": "carriage\rreturn",
                "dst": "retorno\rde\rcarro",
                "retr": false, "rem": false, "aut": false,
            },
            "tab": {
                "src": "left\tright",
                "dst": "izquierda\tderecha",
                "retr": false, "rem": false, "aut": false,
            },
            "sandwich": {
                "src": "before\n||\nafter",
                "dst": "antes\n||\ndespues",
                "retr": false, "rem": false, "aut": false,
            },
        },
    });
    fs::write(&path, serde_json::to_vec_pretty(&on_disk).unwrap()).unwrap();

    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();

    // `\n` (one char) → `\\n` (two chars: backslash, backslash, n). The
    // replacement string in the source is `"\\\\n"`, which is two literal
    // backslashes followed by `n`. `src` is untouched.
    let nl = loaded.translations.get("newline").unwrap();
    assert_eq!(nl.src, "line\nbreak", "src must not be rewritten");
    assert_eq!(nl.dst, r"linea\\nrota");

    // `\r` → `\\r`.
    let cr = loaded.translations.get("carriage").unwrap();
    assert_eq!(cr.dst, r"retorno\\rde\\rcarro");

    // `\t` → `\\t`.
    let tb = loaded.translations.get("tab").unwrap();
    assert_eq!(tb.dst, r"izquierda\\tderecha");

    // `\n||\n` collapses to `||` *before* the lone-`\n` rule runs, so the
    // surrounding newlines disappear entirely instead of becoming `\\n||\\n`.
    let sw = loaded.translations.get("sandwich").unwrap();
    assert_eq!(sw.dst, "antes||despues");
}

/// The reverse direction: `save` writes `dst` verbatim. No substitutions
/// happen on the way out, so a value containing `\n`/`\r`/`\t`/`\n||\n` lands
/// on disk as those literal characters (JSON-escaped by serde, as expected).
/// Combined with [`load_substitutes_special_chars_in_dst`] this proves the
/// round-trip is intentionally lossy: load mutates, save does not.
#[test]
fn save_writes_dst_verbatim() {
    let tmp = TempDir::new().unwrap();
    let mut pt = PackTranslation {
        version: CURRENT_VERSION,
        language: LANG.to_owned(),
        src_lang: DEFAULT_SRC_LANG.to_owned(),
        pack_name: PACK.to_owned(),
        authors: Vec::new(),
        glossary: BTreeMap::new(),
        translations: HashMap::new(),
    };
    pt.translations.insert("newline".to_owned(), Translation {
        src: "line\nbreak".to_owned(),
        dst: "linea\nrota".to_owned(),
        retr: false, rem: false, aut: false,
    });
    pt.translations.insert("sandwich".to_owned(), Translation {
        src: "x".to_owned(),
        dst: "antes\n||\ndespues".to_owned(),
        retr: false, rem: false, aut: false,
    });

    pt.save(tmp.path(), GAME).unwrap();

    // Parse the file ourselves so we see exactly what serde_json wrote, with
    // no further processing on top.
    let raw: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, pt.version, &pt.src_lang, LANG)).unwrap()).unwrap();

    // `\n` survives as a real newline character in the parsed JSON — i.e.
    // save didn't pre-escape it into `\\n`, which is what `load` would later do.
    assert_eq!(
        raw.pointer("/translations/newline/dst").and_then(Value::as_str),
        Some("linea\nrota"),
    );
    assert_eq!(
        raw.pointer("/translations/sandwich/dst").and_then(Value::as_str),
        Some("antes\n||\ndespues"),
    );

    // Sanity-check the asymmetry end-to-end: feeding that same file back
    // through `load` reapplies the substitutions, so the in-memory value
    // diverges from what we just saved.
    let reloaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, SRC_LANG, LANG).unwrap();
    assert_eq!(reloaded.translations.get("newline").unwrap().dst, r"linea\\nrota");
    assert_eq!(reloaded.translations.get("sandwich").unwrap().dst, "antes||despues");
}

/// English keeps the Translation Hub's file name; other languages use their own code.
#[test]
fn vanilla_loc_file_name_per_language() {
    assert_eq!(PackTranslation::vanilla_loc_file_name("EN"), VANILLA_LOC_NAME_EN);
    assert_eq!(PackTranslation::vanilla_loc_file_name("SP"), "vanilla_sp.tsv");
}

/// Only packs for the requested language are picked, including split ones like
/// `local_sp_2.pack`, without matching other codes that share a prefix.
#[test]
fn locale_pack_paths_matches_language_only() {
    let tmp = TempDir::new().unwrap();
    for name in ["local_sp.pack", "local_sp_2.pack", "LOCAL_SP_3.PACK", "local_spx.pack", "local_en.pack", "local_sp.txt", "data.pack"] {
        fs::write(tmp.path().join(name), b"").unwrap();
    }

    let names = PackTranslation::locale_pack_paths(tmp.path(), "SP").unwrap()
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["LOCAL_SP_3.PACK", "local_sp.pack", "local_sp_2.pack"]);
}

/// Linux ports keep locale packs in `localisation/{lang}/`, and other languages' folders must be ignored.
#[test]
fn locale_pack_paths_finds_linux_localisation_folder() {
    let tmp = TempDir::new().unwrap();
    for (folder, name) in [("sp", "local_sp.pack"), ("sp", "local_sp_gc.pack"), ("en", "local_en.pack")] {
        let folder = tmp.path().join("localisation").join(folder);
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join(name), b"").unwrap();
    }

    let paths = PackTranslation::locale_pack_paths(tmp.path(), "SP").unwrap();

    assert_eq!(paths, vec![
        tmp.path().join("localisation/sp/local_sp.pack"),
        tmp.path().join("localisation/sp/local_sp_gc.pack"),
    ]);
}

/// The glossary survives a round-trip through its UI table.
#[test]
fn glossary_table_roundtrip() {
    let original = sample_v1();
    let table = original.glossary_to_table().unwrap();

    let mut restored = sample_v1();
    restored.glossary.clear();
    restored.glossary_from_table(&table).unwrap();

    assert_eq!(restored.glossary, original.glossary);
}

/// Reading the glossary from its UI table trims the source terms, skips rows without one
/// (like a blank row the user added and never filled), and replaces the previous glossary.
#[test]
fn glossary_from_table_skips_empty_sources() {
    let mut table = TableInMemory::new(&PackTranslation::glossary_definition(), None, "");
    table.set_data(&[
        vec![DecodedData::StringU8("  Empire ".to_owned()), DecodedData::StringU8("Imperio".to_owned())],
        vec![DecodedData::StringU8("   ".to_owned()), DecodedData::StringU8("Nada".to_owned())],
        vec![DecodedData::StringU8(String::new()), DecodedData::StringU8(String::new())],
        vec![DecodedData::StringU8("Faction".to_owned()), DecodedData::StringU8(String::new())],
    ]).unwrap();

    let mut pt = sample_v1();
    pt.glossary.insert("Stale".to_owned(), "Viejo".to_owned());
    pt.glossary_from_table(&table).unwrap();

    let mut expected = BTreeMap::new();
    expected.insert("Empire".to_owned(), "Imperio".to_owned());
    expected.insert("Faction".to_owned(), String::new());
    assert_eq!(pt.glossary, expected);
}

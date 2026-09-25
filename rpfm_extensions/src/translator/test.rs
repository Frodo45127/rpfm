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

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use tempfile::TempDir;

use super::*;

const GAME: &str = "warhammer_3";
const PACK: &str = "test_pack.pack";
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

    PackTranslation {
        version: CURRENT_VERSION,
        language: LANG.to_owned(),
        pack_name: PACK.to_owned(),
        translations,
    }
}

/// Resolve where [`PackTranslation::save`] will drop the JSON for a given
/// base directory, so tests can read the raw on-disk bytes back.
fn translation_path(base: &Path, pack: &str, lang: &str) -> PathBuf {
    base.join(format!("{GAME}/{pack}/{lang}.json"))
}

/// Roundtripping a v1 translation through disk should preserve every field,
/// including the per-entry `aut` flag that doesn't exist in v0.
#[test]
fn roundtrip_v1() {
    let tmp = TempDir::new().unwrap();
    let mut original = sample_v1();

    original.save(tmp.path(), GAME).unwrap();
    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();

    assert_eq!(loaded.version, CURRENT_VERSION);
    assert_eq!(loaded.language, original.language);
    assert_eq!(loaded.pack_name, original.pack_name);
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
    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();

    // Version stays at 0 so the next save keeps writing the legacy shape.
    assert_eq!(loaded.version, 0);
    assert_eq!(loaded.language, original.language);
    assert_eq!(loaded.pack_name, original.pack_name);

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

    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, LANG)).unwrap()).unwrap();
    assert!(json.get("version").is_none(), "v0 must not write a version field");

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

    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, LANG)).unwrap()).unwrap();
    assert_eq!(json.get("version").and_then(Value::as_u64), Some(CURRENT_VERSION as u64));

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
    let path = translation_path(tmp.path(), PACK, LANG);
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

    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();
    assert_eq!(loaded.version, 0, "missing version field must be treated as v0");

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

    let mut upgraded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();
    assert_eq!(upgraded.version, 0);
    upgraded.version = CURRENT_VERSION;
    upgraded.save(tmp.path(), GAME).unwrap();

    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, LANG)).unwrap()).unwrap();
    assert_eq!(json.get("version").and_then(Value::as_u64), Some(CURRENT_VERSION as u64));

    let reloaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();
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

    let mut downgraded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();
    assert_eq!(downgraded.version, CURRENT_VERSION);
    downgraded.version = 0;
    downgraded.save(tmp.path(), GAME).unwrap();

    // Wire-level: the on-disk file must look like v0 — no `version`, no `aut`, old field names.
    let json: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, LANG)).unwrap()).unwrap();
    assert!(json.get("version").is_none());
    let entry = json.pointer("/translations/greeting").expect("entry missing");
    assert!(entry.get("aut").is_none(), "aut must be dropped when downgrading to v0");
    assert_eq!(entry.get("value_original").and_then(Value::as_str), Some("Hello"));

    // Reload-level: data still round-trips, just without `aut`.
    let reloaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();
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

/// [`PackTranslation::Default`] must produce a fresh-but-current translation.
/// This is what [`PackTranslation::new`] uses when no prior translation file exists.
#[test]
fn default_is_current_version() {
    let pt = PackTranslation::default();
    assert_eq!(pt.version, CURRENT_VERSION);
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
    let path = translation_path(tmp.path(), PACK, LANG);
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    // Hand-built so we control the on-disk bytes exactly. Each entry exercises
    // one substitution rule the loader is supposed to apply.
    let on_disk = json!({
        "version": CURRENT_VERSION,
        "language": LANG,
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

    let loaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();

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
        pack_name: PACK.to_owned(),
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
    let raw: Value = serde_json::from_slice(&fs::read(translation_path(tmp.path(), PACK, LANG)).unwrap()).unwrap();

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
    let reloaded = PackTranslation::load(&[tmp.path().to_path_buf()], PACK, GAME, LANG).unwrap();
    assert_eq!(reloaded.translations.get("newline").unwrap().dst, r"linea\\nrota");
    assert_eq!(reloaded.translations.get("sandwich").unwrap().dst, "antes||despues");
}

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Tests for the grouping of batch auto-translation requests.

use super::*;

fn texts(sizes: &[usize]) -> Vec<String> {
    sizes.iter().map(|size| "a".repeat(*size)).collect()
}

/// Backends without multi-text requests get one text per group.
#[test]
fn batch_groups_single_text_backends() {
    let sources = texts(&[10, 20, 30]);

    for method in [BatchTranslateMethod::Ai, BatchTranslateMethod::Google] {
        assert_eq!(ToolTranslator::batch_groups(&sources, method), vec![0..1, 1..2, 2..3]);
    }
}

/// DeepL groups are cut at the text count limit.
#[test]
fn batch_groups_deepl_text_limit() {
    let sources = texts(&[1; DEEPL_GROUP_MAX_TEXTS * 2 + 1]);

    let groups = ToolTranslator::batch_groups(&sources, BatchTranslateMethod::Deepl);

    assert_eq!(groups, vec![0..DEEPL_GROUP_MAX_TEXTS, DEEPL_GROUP_MAX_TEXTS..DEEPL_GROUP_MAX_TEXTS * 2, DEEPL_GROUP_MAX_TEXTS * 2..sources.len()]);
}

/// DeepL groups are cut before going over the byte limit, and a text over the limit goes alone.
#[test]
fn batch_groups_deepl_byte_limit() {
    let half = DEEPL_GROUP_MAX_BYTES / 2;
    let sources = texts(&[half, half, 1, DEEPL_GROUP_MAX_BYTES + 1, 1]);

    let groups = ToolTranslator::batch_groups(&sources, BatchTranslateMethod::Deepl);

    assert_eq!(groups, vec![0..2, 2..3, 3..4, 4..5]);
}

/// No texts means no requests.
#[test]
fn batch_groups_empty() {
    assert!(ToolTranslator::batch_groups(&[], BatchTranslateMethod::Deepl).is_empty());
}

/// DeepL glossaries take base language codes, so regional variants of the translation languages are collapsed.
#[test]
fn deepl_glossary_language_collapses_variants() {
    for (language, expected) in [
        (Lang::EN_GB, GlossaryLanguage::En),
        (Lang::PT_BR, GlossaryLanguage::Pt),
        (Lang::ZH_HANS, GlossaryLanguage::Zh),
        (Lang::ZH_HANT, GlossaryLanguage::Zh),
        (Lang::DE, GlossaryLanguage::De),
        (Lang::TR, GlossaryLanguage::Tr),
    ] {
        assert_eq!(ToolTranslator::deepl_glossary_language(&language).unwrap(), expected);
    }
}

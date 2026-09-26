//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module containing tests for decoding/encoding `GroupFormations` files.

use std::io::{BufReader, BufWriter, Write};
use std::fs::File;

use crate::binary::ReadBytes;
use crate::error::RLibError;
use crate::files::*;

use super::*;
use super::validation::ValidationIssue;
use super::versions::v2;

#[test]
fn test_encode_group_formation_pharaoh() {

    let path_1 = "../test_files/test_decode_group_formations_pharaoh.bin";
    let path_2 = "../test_files/test_encode_group_formations_pharaoh.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_PHARAOH).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_warhammer_3() {

    let path_1 = "../test_files/test_decode_group_formations_wh3.bin";
    let path_2 = "../test_files/test_encode_group_formations_wh3.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_WARHAMMER_3).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_troy() {

    let path_1 = "../test_files/test_decode_group_formations_troy.bin";
    let path_2 = "../test_files/test_encode_group_formations_troy.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_TROY).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_three_kingdoms() {

    let path_1 = "../test_files/test_decode_group_formations_3k.bin";
    let path_2 = "../test_files/test_encode_group_formations_3k.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_THREE_KINGDOMS).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_thrones() {

    let path_1 = "../test_files/test_decode_group_formations_tob.bin";
    let path_2 = "../test_files/test_encode_group_formations_tob.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_THRONES_OF_BRITANNIA).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_attila() {

    let path_1 = "../test_files/test_decode_group_formations_att.bin";
    let path_2 = "../test_files/test_encode_group_formations_att.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_ATTILA).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_rome_2() {

    let path_1 = "../test_files/test_decode_group_formations_rom2.bin";
    let path_2 = "../test_files/test_encode_group_formations_rom2.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_ROME_2).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

#[test]
fn test_encode_group_formation_shogun_2() {

    let path_1 = "../test_files/test_decode_group_formations_sho2.bin";
    let path_2 = "../test_files/test_encode_group_formations_sho2.bin";
    let mut reader = BufReader::new(File::open(path_1).unwrap());

    let games = SupportedGames::default();
    let game = games.game(KEY_SHOGUN_2).unwrap();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(game);

    let data_len = reader.len().unwrap();
    let before = reader.read_slice(data_len as usize, true).unwrap();
    let mut data = GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap();

    let extra_data = EncodeableExtraData::new_from_game_info(game);

    let mut after = vec![];
    data.encode(&mut after, &Some(extra_data)).unwrap();

    let mut writer = BufWriter::new(File::create(path_2).unwrap());
    writer.write_all(&after).unwrap();

    assert_eq!(before, after);
}

/// Decodes one of the test files with the provided game.
fn decode_test_file(file_name: &str, game_key: &str) -> GroupFormations {
    let mut reader = BufReader::new(File::open(format!("../test_files/{file_name}")).unwrap());
    let games = SupportedGames::default();

    let mut extra_data = DecodeableExtraData::default();
    extra_data.game_info = Some(games.game(game_key).unwrap());

    GroupFormations::decode(&mut reader, &Some(extra_data)).unwrap()
}

/// Builds a relative container pointing to the provided block, with one entity preference.
fn relative_block(block_id: u32, relative_block_id: u32) -> GroupFormationBlock {
    let mut container = ContainerRelative::default();
    container.set_relative_block_id(relative_block_id);
    container.set_maximum_entity_threshold(-1);
    container.entity_preferences_mut().push(EntityPreference::new(GroupFormationsFormat::Warhammer3));

    let mut block = GroupFormationBlock::default();
    block.set_block_id(block_id);
    block.set_block(Block::ContainerRelative(container));
    block
}

/// Builds an absolute container with one entity preference.
fn absolute_block(block_id: u32) -> GroupFormationBlock {
    let mut container = ContainerAbsolute::default();
    container.set_maximum_entity_threshold(-1);
    container.entity_preferences_mut().push(EntityPreference::new(GroupFormationsFormat::Warhammer3));

    let mut block = GroupFormationBlock::default();
    block.set_block_id(block_id);
    block.set_block(Block::ContainerAbsolute(container));
    block
}

/// Builds a span over the provided blocks.
fn span_block(block_id: u32, spanned_block_ids: Vec<u32>) -> GroupFormationBlock {
    let mut span = Spanning::default();
    span.set_spanned_block_ids(spanned_block_ids);

    let mut block = GroupFormationBlock::default();
    block.set_block_id(block_id);
    block.set_block(Block::Spanning(span));
    block
}

#[test]
fn test_vanilla_group_formations_have_no_validation_errors() {
    let files = [
        ("test_decode_group_formations_sho2.bin", KEY_SHOGUN_2),
        ("test_decode_group_formations_rom2.bin", KEY_ROME_2),
        ("test_decode_group_formations_att.bin", KEY_ATTILA),
        ("test_decode_group_formations_tob.bin", KEY_THRONES_OF_BRITANNIA),
        ("test_decode_group_formations_troy.bin", KEY_TROY),
        ("test_decode_group_formations_pharaoh.bin", KEY_PHARAOH),
        ("test_decode_group_formations_3k.bin", KEY_THREE_KINGDOMS),
        ("test_decode_group_formations_wh3.bin", KEY_WARHAMMER_3),
    ];

    for (file_name, game_key) in files {
        let data = decode_test_file(file_name, game_key);
        let errors = data.validate().into_iter().filter(|(_, issue)| issue.is_error()).collect::<Vec<_>>();
        assert!(errors.is_empty(), "{file_name}: {errors:?}");
    }
}

#[test]
fn test_attila_ai_purpose_flags_match_assembly_kit() {
    let data = decode_test_file("test_decode_group_formations_att.bin", KEY_ATTILA);
    let purpose = |name: &str| data.formations().iter().find(|formation| formation.name() == name).unwrap().ai_purpose().clone();

    assert_eq!(purpose("settlement_area_defend_wide_default"), AIPurpose::V2(v2::AIPurposeFlags::SETTLEMENT_AREA_DEFEND_WIDE));
    assert_eq!(purpose("settlement_area_attack_wide_default"), AIPurpose::V2(v2::AIPurposeFlags::SETTLEMENT_AREA_ATTACK_WIDE));
}

#[test]
fn test_new_formation_round_trips_in_every_format() {
    let games = SupportedGames::default();
    for game_key in [KEY_SHOGUN_2, KEY_ROME_2, KEY_TROY, KEY_WARHAMMER_3] {
        let game = games.game(game_key).unwrap();
        let format = GroupFormationsFormat::from_game(game).unwrap();

        let mut container = ContainerAbsolute::default();
        container.entity_preferences_mut().push(EntityPreference::new(format));

        let mut block = GroupFormationBlock::default();
        block.set_block(Block::ContainerAbsolute(container));

        let mut formation = GroupFormation::new("test", format);
        formation.group_formation_blocks_mut().push(block);

        let mut data = GroupFormations::default();
        data.formations_mut().push(formation);

        let mut encoded = vec![];
        data.encode(&mut encoded, &Some(EncodeableExtraData::new_from_game_info(game))).unwrap();

        let mut extra_data = DecodeableExtraData::default();
        extra_data.game_info = Some(game);
        let decoded = GroupFormations::decode(&mut std::io::Cursor::new(encoded), &Some(extra_data)).unwrap();

        assert_eq!(data, decoded, "{game_key}");
    }
}

#[test]
fn test_encode_rejects_mismatched_versions() {
    let games = SupportedGames::default();
    let game = games.game(KEY_WARHAMMER_3).unwrap();

    let mut data = GroupFormations::default();
    data.formations_mut().push(GroupFormation::new("test", GroupFormationsFormat::Shogun2));

    let mut encoded = vec![];
    let result = data.encode(&mut encoded, &Some(EncodeableExtraData::new_from_game_info(game)));
    assert!(matches!(result, Err(RLibError::EncodingGroupFormationsMismatchedVersion(_))));
}

#[test]
fn test_validate_valid_formation() {
    let mut formation = GroupFormation::new("test", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(vec![absolute_block(0), relative_block(1, 0), span_block(2, vec![0, 1]), relative_block(3, 2)]);

    assert!(formation.validate().is_empty());
}

#[test]
fn test_validate_reports_broken_references() {
    let mut formation = GroupFormation::new("test", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(vec![relative_block(0, 1), relative_block(1, 0), relative_block(2, 9), relative_block(2, 0)]);

    let issues = formation.validate();
    assert!(issues.contains(&ValidationIssue::NoAbsoluteBlock));
    assert!(issues.contains(&ValidationIssue::DuplicateBlockId(2)));
    assert!(issues.contains(&ValidationIssue::MissingReference { block_id: 2, referenced_id: 9 }));
    assert!(issues.contains(&ValidationIssue::ForwardReference { block_id: 0, referenced_id: 1 }));
    assert!(issues.contains(&ValidationIssue::ReferenceCycle(vec![0, 1])));
}

#[test]
fn test_validate_reports_span_cycles() {

    // The span depends on its member, and the member is placed relative to the span.
    let mut formation = GroupFormation::new("test", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(vec![absolute_block(0), relative_block(1, 2), span_block(2, vec![1]), span_block(3, vec![])]);

    let issues = formation.validate();
    assert!(issues.contains(&ValidationIssue::ReferenceCycle(vec![1, 2])));
    assert!(issues.contains(&ValidationIssue::EmptySpan(3)));
}

#[test]
fn test_validate_reports_container_warnings() {
    let mut invalid = absolute_block(0);
    if let Block::ContainerAbsolute(container) = invalid.block_mut() {
        container.set_minimum_entity_threshold(5);
        container.set_maximum_entity_threshold(2);
        container.entity_preferences_mut().clear();
    }

    let mut formation = GroupFormation::new("test", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(vec![invalid]);

    let issues = formation.validate();
    assert_eq!(issues, vec![ValidationIssue::InvalidThresholds(0), ValidationIssue::NoEntityPreferences(0)]);
    assert!(issues.iter().all(|issue| !issue.is_error()));
}

#[test]
fn test_validate_reports_duplicate_formation_names() {
    let mut data = GroupFormations::default();
    let mut formation = GroupFormation::new("test", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(vec![absolute_block(0)]);
    data.set_formations(vec![formation.clone(), formation]);

    assert_eq!(data.validate(), vec![(1, ValidationIssue::DuplicateFormationName)]);
}

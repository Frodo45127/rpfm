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
use super::layout::{self, LayoutParams, LayoutRect};
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

/// Builds a relative container pointing to the provided block, at the provided offset.
fn offset_block(block_id: u32, relative_block_id: u32, offset_x: f32, offset_y: f32) -> GroupFormationBlock {
    let mut block = relative_block(block_id, relative_block_id);
    if let Block::ContainerRelative(container) = block.block_mut() {
        container.set_position_x(offset_x);
        container.set_position_y(offset_y);
    }
    block
}

/// Builds a formation with the provided blocks.
fn formation_with(blocks: Vec<GroupFormationBlock>) -> GroupFormation {
    let mut formation = GroupFormation::new("test", GroupFormationsFormat::Warhammer3);
    formation.set_group_formation_blocks(blocks);
    formation
}

/// Returns the simulated center of a block.
fn center_of(formation: &GroupFormation, block_id: u32) -> (f32, f32) {
    let rect = formation.layout(&LayoutParams::default())[&block_id];
    (rect.center_x(), rect.center_y())
}

#[test]
fn test_layout_places_relative_blocks_from_parent_edges() {

    // With default params, a line of 2 units without spacing is 40m wide and 8m deep.
    let formation = formation_with(vec![
        absolute_block(0),
        offset_block(1, 0, 5.0, 0.0),
        offset_block(2, 0, 0.0, -5.0),
        span_block(3, vec![0, 1]),
        offset_block(4, 3, 0.0, 10.0),
    ]);

    let rects = formation.layout(&LayoutParams::default());
    assert_eq!(rects[&0], LayoutRect::new(0.0, 0.0, 40.0, 8.0));
    assert_eq!(rects[&1], LayoutRect::new(45.0, 0.0, 40.0, 8.0));
    assert_eq!(rects[&2], LayoutRect::new(0.0, -13.0, 40.0, 8.0));
    assert_eq!(rects[&3], LayoutRect::new(22.5, 0.0, 85.0, 8.0));
    assert_eq!(rects[&4], LayoutRect::new(22.5, 18.0, 40.0, 8.0));
}

#[test]
fn test_layout_uses_unit_counts_thresholds_and_arrangement() {
    let mut limited = absolute_block(0);
    let mut column = offset_block(1, 0, 0.0, -5.0);
    if let Block::ContainerAbsolute(container) = limited.block_mut() {
        container.set_maximum_entity_threshold(3);
    }
    if let Block::ContainerRelative(container) = column.block_mut() {
        container.set_entity_arrangement(EntityArrangement::Column);
    }

    let formation = formation_with(vec![limited, column]);
    let mut params = LayoutParams::default();
    params.unit_counts_mut().insert(0, 10);

    let rects = formation.layout(&params);
    assert_eq!(rects[&0].width(), 60.0);
    assert_eq!((rects[&1].width(), rects[&1].height()), (20.0, 16.0));
}

#[test]
fn test_layout_leaves_out_broken_blocks() {
    let formation = formation_with(vec![
        absolute_block(0),
        relative_block(1, 2),
        relative_block(2, 1),
        relative_block(3, 9),
        span_block(4, vec![0, 3]),
    ]);

    let rects = formation.layout(&LayoutParams::default());
    assert_eq!(rects.keys().copied().collect::<Vec<_>>(), vec![0]);
}

#[test]
fn test_offset_for_center_inverts_place_relative() {
    let parent = LayoutRect::new(3.0, -2.0, 42.0, 8.0);
    for (offset_x, offset_y) in [(5.0, 0.0), (-5.0, 0.0), (0.0, -20.0), (0.0, 10.0), (-30.0, 15.0)] {
        let rect = layout::place_relative(&parent, 20.0, 8.0, offset_x, offset_y);
        assert_eq!(layout::offset_for_center(&parent, 20.0, 8.0, rect.center_x(), rect.center_y()), (offset_x, offset_y));
    }
}

#[test]
fn test_sort_blocks_keeps_vanilla_files_unchanged() {
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
        let before = decode_test_file(file_name, game_key);
        let mut after = before.clone();
        after.formations_mut().iter_mut().for_each(GroupFormation::sort_blocks);

        assert_eq!(before, after, "{file_name}");
    }
}

#[test]
fn test_sort_blocks_orders_blocks_after_their_dependencies() {
    let mut formation = formation_with(vec![relative_block(5, 7), span_block(6, vec![5]), absolute_block(7), relative_block(8, 9), relative_block(9, 8)]);

    formation.sort_blocks();
    assert_eq!(formation.group_formation_blocks(), &vec![absolute_block(7), relative_block(5, 7), span_block(6, vec![5]), relative_block(8, 9), relative_block(9, 8)]);
}

#[test]
fn test_add_container_and_span() {
    let mut formation = formation_with(vec![absolute_block(0)]);

    assert_eq!(formation.add_container(Some(0), (5.0, 0.0), GroupFormationsFormat::Warhammer3).unwrap(), 1);
    assert_eq!(formation.add_container(None, (100.0, 0.0), GroupFormationsFormat::Warhammer3).unwrap(), 2);
    assert_eq!(formation.add_span(&[0, 1]).unwrap(), 3);
    assert!(matches!(formation.add_container(Some(9), (0.0, 0.0), GroupFormationsFormat::Warhammer3), Err(RLibError::GroupFormationsBlockNotFound(9))));
    assert!(formation.validate().is_empty());
    assert_eq!(center_of(&formation, 1), (46.0, 0.0));
}

#[test]
fn test_reparent_keeps_position_and_order() {
    let mut formation = formation_with(vec![absolute_block(0), offset_block(1, 0, 5.0, 0.0), offset_block(2, 0, -5.0, 0.0)]);
    let before = center_of(&formation, 1);

    formation.reparent(1, Some(2), &LayoutParams::default()).unwrap();
    assert_eq!(formation.group_formation_blocks().iter().map(|block| *block.block_id()).collect::<Vec<_>>(), vec![0, 2, 1]);
    assert_eq!(center_of(&formation, 1), before);
    assert!(formation.validate().is_empty());

    formation.reparent(1, None, &LayoutParams::default()).unwrap();
    assert!(matches!(formation.group_formation_blocks()[2].block(), Block::ContainerAbsolute(_)));
    assert_eq!(center_of(&formation, 1), before);
}

#[test]
fn test_reparent_rejects_cycles_and_spans() {
    let mut formation = formation_with(vec![absolute_block(0), relative_block(1, 0), span_block(2, vec![1])]);
    let params = LayoutParams::default();

    assert!(matches!(formation.reparent(0, Some(1), &params), Err(RLibError::GroupFormationsCyclicReference(0, 1))));
    assert!(matches!(formation.reparent(1, Some(2), &params), Err(RLibError::GroupFormationsCyclicReference(1, 2))));
    assert!(matches!(formation.reparent(2, Some(0), &params), Err(RLibError::GroupFormationsBlockIsNotAContainer(2))));
    assert!(matches!(formation.reparent(1, Some(1), &params), Err(RLibError::GroupFormationsCyclicReference(1, 1))));
}

#[test]
fn test_move_block_snaps_offsets() {
    let mut formation = formation_with(vec![absolute_block(0), offset_block(1, 0, 5.0, 0.0)]);

    formation.move_block(1, (50.0, -1.0), 5.0, &LayoutParams::default()).unwrap();
    assert_eq!(formation.group_formation_blocks()[1], offset_block(1, 0, 10.0, 0.0));

    formation.move_block(0, (2.0, 4.0), 5.0, &LayoutParams::default()).unwrap();
    assert_eq!(center_of(&formation, 0), (0.0, 5.0));
}

#[test]
fn test_delete_blocks_reattaches_children() {
    let params = LayoutParams::default();

    // Deleting a relative container re-attaches its children to its parent.
    let mut formation = formation_with(vec![absolute_block(0), offset_block(1, 0, 5.0, 0.0), offset_block(2, 1, 5.0, 0.0)]);
    let before = center_of(&formation, 2);
    formation.delete_blocks(&[1], &params).unwrap();
    assert_eq!(formation.group_formation_blocks().len(), 2);
    assert!(matches!(formation.group_formation_blocks()[1].block(), Block::ContainerRelative(container) if *container.relative_block_id() == 0));
    assert_eq!(center_of(&formation, 2), before);

    // Deleting the root turns its children into absolute containers.
    let mut formation = formation_with(vec![absolute_block(0), offset_block(1, 0, 5.0, 0.0)]);
    let before = center_of(&formation, 1);
    formation.delete_blocks(&[0], &params).unwrap();
    assert!(matches!(formation.group_formation_blocks()[0].block(), Block::ContainerAbsolute(_)));
    assert_eq!(center_of(&formation, 1), before);
}

#[test]
fn test_delete_blocks_removes_emptied_spans() {
    let mut formation = formation_with(vec![
        absolute_block(0),
        offset_block(1, 0, 5.0, 0.0),
        span_block(2, vec![0, 1]),
        span_block(3, vec![1]),
        offset_block(4, 3, 0.0, 10.0),
    ]);
    let before = center_of(&formation, 4);

    formation.delete_blocks(&[1], &LayoutParams::default()).unwrap();
    assert_eq!(formation.group_formation_blocks().len(), 3);
    assert_eq!(formation.group_formation_blocks()[1], span_block(2, vec![0]));
    assert!(matches!(formation.group_formation_blocks()[2].block(), Block::ContainerAbsolute(_)));
    assert_eq!(center_of(&formation, 4), before);
}

#[test]
fn test_delete_subtree() {
    let mut formation = formation_with(vec![
        absolute_block(0),
        relative_block(1, 0),
        relative_block(2, 1),
        span_block(3, vec![1, 2]),
        relative_block(4, 3),
        relative_block(5, 0),
    ]);

    formation.delete_subtree(1, &LayoutParams::default()).unwrap();
    assert_eq!(formation.group_formation_blocks(), &vec![absolute_block(0), relative_block(5, 0)]);
}

#[test]
fn test_set_span_members() {
    let mut formation = formation_with(vec![absolute_block(0), relative_block(1, 0), span_block(2, vec![0]), relative_block(3, 2)]);

    assert!(matches!(formation.set_span_members(2, &[3]), Err(RLibError::GroupFormationsCyclicReference(2, 3))));
    assert!(matches!(formation.set_span_members(1, &[0]), Err(RLibError::GroupFormationsBlockIsNotASpan(1))));

    formation.set_span_members(2, &[0, 1]).unwrap();
    assert_eq!(formation.group_formation_blocks()[2], span_block(2, vec![0, 1]));
}

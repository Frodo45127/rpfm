//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

use super::*;

/// A `victory_objectives.txt` with one faction, in Attila's format.
const VICTORY_SCRIPT: &str = "
victory_objectives
{
    // A comment, with { braces }.
    victory_objective
    {
        faction att_fact_a;
        campaign_type mp_vs_normal;
        mission
        {
            victory_type att_vc_1_short;
            key att_victory_mission_1_success;
            issuer CLAN_ELDERS;
            primary_objectives_and_payload
            {
                objective { type REACH_SPECIFIED_DATE; year 1177; week_of_year 0; }
                objective { type OWN_N_NAVAL_UNITS; total 20; }
                objective
                {
                    type CONSTRUCT_N_BUILDINGS_INCLUDING;
                    total 2;
                    building_level bld_a;
                    building_level bld_b;
                    faction att_fact_a;
                }
                payload { game_victory; }
            }
        }
    }
}";

#[test]
fn test_parse_script() {
    let script = parse_script("a 1 2; // comment\nb { c; d e; }").unwrap();

    assert_eq!(script.len(), 2);
    assert_eq!(script[0].key(), "a");
    assert_eq!(script[0].values(), &["1".to_owned(), "2".to_owned()]);
    assert_eq!(script[1].children().iter().map(ScriptItem::key).collect::<Vec<_>>(), vec!["c", "d"]);
    assert_eq!(script[1].child_value("d").unwrap(), "e");
}

#[test]
fn test_parse_script_unbalanced_blocks() {
    assert!(parse_script("a { b;").is_err());
    assert!(parse_script("a; }").is_err());
}

#[test]
fn test_parse_victory_objectives() {
    let objectives = parse_victory_objectives(VICTORY_SCRIPT).unwrap();

    assert_eq!(objectives.len(), 1);
    assert_eq!(objectives[0].faction(), "att_fact_a");
    assert_eq!(objectives[0].campaign_type(), CampaignType::MultiplayerVersus);
    let mission = &objectives[0].missions()[0];
    assert_eq!(mission.key(), "att_victory_mission_1_success");
    assert_eq!(mission.victory_type(), "att_vc_1_short");
    assert_eq!(mission.objectives(), &vec![
        MissionObjective::ReachSpecifiedDate { year: 1177, week: 0 },
        MissionObjective::OwnNUnits { total: 20, naval: true, units: vec![] },
        MissionObjective::ConstructNBuildingsIncluding { total: 2, buildings: vec!["bld_a".to_owned(), "bld_b".to_owned()], faction: "att_fact_a".to_owned() },
    ]);
}

#[test]
fn test_parse_victory_objectives_unsupported() {
    let unknown_objective = VICTORY_SCRIPT.replace("OWN_N_NAVAL_UNITS", "OWN_N_DRAGONS");
    let unknown_payload = VICTORY_SCRIPT.replace("game_victory", "money 100");

    assert!(parse_victory_objectives(&unknown_objective).is_err());
    assert!(parse_victory_objectives(&unknown_payload).is_err());
}

#[test]
fn test_parse_objectives_of_other_campaigns() {
    let script = |objective: &str| ScriptItem { words: vec!["objective".to_owned()], block: Some(parse_script(objective).unwrap()) };
    let parsed = [
        ("type RECLAIM_N_DIFFERENT_SETTLEMENTS_INCLUDING; total 40; region reg_a; region reg_b;", MissionObjective::ReclaimNDifferentSettlementsIncluding { total: 40, regions: vec!["reg_a".to_owned(), "reg_b".to_owned()] }),
        ("type ENSURE_FACTIONS_HAVE_NO_MILITARY_PRESENCE; total 1; faction fact_a;", MissionObjective::EnsureFactionsHaveNoMilitaryPresence { factions: vec!["fact_a".to_owned()] }),
        ("type AT_LEAST_X_PUBLIC_ORDER_IN_PROVINCES; total 30; province prov_a;", MissionObjective::AtLeastXPublicOrderInProvinces { total: 30.0, provinces: vec!["prov_a".to_owned()] }),
        ("type STAY_HORDE;", MissionObjective::StayHorde),
        ("type MAINTAIN_N_ALLIANCES; total 3;", MissionObjective::MaintainNAlliances { total: 3 }),
        ("type INCOME_AT_LEAST_X; income 10000;", MissionObjective::IncomeAtLeastX { income: 10000 }),
        ("type ELIMINATE_CHARACTER_IN_BATTLE; faction fact_a;", MissionObjective::EliminateCharacterInBattle { faction: "fact_a".to_owned() }),
        ("type CONFEDERATE_FACTIONS; total 1;", MissionObjective::ConfederateFactions { total: 1 }),
        ("type ATTAIN_FAME_LEVEL; fame_level 5;", MissionObjective::AttainFameLevel { level: 5, status: None }),
        ("type ATTAIN_FAME_LEVEL; fame_level 532; custom_completion_status text_a;", MissionObjective::AttainFameLevel { level: 532, status: Some("text_a".to_owned()) }),
        ("type SCRIPTED; script_key key_a; override_text text_a;", MissionObjective::Scripted { script_key: "key_a".to_owned(), text: Some("text_a".to_owned()) }),
        ("type CONTROL_N_PORTS_INCLUDING; total 15;", MissionObjective::ControlNPortsIncluding { total: 15, regions: vec![] }),
        ("type OWN_A_PORT_ADJOINING_SEA_REGIONS_INCLUDING; region sea_a; region sea_b;", MissionObjective::OwnAPortAdjoiningSeaRegionsIncluding { regions: vec!["sea_a".to_owned(), "sea_b".to_owned()] }),
        ("type SUBJUGATE_FACTIONS; total 1; faction fact_a;", MissionObjective::SubjugateFactions { total: 1, faction: Some("fact_a".to_owned()) }),
        ("type SUBJUGATE_FACTIONS; total 2;", MissionObjective::SubjugateFactions { total: 2, faction: None }),
        ("type OWN_N_UNITS; total 40; unit unit_a; unit unit_b;", MissionObjective::OwnNUnits { total: 40, naval: false, units: vec!["unit_a".to_owned(), "unit_b".to_owned()] }),
        ("type HOLD_ENTIRETY_OF_N_PROVINCES_INCLUDING; total 6; province prov_a; province prov_b;", MissionObjective::HoldEntiretyOfNProvincesIncluding { total: 6, provinces: vec!["prov_a".to_owned(), "prov_b".to_owned()] }),
        ("type CONSTRUCT_N_BUILDINGS_FROM; total 1; building_level bld_a; building_level bld_b; faction fact_a;", MissionObjective::ConstructNBuildingsFrom { total: 1, buildings: vec!["bld_a".to_owned(), "bld_b".to_owned()], faction: "fact_a".to_owned() }),
        ("type CONSTRUCT_N_OF_A_BUILDING; total 6; building_level bld_a; faction fact_a;", MissionObjective::ConstructNOfABuilding { total: 6, building: "bld_a".to_owned(), faction: "fact_a".to_owned() }),
        ("type HAVE_AT_LEAST_X_MONEY; total 1000000;", MissionObjective::HaveAtLeastXMoney { total: 1000000 }),
        ("type MAINTAIN_TRADE_WITH_N_FACTIONS; total 5;", MissionObjective::MaintainTradeWithNFactions { total: 5 }),
        ("type MAKE_ALLIANCE; faction fact_a;", MissionObjective::MakeAlliance { faction: "fact_a".to_owned() }),
        ("type CONTROL_N_REGIONS_INCLUDING; total 45; region reg_a;", MissionObjective::ControlNRegionsIncluding { total: 45, regions: vec!["reg_a".to_owned()] }),
    ];

    for (text, expected) in parsed {
        assert_eq!(MissionObjective::from_item(&script(text)).unwrap(), expected, "{text}");
    }
}

#[test]
fn test_written_victory_objectives_parse_back() {
    let objectives = parse_victory_objectives(VICTORY_SCRIPT).unwrap();

    let script = write_victory_objectives(&objectives);

    assert_eq!(parse_victory_objectives(&script).unwrap(), objectives);
}

#[test]
#[ignore = "needs TDD's data, see writer/victory_test.rs"]
fn test_tdd_victory_objectives_parse_back() {
    let objectives = crate::startpos::writer::victory::victory_test::tdd_objectives();

    let script = write_victory_objectives(&objectives);

    assert_eq!(parse_victory_objectives(&script).unwrap(), objectives);
}

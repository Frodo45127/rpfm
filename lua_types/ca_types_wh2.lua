
-- CLASS DECLARATION
--# assume global class CM
--# assume global class CUIM
--# assume global class CUIM_OVERRIDE
--# assume global class CA_CampaignUI
--# assume global class CA_UIC
--# assume global class CA_Component
--# assume global class CA_UIContext
--# assume global class CA_CHAR_CONTEXT
--# assume global class CA_SETTLEMENT_CONTEXT
--# assume global class CA_CQI
--# assume global class CA_CHAR
--# assume global class CA_CHAR_LIST
--# assume global class CA_MILITARY_FORCE
--# assume global class CA_MILITARY_FORCE_LIST
--# assume global class CA_REGION
--# assume global class CA_REGION_LIST
--# assume global class CA_REGION_MANAGER
--# assume global class CA_SETTLEMENT
--# assume global class CA_GARRISON_RESIDENCE
--# assume global class CA_SLOT_LIST
--# assume global class CA_SLOT
--# assume global class CA_BUILDING
--# assume global class CA_FACTION
--# assume global class CA_FACTION_LIST
--# assume global class CA_GAME
--# assume global class CA_MODEL
--# assume global class CA_WORLD
--# assume global class CA_EFFECT
--# assume global class CA_PENDING_BATTLE
--# assume global class CA_UNIT
--# assume global class CA_UNIT_LIST
--# assume global class CA_POOLED
--# assume global class CA_FACTION_RITUALS
--# assume global class CA_RITUAL
--# assume global class CA_RITUAL_LIST

--# assume global class CORE
--# assume global class _G


-- TYPES
--# type global CA_EventName = 
--# "CharacterCreated"      | "ComponentLClickUp"     | "ComponentMouseOn"    |
--# "PanelClosedCampaign"   | "PanelOpenedCampaign" |
--# "TimeTrigger"           | "UICreated"

--# type global BUTTON_STATE = 
--# "active" | "hover" | "down" | 
--# "selected" | "selected_hover" | "selected_down" |
--# "drop_down"


--# type global BATTLE_SIDE =
--# "Attacker" | "Defender" 

--# type global CA_MARKER_TYPE = 
--# "pointer" | "move_to_vfx" | "look_at_vfx" | "tutorial_marker"


-- CONTEXT
--# assume CA_UIContext.component: CA_Component
--# assume CA_UIContext.string: string
--# assume CA_SETTLEMENT_CONTEXT.garrison_residence: method() --> CA_GARRISON_RESIDENCE
--# assume CA_CHAR_CONTEXT.character: method() --> CA_CHAR

-- UIC
--# assume CA_UIC.Address: method() --> CA_Component
--# assume CA_UIC.Adopt: method(pointer: CA_Component)
--# assume CA_UIC.ChildCount: method() --> number
--# assume CA_UIC.ClearSound: method()
--# assume CA_UIC.CreateComponent: method(name: string, path: string)
--# assume CA_UIC.CurrentState: method() --> BUTTON_STATE
--# assume CA_UIC.DestroyChildren: method()
--# assume CA_UIC.Dimensions: method() --> (number, number)
--# assume CA_UIC.Find: method(arg: number | string) --> CA_Component
--# assume CA_UIC.GetTooltipText: method() --> string
--# assume CA_UIC.Id: method() --> string
--# assume CA_UIC.MoveTo: method(x: number, y: number)
--# assume CA_UIC.Parent: method() --> CA_Component
--# assume CA_UIC.Position: method() --> (number, number)
--# assume CA_UIC.Resize: method(w: number, h: number)
--# assume CA_UIC.SetInteractive: method(interactive: boolean)
--# assume CA_UIC.SetOpacity: method(opacity: number)
--# assume CA_UIC.SetState: method(state: BUTTON_STATE)
--# assume CA_UIC.SetStateText: method(text: string)
--# assume CA_UIC.SetVisible: method(visible: boolean)
--# assume CA_UIC.SetDisabled: method(disabled: boolean)
--# assume CA_UIC.ShaderTechniqueSet: method(technique: string | number, unknown: boolean)
--# assume CA_UIC.ShaderVarsSet: method(p1: number, p2: number, p3: number, p4: number, unknown: boolean)
--# assume CA_UIC.SimulateClick: method()
--# assume CA_UIC.SimulateMouseOn: method()
--# assume CA_UIC.Visible: method() --> boolean

--# assume CA_UIC.SetImage: method(path: string)
--# assume CA_UIC.SetCanResizeHeight: method(state: boolean)
--# assume CA_UIC.SetCanResizeWidth: method(state: boolean)
--# assume CA_UIC.SetTooltipText: method(tooltip: string, state: boolean?)
--# assume CA_UIC.GetStateText: method() --> string
--# assume CA_UIC.PropagatePriority: method(priority: number)
--# assume CA_UIC.Priority: method() --> number
--# assume CA_UIC.Bounds: method() --> (number, number)
--# assume CA_UIC.Height: method() --> number
--# assume CA_UIC.Width: method() --> number
--# assume CA_UIC.SetImageRotation:  method(unknown: number, rotation: number)
--# assume CA_UIC.ResizeTextResizingComponentToInitialSize: method(width: number, height: number)
--# assume CA_UIC.SimulateLClick: method()
--# assume CA_UIC.SimulateKey: method(keyString: string)


-- CAMPAIGN MANAGER
--# assume CM.get_game_interface: method() --> CA_GAME
--# assume CM.model: method() --> CA_MODEL
--# assume CM.is_multiplayer: method() --> boolean
--# assume CM.is_new_game: method() --> boolean
--# assume CM.get_local_faction: method(force: boolean?) --> string
--# assume CM.whose_turn_is_it: method() --> string
--# assume CM.get_human_factions: method() --> vector<string>
--get functions
--# assume CM.get_highest_ranked_general_for_faction: method(faction_key: string) --> CA_CHAR
--# assume CM.get_character_by_cqi: method(cqi: CA_CQI) --> CA_CHAR
--# assume CM.get_region: method(regionName: string) --> CA_REGION
--# assume CM.get_faction: method(factionName: string) --> CA_FACTION
--# assume CM.get_character_by_mf_cqi: method(cqi: CA_CQI) --> CA_CHAR
--# assume CM.char_lookup_str: method(char: CA_CQI | CA_CHAR | number) --> string
--UI
--# assume CM.get_campaign_ui_manager: method() --> CUIM
--# assume CM.disable_end_turn: method(opt: boolean)
--# assume CM.disable_shortcut: method(button: string, action: string, opt: boolean)
--# assume CM.override_ui: method(override: string, opt: boolean)
--# assume CM.steal_user_input: method(opt: bool)
--Camera
--# assume CM.scroll_camera_from_current: WHATEVER
--# assume CM.get_camera_position: method() --> (number, number, number, number)
--# assume CM.fade_scene: method(unknown: number, unknown2: number)
--callbacks
--# assume CM.add_game_created_callback: method(callback: function)
--# assume CM.callback: method(
--#     callback: function(),
--#     delay: number?,
--#     name: string?
--# )
--# assume CM.remove_callback: method(name: string)
--# assume CM.repeat_callback: method(
--#     callback: function(),
--#     delay: number,
--#     name: string
--# )
--# assume CM.add_turn_countdown_event: method(faction_name: string, turn_offset: number, event_name: string, context_str: string?)
--random number
--# assume CM.random_number: method(num: number | int, max: number?) --> int
--message events
--# assume CM.show_message_event_located: method(
--#     faction_key: string,
--#     primary_detail: string,
--#     secondary_detail: string,
--#     flavour_text: string,
--#     location_x: number,
--#     location_y: number,
--#     show_immediately: boolean,
--#     event_picture_id: number
--#)
--# assume CM.show_message_event: method(
--#    faction_key: string,
--#    primary_detail: string,
--#    secondary_detail: string,
--#    flavour_text: string,
--#    show_immediately: boolean,
--#    event_picture_id: number
--#)
--traits, ancillaries & skills
--# assume CM.force_add_trait: method(lookup: string, trait_key: string, showMessage: boolean)
--# assume CM.force_add_trait_on_selected_character: method(trait_key: string)
--# assume CM.force_remove_trait: method(lookup: string, trait_key: string)
--# assume CM.zero_action_points: method(charName: string)
--# assume CM.add_agent_experience: method(charName: string, experience: number)
--# assume CM.force_add_skill: method(lookup: string, skill_key: string)
--# assume CM.force_add_and_equip_ancillary: method(lookup: string, ancillary: string)
--More character commands
--# assume CM.award_experience_level: method(char_lookup_str: string, level: int)
--# assume CM.kill_character: method(lookup: CA_CQI, kill_army: boolean, throughcq: boolean)
--# assume CM.set_character_immortality: method(lookup: string, immortal: boolean)
--# assume CM.kill_all_armies_for_faction: method(factionName: CA_FACTION)
--# assume CM.teleport_to: method(charString: string, xPos: number, yPos: number, useCommandQueue: boolean)
--# assume CM.replenish_action_points: method(lookup:string)
--spawning
--# assume CM.create_force_with_general: method(
--#     faction_key: string,
--#     army_list: string,
--#     region_key: string,
--#     xPos: number,
--#     yPos: number,
--#     agent_type: string,
--#     agent_subtype: string,
--#     forename: string,
--#     clan_name: string,
--#     family_name: string,
--#     other_name: string,
--#     make_faction_leader: boolean,
--#     success_callback: function(CA_CQI)
--# )
--# assume CM.create_force: method(
--#     faction_key: string,
--#     unitstring: string,
--#     region_key: string,
--#     xPos: number,
--#     yPos: number,
--#     un1: boolean,
--#     un2: boolean,
--#     callback: (function(CA_CQI))?
--# )
--# assume CM.spawn_character_to_pool: method(
--#    factionKey: string, forname: string, familyName: string, clanName: string, 
--#    otherName: string, age: int, male: boolean, agentKey: string, agent_subtypeKey: string, 
--#    isImmortal: boolean, artSetId: string
--#)
--saving and loading
--# assume CM.add_saving_game_callback: method(function(context: WHATEVER))
--# assume CM.add_loading_game_callback: method(function(context: WHATEVER))
--# assume CM.set_saved_value: method(valueKey: string, value: any)
--# assume CM.get_saved_value: method(valueKey: string) --> WHATEVER
--# assume CM.save_named_value: method(name: string, value: any, context: WHATEVER?)
--# assume CM.load_named_value: method(name: string, default: any, context: WHATEVER?) --> WHATEVER
--# assume CM.disable_saving_game: method(opt: boolean)
--effect bundle commands
--# assume CM.apply_effect_bundle_to_region: method(bundle: string, region: string, turns: number)
--# assume CM.remove_effect_bundle_from_region: method(bundle: string, region: string)
--# assume CM.apply_effect_bundle_to_force: method(bundle: string, force: CA_CQI, turns: number)
--# assume CM.apply_effect_bundle: method(bundle: string, faction: string, turns: number)
--# assume CM.remove_effect_bundle: method(bundle: string, faction: string)
--# assume CM.apply_effect_bundle_to_characters_force: method(bundleKey: string, charCqi: CA_CQI, turns: number, useCommandQueue: boolean)
--# assume CM.remove_effect_bundle_from_characters_force: method(bundle_key: string, char_cqi: CA_CQI)
--unit manipulation
--# assume CM.remove_unit_from_character: method(lookup_string: string, unitID: string)
--# assume CM.grant_unit_to_character: method(lookup: string , unit: string)
--# assume CM.remove_all_units_from_general: method(character: CA_CHAR)
--diplomacy commands
--# assume CM.force_diplomacy:  method(faction: string, other_faction: string, record: string, offer: boolean, accept: boolean, enable_payments: boolean)
--# assume CM.make_diplomacy_available: method(faction: string, other_faction: string)
--# assume CM.force_make_peace: method(faction: string, other_faction: string)
--# assume CM.force_declare_war: method(declarer: string, declaree: string, attacker_allies: boolean, defender_allies: boolean)
--# assume CM.force_make_vassal: method(vassaliser: string, vassal: string)
--# assume CM.force_make_trade_agreement: method(faction1: string, faction2: string)
--# assume CM.faction_has_trade_agreement_with_faction: method( first_faction: CA_FACTION, second_faction: CA_FACTION)
--# assume CM.faction_has_nap_with_faction: method(first_faction: CA_FACTION, second_faction: CA_FACTION)
--# assume CM.force_confederation: method(confederator: string, confederated: string)
--# assume CM.force_alliance: method(faction: string, other_faction:string, unknown_bool: boolean)
--pending battle commands
--# assume CM.pending_battle_cache_get_defender: method(pos: int) --> (CA_CQI, CA_CQI, string)
--# assume CM.pending_battle_cache_get_attacker: method(pos: int) --> (CA_CQI, CA_CQI, string)
--# assume CM.pending_battle_cache_get_enemies_of_char: method(char: CA_CHAR) --> vector<CA_CHAR>
--# assume CM.pending_battle_cache_attacker_victory: method() --> boolean
--# assume CM.pending_battle_cache_faction_is_involved: method(faction_key: string) --> boolean
--# assume CM.pending_battle_cache_num_attackers: method() --> int
--# assume CM.pending_battle_cache_num_defenders: method() --> int
--CAI
--# assume CM.force_change_cai_faction_personality: method(key: string, personality: string)
---Markers
--# assume CM.add_marker: method(
--# name: string,
--# marker_type: CA_MARKER_TYPE,
--# location_x: number,
--# location_y: number,
--# location_z: number )
--# assume CM.remove_marker: method (name: string)
--Region Commands
--# assume CM.transfer_region_to_faction: method(region: string, faction:string)
--# assume CM.set_region_abandoned: method(region: string)
--autoresolve
--# assume CM.win_next_autoresolve_battle: method(faction: string)
--# assume CM.modify_next_autoresolve_battle: method(attacker_win_chance: number, defender_win_chance: number, attacker_losses_modifier: number, defender_losses_modifier: number, wipe_out_loser: boolean)
--events
--# assume CM.trigger_dilemma: method(faction_key: string, dilemma_key: string, trigger_immediately: boolean)
--# assume CM.trigger_incident: method(factionName: string, incidentKey: string, fireImmediately: boolean)
--# assume CM.trigger_mission: method(faction_key: string, mission_key: string, trigger_immediately: boolean)
--# assume CM.cancel_custom_mission: method(faction_key: string, mission_key: string)
--# assume CM.disable_event_feed_events: method(disable: boolean, categories: string, subcategories: string, events: string)
--# assume CM.complete_scripted_mission_objective: method(mission_key: string, objective_key: string, success: boolean)
--locks and unlocks
--# assume CM.lock_technology: method(faction_key: string, tech_key: string)
--# assume CM.unlock_starting_general_recruitment: method(startpos: string, faction: string)
--# assume CM.unlock_technology: method(faction_key: string, tech_key: string)
--# assume CM.add_event_restricted_unit_record_for_faction: method(unit: string, faction_key: string)
--# assume CM.remove_event_restricted_unit_record_for_faction: method(unit: string, faction_key: string)
--# assume CM.add_restricted_building_level_record: method(faction_key: string, building_key: string)
--# assume CM.remove_restricted_building_level_record: method(faction_key: string, building_key: string)
--rituals commands
--# assume CM.set_ritual_unlocked: method(cqi: CA_CQI, rite_key: string, unlock: boolean)
--# assume CM.set_ritual_chain_unlocked: method(cqi: CA_CQI, ritual_chain_key: string, unlock: boolean)
--# assume CM.rollback_linked_ritual_chain: method(chain_key: string, level: number)
--faction wide variables
--# assume CM.treasury_mod: method(faction_key: string, quantity: number)
--# assume CM.pooled_resource_mod: method(cqi: CA_CQI, pooled_resource: string, factor: string, quantity: number)
--# assume CM.faction_set_food_factor_value: method(faction_key: string, factor_key: string, quantity: number)
--checks
--# assume CM.char_is_mobile_general_with_army: method(char: CA_CHAR) --> boolean
--model overrides
--# assume CM.override_building_chain_display: method(building_chain: string, settlement_skin: string)

-- CAMPAIGN UI MANAGER
--# assume CUIM.get_char_selected: method() --> string
--# assume CUIM.settlement_selected: string
--# assume CUIM.override: method(ui_override: string) --> CUIM_OVERRIDE
--# assume CUIM.start_scripted_sequence: method()
--# assume CUIM.stop_scripted_sequence: method()

-- CAMPAIGN UI MANAGER OVERRIDES
--# assume CUIM_OVERRIDE.set_allowed: method(allowed: bool)

--CA CAMPAIGN_UI
--# assume CA_CampaignUI.TriggerCampaignScriptEvent: function(cqi: CA_CQI, event: string)
--# assume CA_CampaignUI.ClearSelection: function()
--# assume CA_CampaignUI.UpdateSettlementEffectIcons: function()

-- GAME INTERFACE
--# assume CA_GAME.filesystem_lookup: method(filePath: string, matchRegex:string) --> string


-- CHARACTER
--# assume CA_CHAR.has_trait: method(traitName: string) --> boolean
--# assume CA_CHAR.logical_position_x: method() --> number
--# assume CA_CHAR.logical_position_y: method() --> number
--# assume CA_CHAR.display_position_x: method() --> number
--# assume CA_CHAR.display_position_y: method() --> number
--# assume CA_CHAR.character_subtype_key: method() --> string
--# assume CA_CHAR.region: method() --> CA_REGION
--# assume CA_CHAR.faction: method() --> CA_FACTION
--# assume CA_CHAR.military_force: method() --> CA_MILITARY_FORCE
--# assume CA_CHAR.garrison_residence: method() --> CA_GARRISON_RESIDENCE
--# assume CA_CHAR.character_subtype: method(subtype: string) --> boolean
--# assume CA_CHAR.character_type: method(char_type: string) --> boolean
--# assume CA_CHAR.get_forename: method() --> string
--# assume CA_CHAR.get_surname: method() --> string
--# assume CA_CHAR.command_queue_index: method() --> CA_CQI
--# assume CA_CHAR.cqi: method() --> CA_CQI
--# assume CA_CHAR.rank: method() --> int
--# assume CA_CHAR.won_battle: method() --> boolean
--# assume CA_CHAR.battles_fought: method() --> number
--# assume CA_CHAR.is_wounded: method() --> boolean
--# assume CA_CHAR.has_military_force: method() --> boolean
--# assume CA_CHAR.is_faction_leader: method() --> boolean
--# assume CA_CHAR.family_member: method() --> CA_CHAR
--# assume CA_CHAR.is_null_interface: method() --> boolean
--# assume CA_CHAR.has_skill: method(skill_key: string) --> boolean
--# assume CA_CHAR.is_politician: method() --> boolean
--# assume CA_CHAR.has_garrison_residence: method() --> boolean

-- CHARACTER LIST
--# assume CA_CHAR_LIST.num_items: method() --> number
--# assume CA_CHAR_LIST.item_at: method(index: number) --> CA_CHAR


-- MILITARY FORCE
--# assume CA_MILITARY_FORCE.general_character: method() --> CA_CHAR
--# assume CA_MILITARY_FORCE.unit_list: method() --> CA_UNIT_LIST
--# assume CA_MILITARY_FORCE.command_queue_index: method() --> CA_CQI
--# assume CA_MILITARY_FORCE.has_effect_bundle: method(bundle: string) --> boolean
--# assume CA_MILITARY_FORCE.character_list: method() --> CA_CHAR_LIST
--# assume CA_MILITARY_FORCE.has_general: method() --> boolean
--# assume CA_MILITARY_FORCE.is_armed_citizenry: method() --> boolean
--# assume CA_MILITARY_FORCE.morale: method() --> number

-- MILITARY FORCE LIST
--# assume CA_MILITARY_FORCE_LIST.num_items: method() --> number
--# assume CA_MILITARY_FORCE_LIST.item_at: method(index: number) --> CA_MILITARY_FORCE

--UNIT
--# assume CA_UNIT.faction: method() --> CA_FACTION
--# assume CA_UNIT.unit_key: method() --> string
--# assume CA_UNIT.has_force_commander: method() --> boolean
--# assume CA_UNIT.force_commander: method() --> CA_CHAR
--# assume CA_UNIT.military_force: method() --> CA_MILITARY_FORCE
--# assume CA_UNIT.has_military_force: method() --> boolean
--# assume CA_UNIT.percentage_proportion_of_full_strength: method() --> number


--UNIT_LIST

--#assume CA_UNIT_LIST.num_items: method() --> number
--# assume CA_UNIT_LIST.item_at: method(j: number) --> CA_UNIT
--# assume CA_UNIT_LIST.has_unit: method(unit: string) --> boolean

-- REGION
--# assume CA_REGION.settlement: method() --> CA_SETTLEMENT
--# assume CA_REGION.garrison_residence: method() --> CA_GARRISON_RESIDENCE
--# assume CA_REGION.name: method() --> string
--# assume CA_REGION.province_name: method() --> string
--# assume CA_REGION.public_order: method() --> number
--# assume CA_REGION.is_null_interface: method() --> boolean
--# assume CA_REGION.is_abandoned: method() --> boolean
--# assume CA_REGION.owning_faction: method() --> CA_FACTION
--# assume CA_REGION.slot_list: method() --> CA_SLOT_LIST
--# assume CA_REGION.is_province_capital: method() --> boolean
--# assume CA_REGION.building_exists: method(building: string) --> boolean
--# assume CA_REGION.resource_exists: method(resource_key: string) --> boolean
--# assume CA_REGION.any_resource_available: method() --> boolean
--# assume CA_REGION.adjacent_region_list: method() --> CA_REGION_LIST

-- SETTLEMENT
--# assume CA_SETTLEMENT.logical_position_x: method() --> number
--# assume CA_SETTLEMENT.logical_position_y: method() --> number
--# assume CA_SETTLEMENT.display_position_x: method() --> number
--# assume CA_SETTLEMENT.display_position_y: method() --> number
--# assume CA_SETTLEMENT.get_climate: method() --> string
--# assume CA_SETTLEMENT.is_null_interface: method() --> boolean
--# assume CA_SETTLEMENT.faction: method() -->CA_FACTION
--# assume CA_SETTLEMENT.commander: method() --> CA_CHAR
--# assume CA_SETTLEMENT.has_commander: method() --> boolean
--# assume CA_SETTLEMENT.slot_list: method() --> CA_SLOT_LIST
--# assume CA_SETTLEMENT.is_port: method() --> boolean
--# assume CA_SETTLEMENT.region: method() --> CA_REGION
--SLOT LIST
--# assume CA_SLOT_LIST.num_items: method() --> number
--# assume CA_SLOT_LIST.item_at: method(index: number) --> CA_SLOT
--# assume CA_SLOT_LIST.slot_type_exists: method(slot_key: string) --> boolean
--# assume CA_SLOT_LIST.building_type_exists: method(building_key: string) --> boolean


--SLOT
--# assume CA_SLOT.has_building: method() --> boolean
--# assume CA_SLOT.building: method() --> CA_BUILDING
--# assume CA_SLOT.resource_key: method() --> string


--BUILDING
--# assume CA_BUILDING.name: method() --> string
--# assume CA_BUILDING.chain: method() --> string
--# assume CA_BUILDING.superchain: method() --> string
--# assume CA_BUILDING.faction: method() --> CA_FACTION
--# assume CA_BUILDING.region: method() --> CA_REGION

-- GARRISON RESIDENCE
--# assume CA_GARRISON_RESIDENCE.region: method() --> CA_REGION
--# assume CA_GARRISON_RESIDENCE.faction: method() --> CA_FACTION
--# assume CA_GARRISON_RESIDENCE.is_under_siege: method() --> boolean
--# assume CA_GARRISON_RESIDENCE.settlement_interface: method() --> CA_SETTLEMENT
--# assume CA_GARRISON_RESIDENCE.army: method() --> CA_MILITARY_FORCE
--# assume CA_GARRISON_RESIDENCE.command_queue_index: method() --> CA_CQI
--# assume CA_GARRISON_RESIDENCE.unit_count: method() --> number
--# assume CA_GARRISON_RESIDENCE.can_be_occupied_by_faction: method(faction_key: string) --> boolean

-- MODEL
--# assume CA_MODEL.world: method() --> CA_WORLD
--# assume CA_MODEL.difficulty_level: method() --> number
--# assume CA_MODEL.turn_number: method() --> number
--# assume CA_MODEL.pending_battle: method() --> CA_PENDING_BATTLE
--# assume CA_MODEL.combined_difficulty_level: method() --> int
--# assume CA_MODEL.campaign_name: method(campaign_name: string) --> boolean
--# assume CA_MODEL.campaign_type: method() --> number
--# assume CA_MODEL.is_multiplayer: method() --> boolean
--# assume CA_MODEL.military_force_for_command_queue_index: method(cqi: CA_CQI) --> CA_MILITARY_FORCE
--# assume CA_MODEL.character_for_command_queue_index: method(cqi: CA_CQI) --> CA_CHAR
--# assume CA_MODEL.random_percent: method(chance: number) --> boolean
--# assume CA_MODEL.faction_is_local: method(faction_key: string) --> boolean
--# assume CA_MODEL.faction_for_command_queue_index: method(cqi: CA_CQI) --> CA_FACTION

-- WORLD
--# assume CA_WORLD.faction_list: method() --> CA_FACTION_LIST
--# assume CA_WORLD.faction_by_key: method(factionKey: string) --> CA_FACTION
--# assume CA_WORLD.whose_turn_is_it: method() --> CA_FACTION
--# assume CA_WORLD.region_manager: method() --> CA_REGION_MANAGER

--REGION_MANAGER
--# assume CA_REGION_MANAGER.region_list: method() --> CA_REGION_LIST
--# assume CA_REGION_MANAGER.region_by_key: method(key: string) --> CA_REGION


-- FACTION
--# assume CA_FACTION.character_list: method() --> CA_CHAR_LIST
--# assume CA_FACTION.treasury: method() --> number
--# assume CA_FACTION.name: method() --> string
--# assume CA_FACTION.subculture: method() --> string
--# assume CA_FACTION.culture: method() --> string
--# assume CA_FACTION.military_force_list: method() --> CA_MILITARY_FORCE_LIST
--# assume CA_FACTION.is_human: method() --> boolean
--# assume CA_FACTION.is_dead: method() --> boolean
--# assume CA_FACTION.is_vassal_of: method(faction: CA_FACTION) --> boolean
--# assume CA_FACTION.is_vassal: method() --> boolean
--# assume CA_FACTION.is_ally_vassal_or_client_state_of: method(faction: string) --> boolean
--# assume CA_FACTION.allied_with: method(faction: CA_FACTION)
--# assume CA_FACTION.at_war_with: method(faction: CA_FACTION) --> boolean
--# assume CA_FACTION.region_list: method() --> CA_REGION_LIST
--# assume CA_FACTION.has_effect_bundle: method(bundle:string) --> boolean
--# assume CA_FACTION.home_region: method() --> CA_REGION
--# assume CA_FACTION.command_queue_index: method() --> CA_CQI
--# assume CA_FACTION.is_null_interface: method() --> boolean
--# assume CA_FACTION.faction_leader: method() --> CA_CHAR
--# assume CA_FACTION.has_home_region: method() --> boolean
--# assume CA_FACTION.factions_met: method() --> CA_FACTION_LIST
--# assume CA_FACTION.factions_at_war_with: method() --> CA_FACTION_LIST
--# assume CA_FACTION.at_war: method() --> boolean
--# assume CA_FACTION.has_pooled_resource: method(resource: string) --> boolean
--# assume CA_FACTION.pooled_resource: method(resource: string) --> CA_POOLED
--# assume CA_FACTION.rituals: method() --> CA_FACTION_RITUALS
--# assume CA_FACTION.has_rituals: method() --> boolean
--# assume CA_FACTION.holds_entire_province: method(province_key: string, include_vassals: boolean)

-- FACTION LIST
--# assume CA_FACTION_LIST.num_items: method() --> number
--# assume CA_FACTION_LIST.item_at: method(index: number) --> CA_FACTION

--REGION LIST
--# assume CA_REGION_LIST.num_items: method() --> number
--# assume CA_REGION_LIST.item_at: method(i: number) --> CA_REGION

-- EFFECT
--# assume CA_EFFECT.get_localised_string: function(key: string) --> string


-- PENDING BATTLE
--# assume CA_PENDING_BATTLE.attacker: method() --> CA_CHAR
--# assume CA_PENDING_BATTLE.defender: method() --> CA_CHAR
--# assume CA_PENDING_BATTLE.ambush_battle: method() --> boolean
--# assume CA_PENDING_BATTLE.has_been_fought: method() --> boolean
--# assume CA_PENDING_BATTLE.set_piece_battle_key: method() --> string

-- CORE
--# assume CORE.get_ui_root: method() --> CA_UIC
--# assume CORE.add_listener: method(
--#     listenerName: string,
--#     eventName: string,
--#     conditionFunc: (function(context: WHATEVER?) --> boolean) | boolean,
--#     listenerFunc: function(context: WHATEVER?),
--#     persistent: boolean
--# )
--# assume CORE.remove_listener: method(listenerName: string)
--# assume CORE.add_ui_created_callback: method(function())
--# assume CORE.get_screen_resolution: method() --> (number, number)
--# assume CORE.trigger_event: method(event_name: string, any...)

-- POOLED RESOURCE
--# assume CA_POOLED.value: method() --> number

--FACTION RITUALS
--# assume CA_FACTION_RITUALS.active_rituals: method() --> CA_RITUAL_LIST
--# assume CA_FACTION_RITUALS.ritual_status: method(ritual_key: string) --> boolean

--RITUAL
--# assume CA_RITUAL.ritual_sites: method() --> CA_REGION_LIST
--# assume CA_RITUAL.ritual_chain_key: method() --> string
--# assume CA_RITUAL.ritual_key: method() --> string
--# assume CA_RITUAL.is_part_of_chain: method() --> boolean
--# assume CA_RITUAL.target_faction: method() --> CA_FACTION
--# assume CA_RITUAL.cast_time: method() --> number
--# assume CA_RITUAL.is_null_interface: method() --> boolean
--# assume CA_RITUAL.cooldown_time: method() --> number
--# assume CA_RITUAL.expended_resources: method() --> number
--# assume CA_RITUAL.slave_cost: method() --> number
--# assume CA_RITUAL.ritual_category: method() --> string


--RITUAL LIST
--# assume CA_RITUAL_LIST.item_at: method(i: int) --> CA_RITUAL
--# assume CA_RITUAL_LIST.is_empty: method() --> boolean
--# assume CA_RITUAL_LIST.num_items: method() --> int


-- GLOBAL FUNCTIONS
-- COMMON
--# assume global find_uicomponent: function(parent: CA_UIC, string...) --> CA_UIC
--# assume global UIComponent: function(pointer: CA_Component) --> CA_UIC
--# assume global find_uicomponent_from_table: function(root: CA_UIC, findtable: vector<string>) --> CA_UIC
--# assume global uicomponent_descended_from: function(root: CA_UIC, parent_name: string) --> boolean
--# assume global out: function(out: string | number)  
--# assume global print_all_uicomponent_children: function(component: CA_UIC)
--# assume global is_uicomponent: function(object: any) --> boolean
--# assume global output_uicomponent: function(uic: CA_UIC, omit_children: boolean)
--# assume global wh_faction_is_horde: function(faction: CA_FACTION) --> boolean
--# assume global uicomponent_to_str: function(component: CA_UIC) --> string
--# assume global is_string: function(arg: string) --> boolean
--# assume global is_table: function(arg: table) --> boolean
--# assume global is_number: function(arg: number) --> boolean
--# assume global is_function: function(arg: function) --> boolean
--# assume global is_boolean: function(arg: boolean) --> boolean
--# assume global get_timestamp: function() --> string
--# assume global script_error: function(msg: string)
--# assume global to_number: function(n: any) --> number
--# assume global load_script_libraries: function()

-- CAMPAIGN
--# assume global get_cm: function() --> CM
--# assume global get_events: function() --> map<string, vector<function(context:WHATEVER?)>>
--# assume global Get_Character_Side_In_Last_Battle: function(char: CA_CHAR) --> BATTLE_SIDE
--# assume global q_setup: function()
--# assume global set_up_rank_up_listener: function(quest_table: vector<vector<string | number>>, subtype: string, infotext: vector<string | number>)
--# assume global CampaignUI: CA_CampaignUI





--CA LUA OBJECTS:

-- RITES UNLOCK OBJECT

--# assume global class RITE_UNLOCK

--# assume RITE_UNLOCK.new: method(rite_key: string, event_name: string, condition: function(context: WHATEVER, faction_name: string)--> boolean, faction: string?) --> RITE_UNLOCK
--# assume RITE_UNLOCK.start: method(human_faction_name: string)

-- MISSION MANAGER OBJECT

--# assume global class MISSION_MANAGER
--# type global CA_MISSION_OBJECTIVE =
--# "CAPTURE_REGIONS" | "SCRIPTED" | "RAZE_OR_SACK_N_DIFFERENT_SETTLEMENTS_INCLUDING" |
--# "ELIMINATE_CHARACTER_IN_BATTLE" | "MOVE_TO_REGION" | "DEFEAT_N_ARMIES_OF_FACTION"
--creation
--# assume MISSION_MANAGER.new: method(faction_key: string, mission_key: string, success_callback: function?, failure_callback: function?, cancellation_callback: function?) --> MISSION_MANAGER

--basic
--# assume MISSION_MANAGER.add_new_objective: method(objective_type: CA_MISSION_OBJECTIVE)
--# assume MISSION_MANAGER.add_condition: method(condition_string: string)
--# assume MISSION_MANAGER.add_payload: method(payload_string: string)
--# assume MISSION_MANAGER.set_turn_limit: method(turns: number)
--# assume MISSION_MANAGER.set_chapter: method(turns: integer)
--# assume MISSION_MANAGER.set_mission_issuer: method(issuer: string)
--localisation
--# assume MISSION_MANAGER.add_heading: method(heading_loc_key: string)
--# assume MISSION_MANAGER.add_description: method(description_loc_key: string)
--scripted objectives
------Here, string key can be ommited when creating an objective. This will generate it randomly. The script key can only be ommitted from other functions if there is only one scripted objective.
--# assume MISSION_MANAGER.add_new_scripted_objective: method(objective_loc_key: string, event: string, condition: function(context: WHATEVER) --> boolean, script_key: string?)
--# assume MISSION_MANAGER.add_scripted_objective_success_condition: method(event: string, condition: function(context: WHATEVER) --> boolean, script_key: string?)
--# assume MISSION_MANAGER.add_scripted_objective_failure_condition: method(event: string, condition: function(context: WHATEVER) --> boolean, script_key: string?)
--# assume MISSION_MANAGER.force_scripted_objective_success: method(script_key: string?)
--# assume MISSION_MANAGER.force_scripted_objective_failure: method(script_key: string?)
--# assume MISSION_MANAGER.update_scripted_objective_text: method(override_text_loc: string, script_key: string?)

--# assume MISSION_MANAGER.set_should_cancel_before_issuing: method(boolean)
--# assume MISSION_MANAGER.set_should_should_whitelist: method(boolean)

--# assume MISSION_MANAGER.set_first_time_startup_callback: method(callback: function())
--# assume MISSION_MANAGER.set_each_time_startup_callback: method(callback: function())

--# assume MISSION_MANAGER.trigger: method(dismiss_callback: function?, delay: number?)
--# assume CM.get_mission_manager: method(mission_key: string) --> MISSION_MANAGER

-- RANDOM ARMY MANAGER OBJECT
--# assume global class RAM
--# assume RAM.new_force: method(key: string)
--# assume RAM.add_mandatory_unit: method(forcekey: string, unitkey: string, q: number)
--# assume RAM.add_unit: method(forcekey: string, unitkey: string, q: number)
--# assume RAM.generate_force: method(id: string, sizes: {number, number}) --> string

-- CAMPAIGN CUTSCENE OBJECT
--# assume global class CA_CUTSCENE
--# assume CA_CUTSCENE.new: method(key: string, time: number) --> CA_CUTSCENE
--# assume CA_CUTSCENE.set_disable_settlement_labels: method(setting: boolean)
--# assume CA_CUTSCENE.set_restore_shroud: method(setting: boolean)
--# assume CA_CUTSCENE.action: method(action: function(), timer: number)


-- LL UNLOCK OBJECT
--# assume global class LL_UNLOCK
--# assume LL_UNLOCK.new: method(faction_key: string, startpos_id: string, forename_key: string, event: string, condition: (function(context: WHATEVER) --> boolean)) --> LL_UNLOCK
--# assume LL_UNLOCK.start: method()

--INVASION MANAGER OBJECT
--# assume global class INVASION_MANAGER
--# assume global class INVASION
--# type global INVASION_TARGETS = "NONE" | "REGION" | "LOCATION" | "CHARACTER" | "PATROL"
--# assume INVASION_MANAGER.new_invasion: method(name: string, faction: string, units: string, coordinates: vector<number>) --> INVASION

--# assume INVASION.set_target: method(target_type: INVASION_TARGETS, target: WHATEVER, target_faction_key: string)
--# assume INVASION.apply_effect: method(effect_key: string, turns: number)
--# assume INVASION.add_character_experience: method(quanity: number)
--# assume INVASION.add_unit_experience: method(quantity: number)
--# assume INVASION.start_invasion: method(callback: function?, declare_war: boolean?, invite_attacker_allies: boolean?, invite_defender_allies: boolean?)

-- GLOBAL VARIABLES
--leave at the bottom of this file
--# assume global cm: CM
--# assume global core: CORE
--# assume global effect: CA_EFFECT
--# assume global __write_output_to_logfile: boolean
--# assume global mission_manager: MISSION_MANAGER
--# assume global rite_unlock: RITE_UNLOCK
--# assume global ll_unlock: LL_UNLOCK
--# assume global random_army_manager: RAM
--# assume global campaign_cutscene: CA_CUTSCENE
--# assume global invasion_manager: INVASION_MANAGER

--string extensions
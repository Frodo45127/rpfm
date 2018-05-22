// In this file are all the helper functions used by the UI when editing Text PackedFiles.
extern crate std;
extern crate gtk;
extern crate sourceview;
extern crate failure;

use std::cell::RefCell;
use std::rc::Rc;
use failure::Error;
use sourceview::prelude::*;
use sourceview::{
    Buffer, View, Language, LanguageManager, StyleScheme, StyleSchemeManager, CompletionWords
};
use gtk::prelude::*;
use gtk::ScrolledWindow;

use packfile::packfile::PackFile;
use common::coding_helpers::*;
use AppUI;
use packfile::update_packed_file_data_text;
use ui::*;


/// This function is used to create a ScrolledWindow with the SourceView inside. If there is an
/// error, just say it in the statusbar.
pub fn create_text_view(
    app_ui: &AppUI,
    pack_file: &Rc<RefCell<PackFile>>,
    packed_file_decoded_index: &usize,
    is_packedfile_opened: &Rc<RefCell<bool>>
) -> Result<(), Error> {

    // Get the name of the PackedFile.
    let packed_file_name = &pack_file.borrow().data.packed_files[*packed_file_decoded_index].path.last().unwrap().to_owned();

    // Try to decode the PackedFile as a normal UTF-8 string.
    let mut decoded_string = decode_string_u8(&pack_file.borrow().data.packed_files[*packed_file_decoded_index].data);

    // If there is an error, try again as ISO_8859_1, as there are some text files using that encoding.
    if decoded_string.is_err() {
        if let Ok(string) = decode_string_u8_iso_8859_1(&pack_file.borrow().data.packed_files[*packed_file_decoded_index].data) {
            decoded_string = Ok(string);
        }
    }

    // We try to decode the data. Only if we success, we create the SourceView and add the data to it.
    // NOTE: This only works for UTF-8 and ISO_8859_1 encoded files. Check their encoding before adding them here to be decoded.
    match decoded_string {
        Ok(text) => {

            // We create the new SourceView (in a ScrolledWindow) and his buffer,
            // get his buffer and put the text in it.
            let source_view_scroll = ScrolledWindow::new(None, None);
            let source_view_buffer: Buffer = Buffer::new(None);
            let source_view = View::new_with_buffer(&source_view_buffer);

            // We config the SourceView for our needs.
            source_view_scroll.set_vexpand(true);
            source_view_scroll.set_hexpand(true);
            source_view.set_tab_width(4);
            source_view.set_show_line_numbers(true);
            source_view.set_indent_on_tab(true);
            source_view.set_highlight_current_line(true);

            // Set the syntax hightlight to "monokai-extended".
            // TODO: Make this be toggleable through the preferences window.
            let style_scheme_manager = StyleSchemeManager::get_default().unwrap();
            let style: Option<StyleScheme> = style_scheme_manager.get_scheme("monokai-extended");
            if let Some(style) = style { source_view_buffer.set_style_scheme(&style); }

            // We attach it to the main grid.
            app_ui.packed_file_data_display.attach(&source_view_scroll, 0, 0, 1, 1);

            // Then, we get the Language of the file.
            let language_manager = LanguageManager::get_default().unwrap();
            let packedfile_language: Option<Language> = if packed_file_name.ends_with(".xml") ||
                packed_file_name.ends_with(".xml.shader") ||
                packed_file_name.ends_with(".xml.material") ||
                packed_file_name.ends_with(".variantmeshdefinition") ||
                packed_file_name.ends_with(".environment") ||
                packed_file_name.ends_with(".lighting") ||
                packed_file_name.ends_with(".wsmodel") {

                // For any of this, use xml.
                language_manager.get_language("xml")
            }
            else if packed_file_name.ends_with(".lua") {

                // Completion Stuff from here.
                // Get the `Completion` of our `View`.
                let completion = source_view.get_completion().unwrap();

                // Get the lists of lua keywods and TW functions for autocompletion.
                let keyword_list = get_keyword_list();
                let fn_list = get_function_list();

                // Enable autocompletion from a secondary buffer with all the functions and stuff.
                let fn_buffer = Buffer::new(None);
                fn_buffer.begin_not_undoable_action();
                fn_buffer.set_text(&fn_list);
                fn_buffer.end_not_undoable_action();
                let tw_fn_provider = CompletionWords::new(Some("Total War Functions"), None);
                tw_fn_provider.register(&fn_buffer);
                completion.add_provider(&tw_fn_provider).unwrap();

                let keyword_buffer = Buffer::new(None);
                keyword_buffer.begin_not_undoable_action();
                keyword_buffer.set_text(&keyword_list);
                keyword_buffer.end_not_undoable_action();
                let lua_keyword_provider = CompletionWords::new(Some("Lua Keywords"), None);
                lua_keyword_provider.register(&keyword_buffer);
                completion.add_provider(&lua_keyword_provider).unwrap();

                // Enable autocompletion with words from the same file.
                let local_provider = CompletionWords::new(Some("Current File"), None);
                local_provider.register(&source_view_buffer);
                completion.add_provider(&local_provider).unwrap();

                // Return the Language.
                language_manager.get_language("lua")
            }
            else if packed_file_name.ends_with(".csv") ||
                packed_file_name.ends_with(".tsv") {
                language_manager.get_language("csv")
            }
            else if packed_file_name.ends_with(".inl") {

                // These seem to be written in C++.
                language_manager.get_language("cpp")
            }
            else {

                // If none of the conditions has been met, it's a plain text file.
                None
            };

            // Then we set the Language of the file, if it has one.
            if let Some(language) = packedfile_language {
                source_view_buffer.set_language(&language);
            }

            // Add the text to the SourceView.
            source_view_buffer.begin_not_undoable_action();
            source_view_buffer.set_text(&*text);
            source_view_buffer.end_not_undoable_action();

            // And show everything.
            source_view_scroll.add(&source_view);
            app_ui.packed_file_data_display.show_all();

            // When we destroy the `ScrolledWindow`, we need to tell the program we no longer have an open PackedFile.
            source_view_scroll.connect_destroy(clone!(
                is_packedfile_opened => move |_| {
                    *is_packedfile_opened.borrow_mut() = false;
                }
            ));

            // In case we change anything in the sourceview buffer, we save the PackedFile.
            source_view_buffer.connect_changed(clone!(
                app_ui,
                pack_file,
                packed_file_decoded_index => move |source_view_buffer| {
                    let packed_file_data = encode_string_u8(&source_view_buffer.get_slice(
                        &source_view_buffer.get_start_iter(),
                        &source_view_buffer.get_end_iter(),
                        true
                    ).unwrap());

                    update_packed_file_data_text(
                        &packed_file_data,
                        &mut pack_file.borrow_mut(),
                        packed_file_decoded_index
                    );

                    set_modified(true, &app_ui.window, &mut *pack_file.borrow_mut());
                }
            ));

            // Return success.
            Ok(())
        }
        Err(error) => Err(error)
    }
}

// Complete list of lua functions used in Total War: Warhammer, filtered so we can use it.
// Source: https://pastebin.com/9v2wtZPy
fn get_function_list() -> String {
    "enable_ui
    add_restricted_building_level_record
    remove_restricted_building_level_record
    add_restricted_building_level_record_for_faction
    remove_restricted_building_level_record_for_faction
    add_custom_battlefield
    remove_custom_battlefield
    add_visibility_trigger
    remove_visibility_trigger
    add_location_trigger
    remove_location_trigger
    disable_elections
    register_movies
    disable_movement_for_ai_under_shroud
    cancel_actions_for
    shown_message
    pending_auto_show_messages
    compare_localised_string
    advance_to_next_campaign
    add_time_trigger
    remove_time_trigger
    scroll_camera
    scroll_camera_with_direction
    stop_camera
    set_camera_position
    get_camera_position
    fade_scene
    dismiss_advice
    disable_movement_for_character
    disable_movement_for_faction
    show_shroud
    take_shroud_snapshot
    restore_shroud_from_snapshot
    make_neighbouring_regions_visible_in_shroud
    make_neighbouring_regions_seen_in_shroud
    make_sea_region_visible_in_shroud
    make_sea_region_seen_in_shroud
    force_diplomacy
    force_diplomacy_new
    display_turns
    stop_user_input
    steal_user_input
    steal_escape_key
    is_new_game
    save_named_value
    load_named_value
    disable_shopping_for_ai_under_shroud
    add_settlement_model_override
    add_building_model_override
    remove_settlement_model_override
    remove_building_model_override
    optional_extras_for_episodics
    award_experience_level
    add_agent_experience
    enable_auto_generated_missions
    add_unit_model_overrides
    add_attack_of_opportunity_overrides
    remove_attack_of_opportunity_overrides
    register_instant_movie
    register_outro_movie
    show_message_event
    show_message_event_located
    grant_unit
    grant_unit_to_character
    force_assassination_success_for_human
    force_garrison_infiltration_success_for_human
    set_tax_rate
    exempt_region_from_tax
    exempt_province_from_tax_for_all_factions_and_set_default
    disable_rebellions_worldwide
    force_declare_war
    force_make_peace
    force_add_trait
    force_remove_trait
    force_add_ancillary
    add_ancillary_to_faction
    force_add_skill
    add_marker
    remove_marker
    add_vfx
    remove_vfx
    force_make_trade_agreement
    move_to
    teleport_to
    join_garrison
    leave_garrison
    attack
    attack_region
    seek_exchange
    set_looting_options_disabled_for_human
    disable_saving_game
    set_non_scripted_traits_disabled
    set_non_scripted_ancillaries_disabled
    set_technology_research_disabled
    set_liberation_options_disabled
    set_ui_notification_of_victory_disabled
    enable_movement_for_character
    enable_movement_for_faction
    create_force
    create_force_with_general
    create_force_with_existing_general
    create_force_with_full_diplomatic_discovery
    disable_end_turn
    end_turn
    force_normal_character_locomotion_speed_for_turn
    hide_character
    unhide_character
    add_circle_area_trigger
    add_outline_area_trigger
    remove_area_trigger
    dismiss_advice_at_end_turn
    disable_shortcut
    apply_effect_bundle
    remove_effect_bundle
    apply_effect_bundle_to_force
    remove_effect_bundle_from_force
    apply_effect_bundle_to_characters_force
    remove_effect_bundle_from_characters_force
    apply_effect_bundle_to_region
    remove_effect_bundle_from_region
    create_agent
    instantly_dismantle_building
    instantly_upgrade_building
    win_next_autoresolve_battle
    modify_next_autoresolve_battle
    replenish_action_points
    zero_action_points
    override_ui
    kill_character
    wound_character
    force_agent_action_success_for_human
    instantly_repair_building
    render_campaign_to_file
    force_character_force_into_stance
    model
    set_region_abandoned
    override_attacker_win_chance_prediction
    transfer_region_to_faction
    suppress_all_event_feed_event_types
    whitelist_event_feed_event_type
    event_feed_event_type_pending
    highlight_movement_extents
    highlight_selected_character_zoc
    spawn_rogue_army
    create_storm_for_region
    trigger_mission
    trigger_dilemma
    trigger_incident
    trigger_custom_mission
    trigger_custom_mission_from_string
    cancel_custom_mission
    trigger_custom_dilemma
    grant_faction_handover
    set_campaign_ai_force_all_factions_boardering_humans_to_have_invasion_behaviour
    set_campaign_ai_force_all_factions_boardering_human_vassals_to_have_invasion_behaviour
    technology_osmosis_for_playables_enable_culture
    technology_osmosis_for_playables_enable_all
    force_make_vassal
    force_alliance
    force_grant_military_access
    force_rebellion_in_region
    treasury_mod
    lock_technology
    unlock_technology
    set_general_offered_dilemma_permitted
    instant_set_building_health_percent
    make_son_come_of_age
    allow_player_to_embark_navies
    force_change_cai_faction_personality
    set_ignore_end_of_turn_public_order
    set_only_allow_basic_recruit_stance
    set_imperium_level_change_disabled
    set_character_experience_disabled
    set_character_skill_tier_limit
    set_event_generation_enabled
    autosave_at_next_opportunity
    add_event_restricted_unit_record
    remove_event_restricted_unit_record
    add_event_restricted_unit_record_for_faction
    remove_event_restricted_unit_record_for_faction
    add_event_restricted_building_record
    remove_event_restricted_building_record
    add_event_restricted_building_record_for_faction
    remove_event_restricted_building_record_for_faction
    set_ai_uses_human_display_speed
    set_public_order_of_province_for_region
    set_public_order_disabled_for_province_for_region
    set_public_order_disabled_for_province_for_region_for_all_factions_and_set_default
    make_region_visible_in_shroud
    make_region_seen_in_shroud
    modify_faction_slaves_in_a_faction
    modify_faction_slaves_in_a_faction_province
    add_development_points_to_region
    faction_set_food_factor_multiplier
    faction_set_food_factor_value
    faction_mod_food_factor_value
    faction_add_pooled_resource
    add_interactable_campaign_marker
    remove_interactable_campaign_marker
    complete_scripted_mission_objective
    set_scripted_mission_text
    spawn_unique_agent
    spawn_unique_agent_at_region
    spawn_unique_agent_at_character
    add_character_vfx
    remove_character_vfx
    add_garrison_residence_vfx
    remove_garrison_residence_vfx
    pooled_resource_mod
    perform_ritual
    rollback_linked_ritual_chain
    set_ritual_unlocked
    set_ritual_chain_unlocked
    set_ritual_in_chain_unlocked
    make_diplomacy_available
    remove_unit_from_character
    skip_winds_of_magic_gambler
    spawn_character_to_pool
    override_building_chain_display
    cai_strategic_stance_manager_block_all_stances_but_that_specified_towards_target_faction
    cai_strategic_stance_manager_promote_specified_stance_towards_target_faction
    cai_strategic_stance_manager_promote_specified_stance_towards_target_faction_by_number
    cai_force_personality_change
    cai_force_personality_change_with_override_round_number
    cai_strategic_stance_manager_force_stance_update_between_factions
    cai_strategic_stance_manager_set_stance_promotion_between_factions_for_a_given_stance
    cai_strategic_stance_manager_clear_all_promotions_between_factions
    cai_strategic_stance_manager_set_stance_blocking_between_factions_for_a_given_stance
    cai_strategic_stance_manager_clear_all_blocking_between_factions
    cai_disable_movement_for_character
    cai_disable_movement_for_faction
    cai_enable_movement_for_character
    cai_enable_movement_for_faction
    cai_disable_command_assignment_for_character
    cai_enable_command_assignment_for_character
    faction_offers_peace_to_other_faction
    disable_pathfinding_restriction
    set_character_immortality
    set_character_unique
    spawn_character_into_family_tree
    appoint_character_to_most_expensive_force
    play_movie_in_ui
    trigger_music_state
    is_benchmark_mode
    cinematic
    filesystem_lookup
    disable_event_feed_events
    lock_starting_general_recruitment
    unlock_starting_general_recruitment
    toggle_dilemma_generation
    toggle_incident_generation
    toggle_mission_generation
    add_building_to_force
    add_units_to_faction_mercenary_pool
    add_units_to_province_mercenary_pool
".to_owned()
}

// Complete list of lua keywords.
fn get_keyword_list() -> String {
    "and
    break
    do
    else
    elseif
    end
    for
    function
    goto
    if
    in
    local
    not
    or
    repeat
    return
    then
    until
    while
".to_owned()
}

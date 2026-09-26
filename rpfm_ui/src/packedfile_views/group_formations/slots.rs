//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with the slots for GroupFormations Views.

use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;

use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QString;
use qt_core::QVariant;
use qt_core::SlotNoArgs;
use qt_core::SlotOfQModelIndex;
use qt_core::SlotOfQString;

use anyhow::Result;
use getset::Getters;

use std::rc::Rc;
use std::sync::Arc;


use rpfm_ui_common::clone;

use crate::app_ui::AppUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::show_dialog;

use super::*;

/// Name given to new formations, before making it unique.
const NEW_FORMATION_NAME: &str = "new_formation";

/// Offset of new relative blocks: behind their parent, as the most common offset in vanilla files.
const NEW_RELATIVE_OFFSET: (f32, f32) = (0.0, -5.0);

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the slots of a GroupFormations view.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct GroupFormationsSlots {
    filter_formations: QBox<SlotOfQString>,
    formation_selected: QBox<SlotNoArgs>,
    formations_context_menu: QBox<SlotOfQPoint>,
    add_formation: QBox<SlotNoArgs>,
    clone_formation: QBox<SlotNoArgs>,
    delete_formation: QBox<SlotNoArgs>,

    blocks_selected: QBox<SlotNoArgs>,
    blocks_context_menu: QBox<SlotOfQPoint>,
    add_absolute: QBox<SlotNoArgs>,
    add_relative: QBox<SlotNoArgs>,
    add_span: QBox<SlotNoArgs>,
    delete_block: QBox<SlotNoArgs>,
    delete_subtree: QBox<SlotNoArgs>,
    undo: QBox<SlotNoArgs>,
    redo: QBox<SlotNoArgs>,

    formation_fields_changed: QBox<SlotNoArgs>,
    ai_purpose_changed: QBox<SlotNoArgs>,
    min_unit_category_changed: QBox<SlotNoArgs>,
    add_min_unit_category: QBox<SlotNoArgs>,
    remove_min_unit_category: QBox<SlotNoArgs>,
    subcultures_changed: QBox<SlotNoArgs>,
    add_subculture: QBox<SlotNoArgs>,
    remove_subculture: QBox<SlotNoArgs>,
    factions_changed: QBox<SlotNoArgs>,
    add_faction: QBox<SlotNoArgs>,
    remove_faction: QBox<SlotNoArgs>,

    parent_changed: QBox<SlotNoArgs>,
    container_fields_changed: QBox<SlotNoArgs>,
    entity_preferences_changed: QBox<SlotNoArgs>,
    add_entity_preference: QBox<SlotNoArgs>,
    remove_entity_preference: QBox<SlotNoArgs>,
    span_members_changed: QBox<SlotNoArgs>,

    canvas_selection_changed: QBox<SlotNoArgs>,
    unit_count_changed: QBox<SlotNoArgs>,
    fit_canvas: QBox<SlotNoArgs>,

    issue_clicked: QBox<SlotOfQModelIndex>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl GroupFormationsSlots {
    pub unsafe fn new(view: &Arc<GroupFormationsView>, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Self {
        let parent = view.blocks_tree_view();

        //-----------------------------------------------//
        // Formations list.
        //-----------------------------------------------//

        let filter_formations = SlotOfQString::new(parent, clone!(
            view => move |text| {
                view.formations_list_filter().set_filter_fixed_string(text);
            }
        ));

        let formation_selected = SlotNoArgs::new(parent, clone!(
            view => move || {
                view.load_formation();
            }
        ));

        let formations_context_menu = SlotOfQPoint::new(parent, clone!(
            view => move |_| {
                view.formations_context_menu().exec_1a_mut(&QCursor::pos_0a());
            }
        ));

        let add_formation = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let name = view.unique_formation_name(NEW_FORMATION_NAME);
                let result = view.edit(&app_ui, &pack_file_contents_ui, |data| {
                    let mut formation = GroupFormation::new(&name, *view.format());
                    formation.add_container(None, (0.0, 0.0), *view.format())?;
                    data.formations_mut().push(formation);
                    Ok(())
                });

                if report(&view, result) {
                    let last_index = view.data().read().unwrap().formations().len() - 1;
                    view.load_formations_list(Some(last_index));
                }
            }
        ));

        let clone_formation = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let Some(formation_index) = view.selected_formation() else { return };
                let name = view.data().read().unwrap().formations().get(formation_index).map(|formation| format!("{}_copy", formation.name()));
                let Some(name) = name.map(|name| view.unique_formation_name(&name)) else { return };

                let result = view.edit(&app_ui, &pack_file_contents_ui, |data| {
                    let mut formation = data.formations()[formation_index].clone();
                    formation.set_name(name);
                    data.formations_mut().insert(formation_index + 1, formation);
                    Ok(())
                });

                if report(&view, result) {
                    view.load_formations_list(Some(formation_index + 1));
                }
            }
        ));

        let delete_formation = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let Some(formation_index) = view.selected_formation() else { return };
                let result = view.edit(&app_ui, &pack_file_contents_ui, |data| {
                    data.formations_mut().remove(formation_index);
                    Ok(())
                });

                if report(&view, result) {
                    view.load_formations_list(Some(formation_index));
                }
            }
        ));

        //-----------------------------------------------//
        // Blocks tree.
        //-----------------------------------------------//

        let blocks_selected = SlotNoArgs::new(parent, clone!(
            view => move || {
                group_formation_canvas_set_selected_ids_safe(view.canvas(), &view.selected_blocks());
                view.load_inspector();
                view.update_actions();
            }
        ));

        let canvas_selection_changed = SlotNoArgs::new(parent, clone!(
            view => move || {
                view.select_tree_blocks(&group_formation_canvas_selected_ids_safe(view.canvas()));
                view.load_inspector();
                view.update_actions();
            }
        ));

        let unit_count_changed = SlotNoArgs::new(parent, clone!(
            view => move || {
                let selected = view.selected_blocks();
                view.load_canvas();
                group_formation_canvas_set_selected_ids_safe(view.canvas(), &selected);
            }
        ));

        let fit_canvas = SlotNoArgs::new(parent, clone!(
            view => move || {
                group_formation_canvas_fit_safe(view.canvas());
            }
        ));

        let blocks_context_menu = SlotOfQPoint::new(parent, clone!(
            view => move |_| {
                view.blocks_context_menu().exec_1a_mut(&QCursor::pos_0a());
            }
        ));

        let add_absolute = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let mut new_block_id = 0;
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    new_block_id = formation.add_container(None, (0.0, 0.0), *view.format())?;
                    Ok(())
                });

                if report(&view, result) {
                    view.refresh_after_edit(Some(&[new_block_id]), true);
                }
            }
        ));

        let add_relative = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let [parent_id] = view.selected_blocks()[..] else { return };
                let mut new_block_id = 0;
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    new_block_id = formation.add_container(Some(parent_id), NEW_RELATIVE_OFFSET, *view.format())?;
                    Ok(())
                });

                if report(&view, result) {
                    view.refresh_after_edit(Some(&[new_block_id]), true);
                }
            }
        ));

        let add_span = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let members = view.selected_blocks();
                if members.is_empty() {
                    return;
                }

                let mut new_block_id = 0;
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    new_block_id = formation.add_span(&members)?;
                    Ok(())
                });

                if report(&view, result) {
                    view.refresh_after_edit(Some(&[new_block_id]), true);
                }
            }
        ));

        let delete_block = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let blocks = view.selected_blocks();
                if blocks.is_empty() {
                    return;
                }

                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    formation.delete_blocks(&blocks, &view.layout_params())?;
                    Ok(())
                });

                if report(&view, result) {
                    view.refresh_after_edit(Some(&[]), true);
                }
            }
        ));

        let delete_subtree = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let [block_id] = view.selected_blocks()[..] else { return };
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    formation.delete_subtree(block_id, &view.layout_params())?;
                    Ok(())
                });

                if report(&view, result) {
                    view.refresh_after_edit(Some(&[]), true);
                }
            }
        ));

        let undo = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.undo_edit(&app_ui, &pack_file_contents_ui);
            }
        ));

        let redo = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.redo_edit(&app_ui, &pack_file_contents_ui);
            }
        ));

        //-----------------------------------------------//
        // Formation inspector.
        //-----------------------------------------------//

        let formation_fields_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                if view.is_loading() {
                    return;
                }

                let name = view.name_line_edit().text().to_std_string();
                let ai_priority = view.ai_priority_spinbox().value() as f32;
                let uk_2 = view.uk_2_spinbox().value().max(0) as u32;
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    formation.set_name(name);
                    formation.set_ai_priority(ai_priority);
                    formation.set_uk_2(uk_2);
                    Ok(())
                });

                if report(&view, result) {
                    if let Some(formation_index) = view.selected_formation() {
                        view.update_formation_name(formation_index);
                    }
                    view.refresh_issues();
                }
            }
        ));

        let ai_purpose_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                if view.is_loading() {
                    return;
                }

                let bits = view.checked_ai_purpose_bits();
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    formation.set_ai_purpose(formation.ai_purpose().with_bits(bits));
                    Ok(())
                });
                report(&view, result);
            }
        ));

        let min_unit_category_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.sync_min_unit_category(&app_ui, &pack_file_contents_ui);
            }
        ));

        let add_min_unit_category = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.loading(|| view.min_unit_category().append_row(&[
                    QVariant::from_q_string(&QString::from_std_str(UnitCategory::default().to_string())),
                    QVariant::from_int(0),
                ]));
                view.sync_min_unit_category(&app_ui, &pack_file_contents_ui);
            }
        ));

        let remove_min_unit_category = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.loading(|| view.min_unit_category().remove_selected_rows());
                view.sync_min_unit_category(&app_ui, &pack_file_contents_ui);
            }
        ));

        let subcultures_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.sync_subcultures(&app_ui, &pack_file_contents_ui);
            }
        ));

        let add_subculture = SlotNoArgs::new(parent, clone!(
            view => move || {
                add_string_row(&view, view.subcultures());
            }
        ));

        let remove_subculture = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.loading(|| view.subcultures().remove_selected_rows());
                view.sync_subcultures(&app_ui, &pack_file_contents_ui);
            }
        ));

        let factions_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.sync_factions(&app_ui, &pack_file_contents_ui);
            }
        ));

        let add_faction = SlotNoArgs::new(parent, clone!(
            view => move || {
                add_string_row(&view, view.factions());
            }
        ));

        let remove_faction = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.loading(|| view.factions().remove_selected_rows());
                view.sync_factions(&app_ui, &pack_file_contents_ui);
            }
        ));

        //-----------------------------------------------//
        // Block inspector.
        //-----------------------------------------------//

        let parent_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                if view.is_loading() {
                    return;
                }

                let [block_id] = view.selected_blocks()[..] else { return };
                let parent_id = view.parent_combobox().current_data_0a().to_int_0a();
                let parent_id = u32::try_from(parent_id).ok();
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    formation.reparent(block_id, parent_id, &view.layout_params())?;
                    Ok(())
                });

                report(&view, result);
                view.refresh_after_edit(None, true);
            }
        ));

        let container_fields_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                if view.is_loading() {
                    return;
                }

                let position_x = view.position_x_spinbox().value() as f32;
                let position_y = view.position_y_spinbox().value() as f32;
                let priority = view.block_priority_spinbox().value() as f32;
                let arrangement = EntityArrangement::ALL.get(view.arrangement_combobox().current_index().max(0) as usize).copied().unwrap_or_default();
                let spacing = view.spacing_spinbox().value() as f32;
                let crescent_y_offset = view.crescent_y_offset_spinbox().value() as f32;
                let minimum = view.minimum_threshold_spinbox().value();
                let maximum = view.maximum_threshold_spinbox().value();

                let result = view.edit_block(&app_ui, &pack_file_contents_ui, |block| {
                    with_container!(block, container => {
                        container.set_position_x(position_x);
                        container.set_position_y(position_y);
                        container.set_block_priority(priority);
                        container.set_entity_arrangement(arrangement);
                        container.set_inter_entity_spacing(spacing);
                        container.set_crescent_y_offset(crescent_y_offset);
                        container.set_minimum_entity_threshold(minimum);
                        container.set_maximum_entity_threshold(maximum);
                    });
                });

                if report(&view, result) {
                    view.refresh_after_edit(None, false);
                }
            }
        ));

        let entity_preferences_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.sync_entity_preferences(&app_ui, &pack_file_contents_ui);
            }
        ));

        let add_entity_preference = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                let preference = EntityPreference::new(*view.format());
                view.loading(|| view.entity_preferences().append_row(&[
                    QVariant::from_double(*preference.priority() as f64),
                    QVariant::from_q_string(&QString::from_std_str(preference.entity().to_string())),
                    QVariant::from_q_string(&QString::from_std_str(preference.entity_weight().to_string())),
                    QVariant::from_int(*preference.uk_1() as i32),
                    QVariant::from_q_string(&QString::from_std_str(preference.entity_class())),
                ]));
                view.sync_entity_preferences(&app_ui, &pack_file_contents_ui);
            }
        ));

        let remove_entity_preference = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                view.loading(|| view.entity_preferences().remove_selected_rows());
                view.sync_entity_preferences(&app_ui, &pack_file_contents_ui);
            }
        ));

        let span_members_changed = SlotNoArgs::new(parent, clone!(
            app_ui,
            pack_file_contents_ui,
            view => move || {
                if view.is_loading() {
                    return;
                }

                let [span_id] = view.selected_blocks()[..] else { return };
                let members = view.checked_span_members();
                let result = view.edit_formation(&app_ui, &pack_file_contents_ui, |formation| {
                    formation.set_span_members(span_id, &members)?;
                    Ok(())
                });

                // The inspector isn't refilled, as that would rebuild the list while it's emitting its change.
                if report(&view, result) {
                    view.refresh_after_edit(None, false);
                }
            }
        ));

        //-----------------------------------------------//
        // Issues.
        //-----------------------------------------------//

        let issue_clicked = SlotOfQModelIndex::new(parent, clone!(
            view => move |index| {
                let formation_index = index.data_1a(ID_ROLE).to_u_int_0a() as usize;
                let block_id = index.data_1a(ISSUE_BLOCK_ROLE);

                if view.selected_formation() != Some(formation_index) {
                    view.select_formation(formation_index);
                }

                if block_id.is_valid() {
                    view.select_blocks(&[block_id.to_u_int_0a()]);
                } else {
                    view.select_blocks(&[]);
                }
            }
        ));

        Self {
            filter_formations,
            formation_selected,
            formations_context_menu,
            add_formation,
            clone_formation,
            delete_formation,

            blocks_selected,
            blocks_context_menu,
            add_absolute,
            add_relative,
            add_span,
            delete_block,
            delete_subtree,
            undo,
            redo,

            formation_fields_changed,
            ai_purpose_changed,
            min_unit_category_changed,
            add_min_unit_category,
            remove_min_unit_category,
            subcultures_changed,
            add_subculture,
            remove_subculture,
            factions_changed,
            add_faction,
            remove_faction,

            parent_changed,
            container_fields_changed,
            entity_preferences_changed,
            add_entity_preference,
            remove_entity_preference,
            span_members_changed,

            canvas_selection_changed,
            unit_count_changed,
            fit_canvas,

            issue_clicked,
        }
    }
}

impl GroupFormationsView {

    /// Saves the minimum unit category percentages of the inspector into the selected formation.
    unsafe fn sync_min_unit_category(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if self.is_loading() {
            return;
        }

        let rows = self.min_unit_category_rows();
        let result = self.edit_formation(app_ui, pack_file_contents_ui, |formation| {
            formation.set_min_unit_category_percentage(rows);
            Ok(())
        });
        report(self, result);
    }

    /// Saves the supported subcultures of the inspector into the selected formation.
    unsafe fn sync_subcultures(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if self.is_loading() {
            return;
        }

        let subcultures = self.subcultures().strings();
        let result = self.edit_formation(app_ui, pack_file_contents_ui, |formation| {
            formation.set_ai_supported_subcultures(subcultures);
            Ok(())
        });
        report(self, result);
    }

    /// Saves the supported factions of the inspector into the selected formation.
    unsafe fn sync_factions(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if self.is_loading() {
            return;
        }

        let factions = self.factions().strings();
        let result = self.edit_formation(app_ui, pack_file_contents_ui, |formation| {
            formation.set_ai_supported_factions(factions);
            Ok(())
        });
        report(self, result);
    }

    /// Saves the entity preferences of the inspector into the selected block.
    unsafe fn sync_entity_preferences(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) {
        if self.is_loading() {
            return;
        }

        let preferences = self.entity_preference_rows();
        let result = self.edit_block(app_ui, pack_file_contents_ui, |block| {
            with_container!(block, container => {
                container.set_entity_preferences(preferences);
            });
        });

        if report(self, result) {
            self.refresh_after_edit(None, false);
        }
    }
}

/// Shows the error of a failed edit, if any.
///
/// # Returns
///
/// `true` if the edit succeeded.
unsafe fn report(view: &GroupFormationsView, result: Result<()>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => {
            show_dialog(view.blocks_tree_view(), error, false);
            false
        },
    }
}

/// Appends an empty row to a list of strings, and starts editing it.
///
/// The row is only saved once it has text, when the edit triggers the list's changed slot.
unsafe fn add_string_row(view: &GroupFormationsView, list: &ItemList) {
    view.loading(|| list.append_row(&[QVariant::from_q_string(&QString::new())]));
    let index: cpp_core::CppBox<QModelIndex> = list.model().index_2a(list.model().row_count_0a() - 1, 0);
    list.view().set_current_index(&index);
    list.view().edit(&index);
}

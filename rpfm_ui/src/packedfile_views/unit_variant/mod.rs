//---------------------------------------------------------------------------//
// Copyright (c) 2017-2020 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the UnitVariant Component Views.
!*/

use std::sync::Arc;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::unit_variant::UnitVariant;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::views::debug::DebugView;

use crate::packedfile_views::PackedFileView;

use super::{ViewType, View};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the PackFile Settings.
pub struct PackedFileUnitVariantView {
    debug_view: Arc<DebugView>,
}

// This struct contains the view of the PackFile Settings.
//pub struct PackedFileUnitVariantView {
//    version_label: QBox<QLabel>,
//    unknown_1_label: QBox<QLabel>,
//    categories: Vec<CategoryEntry>,
//}
//
//pub struct CategoryEntry {
//    category_frame: QBox<QGroupBox>,
//    name_line_edit: QBox<QLineEdit>,
//    //id_spinbox: QBox<QSpinBox>,
//    //add: QBox<QPushButton>,
//    //remove: QBox<QPushButton>,
//    equipments: Vec<EquipmentEntry>,
//}
//
//pub struct EquipmentEntry {
//    equipments: (QBox<QLineEdit>, QBox<QLineEdit>),
//    //add: QBox<QPushButton>,
//    //remove: QBox<QPushButton>,
//}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileUnitVariantView`.
impl PackedFileUnitVariantView {

    /// This function creates a new PackedFileUnitVariantView, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
    ) -> Result<Option<PackedFileInfo>> {

        CENTRAL_COMMAND.send_message_qt(Command::DecodePackedFile(packed_file_view.get_path(), packed_file_view.get_data_source()));
        let response = CENTRAL_COMMAND.recv_message_qt();
        let (data, packed_file_info) = match response {
            Response::DecodedPackedFilePackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        /*
        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();

        let info_frame = QGroupBox::from_q_string_q_widget(&qtr("info_title"), packed_file_view.get_mut_widget());
        let info_layout = create_grid_layout(info_frame.static_upcast());

        let version_label = QLabel::from_q_string_q_widget(&QString::from_std_str(&format!("Version: {}", data.get_ref_version())), &info_frame);
        let unknown_1_label = QLabel::from_q_string_q_widget(&QString::from_std_str(&format!("Unknown value: {}", data.get_ref_unknown_1())), &info_frame);

        let scroll_area = QScrollArea::new_1a(packed_file_view.get_mut_widget());
        let categories_widget = QWidget::new_1a(&scroll_area);
        let categories_layout = create_grid_layout(categories_widget.static_upcast());
        scroll_area.set_widget(&categories_widget);
        scroll_area.set_widget_resizable(true);
        scroll_area.horizontal_scroll_bar().set_enabled(true);

        info_layout.add_widget_5a(&version_label, 0, 0, 1, 1);
        info_layout.add_widget_5a(&unknown_1_label, 0, 1, 1, 1);


        let mut categories = vec![];
        for (row, category) in data.get_ref_categories().iter().enumerate() {

            let category_frame = QGroupBox::from_q_string_q_widget(&qtre("category_title", &[&row.to_string()]), &categories_widget);
            let category_layout = create_grid_layout(category_frame.static_upcast());

            let name_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(category.get_ref_name()), &category_frame);
            let id_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(category.get_ref_id().to_string()), &category_frame);

            let equipment_frame = QGroupBox::from_q_string_q_widget(&qtre("equipment_title", &[&row.to_string()]), &category_frame);
            let equipment_layout = create_grid_layout(equipment_frame.static_upcast());

            let mut equipments = vec![];
            for (row2, equipment_list) in category.get_ref_equipments().iter().enumerate() {
                let equipment_1_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(&equipment_list.0), &equipment_frame);
                let equipment_2_line_edit = QLineEdit::from_q_string_q_widget(&QString::from_std_str(&equipment_list.1), &equipment_frame);

                equipment_layout.add_widget_5a(&equipment_1_line_edit, row2 as i32 + 1, 0, 1, 1);
                equipment_layout.add_widget_5a(&equipment_2_line_edit, row2 as i32 + 1, 1, 1, 1);

                equipments.push(EquipmentEntry {
                    equipments: (equipment_1_line_edit, equipment_2_line_edit),
                });
            }
            category_layout.add_widget_5a(&name_line_edit, 0, 0, 1, 1);
            category_layout.add_widget_5a(&id_line_edit, 0, 1, 1, 1);
            category_layout.add_widget_5a(&equipment_frame, 1, 0, 1, 2);

            categories_layout.add_widget_5a(&category_frame, row as i32 + 1, 0, 1, 1);
        layout.add_widget_5a(&info_frame, 0, 0, 1, 1);
        layout.add_widget_5a(&scroll_area, 1, 0, 1, 1);

            let category_entry = CategoryEntry {
                category_frame,
                name_line_edit,
                //id_spinbox,
                //add,
                //remove,
                equipments,
            };

            categories.push(category_entry);
        }


        let view = Arc::new(Self {
            categories,
            version_label,
            unknown_1_label,
        });

        //let pack_file_settings_slots = PackFileSettingsSlots::new(
        //    &pack_file_settings_view,
        //    app_ui,
        //    pack_file_contents_ui
        //);

        //connections::set_connections(&pack_file_settings_view, &pack_file_settings_slots);
*/
        // For now just build a debug view.
        let debug_view = DebugView::new_view(
            packed_file_view.get_mut_widget(),
            data,
            packed_file_view.get_path_raw(),
        )?;

        let packed_file_debug_view = Self {
            debug_view,
        };

        packed_file_view.view = ViewType::Internal(View::UnitVariant(Arc::new(packed_file_debug_view)));
        packed_file_view.packed_file_type = PackedFileType::UnitVariant;

        Ok(Some(packed_file_info))
    }

    // This function saves a PackFileSettingsView into a PackFileSetting.
    //pub unsafe fn save_view(&self) -> UnitVariant {
    //    let unit_variant = UnitVariant::default();
    //
    //    unit_variant
    //}

    /// This function tries to reload the current view with the provided data.
    pub unsafe fn reload_view(&self, data: &UnitVariant) {
        let text = serde_json::to_string_pretty(&data).unwrap();
        self.debug_view.reload_view(&text);
    }
}

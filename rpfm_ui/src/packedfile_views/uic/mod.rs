//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the UI Component Views.
!*/

use qt_widgets::QGridLayout;
use qt_widgets::QGraphicsView;
use qt_widgets::QGraphicsScene;
use qt_widgets::QGraphicsItem;
use qt_widgets::QLabel;
use qt_widgets::QWidget;
use qt_widgets::q_graphics_view::DragMode;

use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QPtr;

use std::rc::Rc;
use std::sync::Arc;

use rpfm_error::{ErrorKind, Result};

use rpfm_lib::packedfile::PackedFileType;
use rpfm_lib::packedfile::uic::UIC;
use rpfm_lib::packfile::packedfile::PackedFileInfo;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::locale::qtr;
use crate::packedfile_views::{PackedFileView, PackFileContentsUI};
use crate::utils::create_grid_layout;
use super::{ViewType, View};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of the PackFile Settings.
pub struct PackedFileUICView {
    viewer: QBox<QGraphicsView>,
    scene: QBox<QGraphicsScene>,
    properties: QBox<QWidget>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `PackedFileUICView`.
impl PackedFileUICView {

    /// This function creates a new PackedFileUICView, and sets up his slots and connections.
    pub unsafe fn new_view(
        packed_file_view: &mut PackedFileView,
        _app_ui: &Rc<AppUI>,
        _pack_file_contents_ui: &Rc<PackFileContentsUI>
    ) -> Result<Option<PackedFileInfo>> {

        let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(packed_file_view.get_path()));
        let response = CentralCommand::recv(&receiver);
        let (data, packed_file_info) = match response {
            Response::UICPackedFileInfo((data, packed_file_info)) => (data, packed_file_info),
            Response::Error(error) => return Err(error),
            Response::Unknown => return Err(ErrorKind::PackedFileTypeUnknown.into()),
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
        let scene = QGraphicsScene::from_q_object(packed_file_view.get_mut_widget());
        let viewer = QGraphicsView::from_q_widget(packed_file_view.get_mut_widget());
        viewer.set_scene(&scene);
        viewer.set_drag_mode(DragMode::ScrollHandDrag);

        let test_item = scene.add_text_1a(&qt_core::QString::from_std_str(&format!("{:?}", data)));
        let flags = QFlags::from(qt_widgets::q_graphics_item::GraphicsItemFlag::ItemIsMovable.to_int());
        test_item.as_ptr().static_upcast::<QGraphicsItem>().set_flags(flags);

        let properties = QWidget::new_1a(packed_file_view.get_mut_widget());
        let properties_layout = create_grid_layout(properties.static_upcast());

        let test_label = QLabel::from_q_string_q_widget(&qtr("format"), &properties);
        properties_layout.add_widget_5a(&test_label, 0, 0, 1, 1);

        layout.add_widget_5a(&viewer, 0, 0, 1, 1);
        layout.add_widget_5a(&properties, 0, 1, 1, 1);

        let view = Arc::new(Self {
            scene,
            viewer,
            properties,
        });

        //let pack_file_settings_slots = PackFileSettingsSlots::new(
        //    &pack_file_settings_view,
        //    app_ui,
        //    pack_file_contents_ui
        //);

        //connections::set_connections(&pack_file_settings_view, &pack_file_settings_slots);
        packed_file_view.packed_file_type = PackedFileType::UIC;
        packed_file_view.view = ViewType::Internal(View::UIC(view));
        Ok(Some(packed_file_info))
    }

    /// This function saves a PackFileSettingsView into a PackFileSetting.
    pub unsafe fn save_view(&self) -> UIC {
        let uic = UIC::default();

        uic
    }
}

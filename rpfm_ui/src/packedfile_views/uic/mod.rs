//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
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

use anyhow::Result;
use getset::Getters;

use std::sync::Arc;

use rpfm_lib::files::{FileType, uic::UIC};

use rpfm_ui_common::locale::qtr;

use crate::packedfile_views::FileView;
use crate::utils::*;
use super::{ViewType, View};

const VIEW_DEBUG: &str = "rpfm_ui/ui_templates/uic_view.ui";
const VIEW_RELEASE: &str = "ui/uic_view.ui";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

#[derive(Getters)]
#[getset(get = "pub")]
pub struct FileUICView {
    viewer: QPtr<QGraphicsView>,
    scene: QBox<QGraphicsScene>,
    properties: QPtr<QWidget>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl FileUICView {

    /// This function creates a new FileUICView, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        data: &UIC,
    ) -> Result<()> {

        let template_path = if cfg!(debug_assertions) { VIEW_DEBUG } else { VIEW_RELEASE };
        let main_widget = load_template(file_view.main_widget(), template_path)?;
        let viewer: QPtr<QGraphicsView> = find_widget(&main_widget.static_upcast(), "viewer_graphics_view")?;
        let properties: QPtr<QWidget> = find_widget(&main_widget.static_upcast(), "properties_widget")?;
        let scene = QGraphicsScene::from_q_object(&main_widget);
        viewer.set_scene(&scene);
        viewer.set_drag_mode(DragMode::ScrollHandDrag);

        let test_item = scene.add_text_1a(&qt_core::QString::from_std_str(format!("{data:?}")));
        let flags = QFlags::from(qt_widgets::q_graphics_item::GraphicsItemFlag::ItemIsMovable.to_int());
        test_item.as_ptr().static_upcast::<QGraphicsItem>().set_flags(flags);

        let properties_layout: QPtr<QGridLayout> = properties.layout().static_downcast();
        let test_label = QLabel::from_q_string_q_widget(&qtr("format"), &properties);
        properties_layout.add_widget_5a(&test_label, 0, 0, 1, 1);

        let view = Arc::new(Self {
            scene,
            viewer,
            properties,
        });

        file_view.file_type = FileType::UIC;
        file_view.view_type = ViewType::Internal(View::UIC(view));
        Ok(())
    }

    pub unsafe fn save_view(&self) -> UIC {
        let uic = UIC::default();

        uic
    }
}

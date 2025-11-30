//---------------------------------------------------------------------------//
// Copyright (c) 2017-2024 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code for VMD files.

use qt_widgets::QGridLayout;
#[cfg(feature = "support_model_renderer")] use qt_widgets::QPushButton;
use qt_widgets::QSplitter;
use qt_widgets::QWidget;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;

#[cfg(feature = "support_model_renderer")] use anyhow::Result;
use getset::Getters;
#[cfg(feature = "support_model_renderer")]use rpfm_ui_common::settings::setting_bool;
#[cfg(feature = "support_model_renderer")]use rpfm_ui_common::utils::show_dialog;


use std::rc::Rc;
use std::sync::{Arc, RwLock};

use rpfm_lib::files::{FileType, text::*};
#[cfg(feature = "support_model_renderer")]use rpfm_ui_common::locale::qtr;
use rpfm_ui_common::utils::create_grid_layout;

use crate::app_ui::AppUI;
use crate::ffi::*;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::{DataSource, FileView, View, ViewType};
use crate::packedfile_views::vmd::slots::FileVMDViewSlots;

mod connections;
mod slots;

const XML: &str = "XML";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the view of a VMD PackedFile.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct FileVMDView {
    path: Option<Arc<RwLock<String>>>,
    data_source: Arc<RwLock<DataSource>>,

    editor: QBox<QWidget>,
    #[cfg(feature = "support_model_renderer")] reload_button: QBox<QPushButton>,
    #[cfg(feature = "support_model_renderer")] renderer_enabled: bool,
    #[cfg(feature = "support_model_renderer")] renderer: QBox<QWidget>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation for `FileVMDView`.
impl FileVMDView {

    /// This function creates a new Text View, and sets up his slots and connections.
    pub unsafe fn new_view(
        file_view: &mut FileView,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        data: &Text,
        file_type: FileType,
    ) {
        let highlighting_mode = QString::from_std_str(XML);

        let layout: QPtr<QGridLayout> = file_view.main_widget().layout().static_downcast();
        let splitter = QSplitter::from_q_widget(file_view.main_widget());
        layout.add_widget_5a(&splitter, 0, 0, 1, 1);

        let left_widget = QWidget::new_1a(&splitter);
        let left_layout = create_grid_layout(left_widget.static_upcast());
        left_widget.size_policy().set_horizontal_stretch(1);
        left_widget.set_minimum_width(600);
        splitter.add_widget(&left_widget);

        #[cfg(feature = "support_model_renderer")] let mut renderer_enabled = false;
        let view = Arc::new(FileVMDView {
            path: Some(file_view.path_raw()),
            data_source: file_view.data_source.clone(),

            editor: {
                let editor = new_text_editor_safe(&file_view.main_widget().static_upcast());
                set_text_safe(&editor.static_upcast(), &QString::from_std_str(data.contents()).as_ptr(), &highlighting_mode.as_ptr());

                left_layout.add_widget_5a(&editor, 0, 0, 1, 1);
                editor
            },
            #[cfg(feature = "support_model_renderer")] reload_button: {
                let button = QPushButton::from_q_string_q_widget(&qtr("reload_renderer"), file_view.main_widget());
                left_layout.add_widget_5a(&button, 1, 0, 1, 1);

                button
            },
            #[cfg(feature = "support_model_renderer")] renderer: {
                if SETTINGS.read().unwrap().bool("enable_renderer") {
                    match create_q_rendering_widget(&mut file_view.main_widget().as_ptr()) {
                        Ok(renderer) => {

                            // We need to manually pause the renderer or it'll keep lagging the UI.
                            if let Err(error) = add_new_primary_asset(&renderer.as_ptr(), &file_view.path().read().unwrap(), data.contents().as_bytes()) {
                                show_dialog(file_view.main_widget(), error, false);
                                pause_rendering(&renderer.as_ptr());
                            }

                            renderer_enabled = true;
                            renderer.size_policy().set_horizontal_stretch(1);
                            splitter.add_widget(&renderer);
                            renderer
                        }
                        Err(error) => {
                            show_dialog(file_view.main_widget(), error, false);
                            QWidget::new_1a(file_view.main_widget())
                        }
                    }
                } else {
                    QWidget::new_1a(file_view.main_widget())
                }
            },

            #[cfg(feature = "support_model_renderer")] renderer_enabled,
        });

        let slots = FileVMDViewSlots::new(&view, app_ui, pack_file_contents_ui);
        connections::set_connections(&view, &slots);

        match file_type {
            FileType::VMD => {
                file_view.file_type = FileType::VMD;
                file_view.view_type = ViewType::Internal(View::VMD(view));
            },
            FileType::WSModel => {
                file_view.file_type = FileType::WSModel;
                file_view.view_type = ViewType::Internal(View::WSModel(view));
            }
            _ => panic!("In theory should never happen.")
        }
    }

    /// This function returns a pointer to the editor widget.
    pub fn get_mut_editor(&self) -> &QBox<QWidget> {
        &self.editor
    }

    #[cfg(feature = "support_model_renderer")]
    pub unsafe fn reload_render(&self) -> Result<()> {
        if self.renderer_enabled {
            add_new_primary_asset(&self. renderer.as_ptr(), &self.path.clone().unwrap().read().unwrap(), get_text_safe(&self.editor).to_std_string().as_bytes())
        } else {
            Ok(())
        }
    }

    /// Function to reload the data of the view without having to delete the view itself.
    pub unsafe fn reload_view(&self, data: &Text) {

        let highlighting_mode = QString::from_std_str(XML);

        let row_number = cursor_row_safe(&self.editor.as_ptr());
        set_text_safe(&self.editor.static_upcast(), &QString::from_std_str(data.contents()).as_ptr(), &highlighting_mode.as_ptr());

        // Try to scroll to the line we were before.
        scroll_to_row_safe(&self.editor.as_ptr(), row_number);
    }
}

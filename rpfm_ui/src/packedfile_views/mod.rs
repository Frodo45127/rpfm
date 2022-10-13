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
Module with all the submodules for controlling the views of each decodeable PackedFile Type.

This module contains the code to manage the views and actions of each decodeable PackedFile View.
!*/

use qt_widgets::QWidget;

use qt_core::QBox;

use anyhow::{anyhow, Result};

use std::{fmt, fmt::Display};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use rpfm_lib::files::{RFileDecoded, FileType};
use rpfm_lib::files::{anims_table::AnimsTable, db::DB, loc::Loc, matched_combat::MatchedCombat, text::Text};

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::ffi::get_text_safe;
use crate::pack_tree::*;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::utils::create_grid_layout;
use crate::utils::show_dialog;
use crate::UI_STATE;
use crate::views::table::utils::get_table_from_view;
use crate::views::table::TableType;

use self::anim_fragment::{PackedFileAnimFragmentView, PackedFileAnimFragmentDebugView};
use self::animpack::PackedFileAnimPackView;
use self::esf::PackedFileESFView;
//use self::decoder::PackedFileDecoderView;
//use self::dependencies_manager::DependenciesManagerView;
use self::external::PackedFileExternalView;
use self::image::PackedFileImageView;
use self::table::PackedFileTableView;
use self::text::PackedFileTextView;
//use self::packfile::PackFileExtraView;
use self::packfile_settings::PackFileSettingsView;
//use self::tips::TipsView;
use self::video::PackedFileVideoView;

#[cfg(feature = "support_rigidmodel")]
use self::rigidmodel::PackedFileRigidModelView;

#[cfg(feature = "support_uic")]
use self::uic::PackedFileUICView;
use self::unit_variant::PackedFileUnitVariantView;

pub mod anim_fragment;
pub mod animpack;
//pub mod decoder;
//pub mod dependencies_manager;
pub mod esf;
pub mod external;
pub mod image;
//pub mod packfile;
pub mod packfile_settings;

#[cfg(feature = "support_rigidmodel")]
pub mod rigidmodel;
pub mod table;
pub mod text;
//pub mod tips;

#[cfg(feature = "support_uic")]
pub mod uic;
pub mod unit_variant;
pub mod utils;
pub mod video;

const RFILE_SAVED_ERROR: &str = "The following PackedFile failed to be saved: ";
const RFILE_RELOAD_ERROR: &str = "The PackedFile you added is not the same type as the one you had before. So… the view showing it will get closed.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the widget of the view of a PackedFile and his info.
pub struct PackedFileView {
    path: Arc<RwLock<String>>,
    main_widget: Arc<QBox<QWidget>>,
    tips_widget: Arc<QBox<QWidget>>,
    //tips_view: Arc<TipsView>,
    is_preview: AtomicBool,
    is_read_only: AtomicBool,
    data_source: Arc<RwLock<DataSource>>,
    view: ViewType,
    packed_file_type: FileType,
}

/// This enum represents the type of the view of a PackFile.
pub enum ViewType {

    /// This type means we have a normal view within RPFM.
    Internal(View),

    // This means the PackFile has been saved to a file on disk, so no internal view is shown.
    External(Arc<PackedFileExternalView>)
}

/// This enum represents the source of the data in the view.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd)]
pub enum DataSource {

    /// This means the data is from somewhere in our PackFile.
    PackFile,

    /// This means the data is from one of the game files.
    GameFiles,

    /// This means the data comes from a parent PackFile.
    ParentFiles,

    /// This means the data comes from the AssKit files.
    AssKitFiles,

    /// This means the data comes from an external file.
    ExternalFile,
}

/// This enum is used to hold in a common way all the view types we have.
pub enum View {
    AnimFragment(Arc<PackedFileAnimFragmentView>),
    AnimFragmentDebug(Arc<PackedFileAnimFragmentDebugView>),
    AnimPack(Arc<PackedFileAnimPackView>),
    /*
    Decoder(Arc<PackedFileDecoderView>),
    DependenciesManager(Arc<DependenciesManagerView>),
    */
    ESF(Arc<PackedFileESFView>),
    Image(PackedFileImageView),
    //PackFile(Arc<PackFileExtraView>),
    PackSettings(Arc<PackFileSettingsView>),

    #[cfg(feature = "support_rigidmodel")]
    RigidModel(Arc<PackedFileRigidModelView>),
    Table(Arc<PackedFileTableView>),
    Text(Arc<PackedFileTextView>),
/*
    #[cfg(feature = "support_uic")]
    UIC(Arc<PackedFileUICView>),*/
    UnitVariant(Arc<PackedFileUnitVariantView>),
    Video(Arc<PackedFileVideoView>),
    None,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Default implementation for `PackedFileView`.
impl Default for PackedFileView {
    fn default() -> Self {
        let path = Arc::new(RwLock::new(String::new()));

        let main_widget_ptr = unsafe { QWidget::new_0a() };
        let main_layout = unsafe { create_grid_layout(main_widget_ptr.static_upcast()) };
        let main_widget = Arc::new(main_widget_ptr);

        let tips_widget_ptr = unsafe { QWidget::new_0a() };
        unsafe { create_grid_layout(tips_widget_ptr.static_upcast()); }
        unsafe { main_layout.add_widget_5a(&tips_widget_ptr, 0, 99, 1, 1); }
        let tips_widget = Arc::new(tips_widget_ptr);
        //let tips_view = unsafe { TipsView::new_view(&tips_widget, &[]) };

        // Hide it by default.
        unsafe { tips_widget.set_visible(false) };

        let is_preview = AtomicBool::new(false);
        let is_read_only = AtomicBool::new(false);
        let data_source = Arc::new(RwLock::new(DataSource::PackFile));
        let view = ViewType::Internal(View::None);
        let packed_file_type = FileType::Unknown;
        Self {
            path,
            main_widget,
            tips_widget,
            //tips_view,
            is_preview,
            is_read_only,
            data_source,
            view,
            packed_file_type,
        }
    }
}

/// Wacky fix for the "You cannot put a pointer in a static" problem.
unsafe impl Send for PackedFileView {}
unsafe impl Sync for PackedFileView {}

/// Implementation for `PackedFileView`.
impl PackedFileView {

    /// This function returns a copy of the path of this `PackedFileView`.
    pub fn get_path(&self) -> String {
        self.path.read().unwrap().to_owned()
    }

    /// This function returns a copy of the path of this `PackedFileView`.
    pub fn get_path_raw(&self) -> Arc<RwLock<String>> {
        self.path.clone()
    }

    /// This function returns a reference to the path of this `PackedFileView`.
    pub fn get_ref_path(&self) -> RwLockReadGuard<String> {
        self.path.read().unwrap()
    }

    /// This function allows you to set a `PackedFileView` as a preview or normal view.
    pub fn set_path(&self, path: &str) {
        *self.path.write().unwrap() = path.to_owned();
        //unsafe { self.tips_view.load_data(path) };
    }

    /// This function returns a mutable pointer to the `Widget` of the `PackedFileView`.
    pub fn get_mut_widget(&self) -> &QBox<QWidget> {
        &self.main_widget
    }

    /// This function returns a mutable pointer to the `Widget` of the `PackedFileView`.
    pub fn get_tips_widget(&self) -> &QBox<QWidget> {
        &self.tips_widget
    }

    /// This function returns if the `PackedFileView` is a preview or not.
    pub fn get_is_preview(&self) -> bool {
        self.is_preview.load(Ordering::SeqCst)
    }

    /// This function allows you to set a `PackedFileView` as a preview or normal view.
    pub fn set_is_preview(&self, is_preview: bool) {
        self.is_preview.store(is_preview, Ordering::SeqCst);
    }

    /// This function returns if the `PackedFileView` is read-only or not.
    pub fn get_is_read_only(&self) -> bool {
        self.is_read_only.load(Ordering::SeqCst)
    }

    /// This function allows you to set a `PackedFileView` as a read-only or normal view.
    pub fn set_is_read_only(&self, is_read_only: bool) {
        self.is_read_only.store(is_read_only, Ordering::SeqCst);
    }

    /// This function returns the DataSource of the specific `PackedFile`.
    pub fn get_data_source(&self) -> DataSource {
        self.data_source.read().unwrap().clone()
    }

    /// This function sets the DataSource of the specific `PackedFile`.
    pub fn set_data_source(&mut self, data_source: DataSource) {
        *self.data_source.write().unwrap() = data_source;
    }

    /// This function returns the ViewType of the specific `PackedFile`.
    pub fn get_view(&self) -> &ViewType {
        &self.view
    }

    /// This function returns a mutable reference to the ViewType of the specific `PackedFile`.
    pub fn get_ref_mut_view(&mut self) -> &mut ViewType {
        &mut self.view
    }

    /// This function returns a copy of the `FileType` of this view.
    pub fn get_packed_file_type(&self) -> FileType {
        self.packed_file_type
    }

    /// This function allows you to save a `PackedFileView` to his corresponding `PackedFile`.
    pub unsafe fn save(
        &self,
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
    ) -> Result<()> {

        // Only save non-read-only, local files.
        if let DataSource::PackFile = self.get_data_source() {
            if !self.get_is_read_only() {
                match self.get_view() {
                    ViewType::Internal(view) => {

                        // This is a two-step process. First, we take the data from the view into a `RFileDecoded` format.
                        // Then, we send that `RFileDecoded` to the backend to replace the older one. We need no response.
                        let data = match self.packed_file_type {
                            FileType::AnimsTable |
                            FileType::DB |
                            FileType::Loc |
                            FileType::MatchedCombat => if let View::Table(view) = view {

                                let new_table = get_table_from_view(&view.get_ref_table().table_model_ptr().static_upcast(), &view.get_ref_table().table_definition())?;
                                match self.packed_file_type {
                                    FileType::AnimsTable => {
                                        let table = AnimsTable::from(new_table);
                                        RFileDecoded::AnimsTable(table)
                                    }

                                    FileType::DB => {

                                        // If this crashes, it's a bug somewhere else.
                                        let table_name = view.get_ref_table().table_name().as_ref().unwrap();
                                        let table_uuid = view.get_ref_table().table_uuid().as_ref().map(|x| &**x);
                                        let mut table = DB::new(&view.get_ref_table().table_definition(), None, table_name, false);
                                        table.set_data(None, &new_table.data(&None)?)?;
                                        RFileDecoded::DB(table)
                                    }
                                    FileType::Loc => {
                                        let table = Loc::from(new_table);
                                        RFileDecoded::Loc(table)
                                    }
                                    FileType::MatchedCombat => {
                                        let table = MatchedCombat::from(new_table);
                                        RFileDecoded::MatchedCombat(table)
                                    }
                                    _ => return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path()))
                                }
                            } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) },

                            // Images are read-only.
                            FileType::Image => return Ok(()),

                            // AnimPacks save on edit.
                            FileType::AnimPack => return Ok(()),

                            FileType::AnimFragment => {
                                match view {
                                    View::AnimFragment(view) => view.save_data()?,
                                    View::AnimFragmentDebug(_) => return Ok(()),
                                    _ => return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())),
                                }
                            },

                            // These ones are a bit special. We just need to send back the current format of the video.
                            FileType::Video => {
                                if let View::Video(view) = view {
                                    let _ = CENTRAL_COMMAND.send_background(Command::SetVideoFormat(self.get_path(), view.get_current_format()));
                                    return Ok(())
                                } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) }
                            },

                            #[cfg(feature = "support_rigidmodel")]
                            FileType::RigidModel => {
                                if let View::RigidModel(view) = view {
                                    let data = view.save_view()?;
                                    RFileDecoded::RigidModel(data)
                                } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) }
                            }

                            FileType::Text => {
                                if let View::Text(view) = view {
                                    let mut text = Text::default();
                                    let widget = view.get_mut_editor();
                                    let string = get_text_safe(widget).to_std_string();
                                    text.set_contents(string);
                                    RFileDecoded::Text(text)
                                } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) }
                            },
                            /*
                            // These ones are like very reduced tables.
                            FileType::DependencyPackFilesList => if let View::DependenciesManager(view) = view {
                                let mut entries = vec![];
                                let model = view.get_ref_table().get_mut_ptr_table_model();
                                if model.is_null() {
                                    log::warn!("
                                        model null on active view!!! WTF?
                                        this basically means RPFM it's going to crash once this if is over.
                                        to avoid crashing, we'll skip this save step.
                                        also, log whatever is in that active view.
                                        because I have no idea how it managed to end up like that:
                                        - is_preview? {}
                                        - is_read_only? {}
                                        - is_the_qbox_seriously_null? {}
                                        - is_the_table_also_null? {}
                                        ",
                                        self.get_is_preview(),
                                        self.get_is_read_only(),
                                        view.get_ref_table().get_ref_table_model().is_null(),
                                        view.get_ref_table().get_mut_ptr_table_view_primary().is_null()
                                    );

                                    show_dialog(&app_ui.main_window,
                                        "Congratulations! You hit a rare bug I'm trying to fix!

                                        First, if there's an update for RPFM, update, as this may have been fixed already in an update.

                                        If not, welcome to hell. RPFM will try to work around the bug and not crash, but if you edited the dependencies with the dependencies manager,
                                        your changes to it may (may or may not) have been lost. Better save, then reopen the PackFile (to avoid further issues),
                                        and open the Dependencies manager again and check if your changes are still there.

                                        Also, RPFM has logged a bit of data that may help pinpoint why this is actually happen in a rpfm.log file you can access going to
                                        Game Selected/Open RPFM Config folder. If you don't mind, share it with the dev, and if you can, specify the steps you took before this appeared,
                                        specially those related to the

                                        ", false);
                                } else {
                                    for row in 0..model.row_count_0a() {
                                        let item = model.item_1a(row as i32).text().to_std_string();
                                        entries.push(item);
                                    }

                                    // Save the new list and return Ok.
                                    let _ = CENTRAL_COMMAND.send_background(Command::SetDependencyPackFilesList(entries));

                                    // Set the packfile as modified. This one is special, as this is a "simulated PackedFile", so we have to mark the PackFile manually.
                                    pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![ContainerPath::PackFile]), DataSource::PackFile);
                                    UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                                }

                                return Ok(())
                            } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) },
                            FileType::PackSettings => {
                                if let View::PackSettings(view) = view {
                                    let _ = CENTRAL_COMMAND.send_background(Command::SetPackSettings(view.save_view()));
                                    return Ok(())
                                } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) }
                            },
                            */

                            // Disable saving UIC until support for saving them is wired up.
                            #[cfg(feature = "support_uic")]
                            FileType::UIC => {
                                return Ok(());
                                //if let View::UIC(view) = view {
                                //    RFileDecoded::UIC(view.save_view())
                                //} else { return Err(ErrorKind::PackedFileSaveError(self.get_path()).into()) }
                            },

                            // UnitVariant use custom saving.
                            FileType::UnitVariant => return Ok(()),

                            // ESF files are re-generated from the view.
                            FileType::ESF => if let View::ESF(view) = view {
                                RFileDecoded::ESF(view.save_view())
                            } else { return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.get_path())) }

                            // Ignore these ones.
                            FileType::Unknown | FileType::Pack => return Ok(()),
                            _ => unimplemented!(),
                        };

                        // Save the PackedFile, and trigger the stuff that needs to be triggered after a save.
                        let receiver = CENTRAL_COMMAND.send_background(Command::SavePackedFileFromView(self.get_path(), data));
                        let response = CentralCommand::recv_try(&receiver);
                        match response {
                            Response::Success => {
                                Ok(())
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }
                    },
                    ViewType::External(view) => {
                        let receiver = CENTRAL_COMMAND.send_background(Command::SavePackedFileFromExternalView(self.get_path(), view.get_external_path()));
                        let response = CentralCommand::recv_try(&receiver);
                        match response {
                            Response::Success => {},
                            Response::Error(error) => show_dialog(pack_file_contents_ui.packfile_contents_tree_view(), error, false),
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }

                        Ok(())
                    }
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// This function reloads the data in a view from the backend. Useful to avoid having to close a PackedFile when the backend changes.
    pub unsafe fn reload(
        &mut self,
        path: &str,
        pack_file_contents_ui: &Rc<PackFileContentsUI>
    ) -> Result<()> {

        let data_source = self.get_data_source();
        if data_source != DataSource::ExternalFile {
            match self.get_ref_mut_view() {
                ViewType::Internal(view) => {

                    let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(path.to_owned(), data_source));
                    let response = CentralCommand::recv(&receiver);

                    match response {

                        Response::AnimFragmentRFileInfo(fragment, packed_file_info) => {
                            if let View::AnimFragment(old_fragment) = view {
                                if old_fragment.reload_view(fragment).is_err() {
                                    return Err(anyhow!(RFILE_RELOAD_ERROR));
                                }
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::AnimPackRFileInfo(anim_pack, files_info, file_info) => {
                            if let View::AnimPack(old_anim_pack) = view {
                                old_anim_pack.reload_view((anim_pack, files_info));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::AnimsTableRFileInfo(table, packed_file_info) => {
                            if let View::Table(old_table) = view {
                                let old_table = old_table.get_ref_table();
                                old_table.reload_view(TableType::AnimsTable(table));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::VideoRFileInfo(ca_vp8, packed_file_info) => {
                            if let View::Video(old_ca_vp8) = view {
                                old_ca_vp8.reload_view(&ca_vp8);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::DBRFileInfo(table, packed_file_info) => {
                            if let View::Table(old_table) = view {
                                let old_table = old_table.get_ref_table();
                                old_table.reload_view(TableType::DB(table));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::ImageRFileInfo(image, packed_file_info) => {
                            if let View::Image(old_image) = view {
                                old_image.reload_view(&image);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::LocRFileInfo(table, packed_file_info) => {
                            if let View::Table(old_table) = view {
                                let old_table = old_table.get_ref_table();
                                old_table.reload_view(TableType::Loc(table));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::MatchedCombatRFileInfo(table, packed_file_info) => {
                            if let View::Table(old_table) = view {
                                let old_table = old_table.get_ref_table();
                                old_table.reload_view(TableType::MatchedCombat(table));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        #[cfg(feature = "support_rigidmodel")]
                        Response::RigidModelRFileInfo((rigidmodel, packed_file_info)) => {
                            if let View::RigidModel(old_rigidmodel) = view {
                                old_rigidmodel.reload_view(&rigidmodel)?;
                                pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::TextRFileInfo(text, packed_file_info) => {
                            if let View::Text(old_text) = view {
                                old_text.reload_view(&text);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        // Debug views retun their entire file.
                        Response::RFileDecodedRFileInfo(packed_file, packed_file_info) => {
                            match packed_file {
                                RFileDecoded::UnitVariant(variant) => {
                                    if let View::UnitVariant(old_variant) = view {
                                        old_variant.reload_view(&variant);
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                                    }
                                    else {
                                        return Err(anyhow!(RFILE_RELOAD_ERROR));
                                    }
                                }
                                RFileDecoded::ESF(esf) => {
                                    if let View::ESF(old_esf) = view {
                                        old_esf.reload_view(&esf);
                                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                                    }
                                    else {
                                        return Err(anyhow!(RFILE_RELOAD_ERROR));
                                    }
                                }
                                _ => return Err(anyhow!(RFILE_RELOAD_ERROR)),
                            }
                        },

                        Response::Error(error) => return Err(error),
                        Response::Unknown => return Err(anyhow!("File Type Unknown.")),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    Ok(())
                },

                // External views don't need reloading.
                ViewType::External(_) => Ok(())
            }
        } else {
            Ok(())
        }
    }

    /// This function cleans the packedfile view from modified markers.
    pub unsafe fn clean(&self) {
        if let DataSource::PackFile = self.get_data_source() {
            if !self.get_is_read_only() {
                if let ViewType::Internal(view) = self.get_view() {

                    match self.packed_file_type {
                        FileType::AnimsTable |
                        FileType::DB |
                        FileType::Loc |
                        FileType::MatchedCombat => if let View::Table(view) = view {
                            view.get_ref_table().clear_markings();
                        } else if let View::AnimFragment(view) = view {
                            view.get_ref_table_view_2().clear_markings();
                        }
                        _ => {},
                    }
                }
            }
        }
    }
}

impl Display for DataSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(match self {
            Self::PackFile => "PackFile",
            Self::GameFiles => "GameFiles",
            Self::ParentFiles => "ParentFiles",
            Self::AssKitFiles => "AssKitFiles",
            Self::ExternalFile => "ExternalFile",
        }, f)
    }
}

impl From<&str> for DataSource {
    fn from(value: &str) -> Self {
        match value {
            "PackFile" => Self::PackFile,
            "GameFiles" => Self::GameFiles,
            "ParentFiles" => Self::ParentFiles,
            "AssKitFiles" => Self::AssKitFiles,
            "ExternalFile" => Self::ExternalFile,
            _ => unreachable!()
        }
    }
}

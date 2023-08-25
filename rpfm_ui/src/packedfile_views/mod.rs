//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
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
use getset::Getters;

use std::{fmt, fmt::Display};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use rpfm_lib::integrations::log::*;
use rpfm_lib::files::{atlas::Atlas, ContainerPath, db::DB, loc::Loc, FileType, RFileDecoded, text::Text};

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

use self::anim_fragment_battle::FileAnimFragmentBattleView;
use self::animpack::PackedFileAnimPackView;
use self::anims_table::FileAnimsTableDebugView;
use self::audio::FileAudioView;
use self::bmd::FileBMDView;
use self::esf::PackedFileESFView;
use self::decoder::PackedFileDecoderView;
use self::dependencies_manager::DependenciesManagerView;
use self::external::PackedFileExternalView;
use self::image::PackedFileImageView;
use self::matched_combat::FileMatchedCombatDebugView;
use self::notes::NotesView;
use self::packfile::PackFileExtraView;
use self::packfile_settings::PackFileSettingsView;
use self::portrait_settings::PortraitSettingsView;
use self::table::PackedFileTableView;
use self::text::PackedFileTextView;
use self::unit_variant::UnitVariantView;
use self::video::PackedFileVideoView;

#[cfg(feature = "support_rigidmodel")]
use self::rigidmodel::PackedFileRigidModelView;

#[cfg(feature = "support_uic")]
use self::uic::FileUICView;

pub mod anim_fragment_battle;
pub mod animpack;
pub mod anims_table;
pub mod audio;
pub mod bmd;
pub mod decoder;
pub mod dependencies_manager;
pub mod esf;
pub mod external;
pub mod image;
pub mod matched_combat;
pub mod notes;
pub mod packfile;
pub mod packfile_settings;
pub mod portrait_settings;

#[cfg(feature = "support_rigidmodel")]
pub mod rigidmodel;
pub mod table;
pub mod text;

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
#[derive(Getters)]
#[getset(get = "pub")]
pub struct FileView {
    path: Arc<RwLock<String>>,
    #[getset(skip)] data_source: Arc<RwLock<DataSource>>,
    #[getset(skip)] is_preview: AtomicBool,
    #[getset(skip)] is_read_only: AtomicBool,
    file_type: FileType,
    view_type: ViewType,

    #[getset(skip)] main_widget: Arc<QBox<QWidget>>,
    #[getset(skip)] notes_widget: Arc<QBox<QWidget>>,
    notes_view: Arc<NotesView>,
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
    AnimFragmentBattle(Arc<FileAnimFragmentBattleView>),
    AnimPack(Arc<PackedFileAnimPackView>),
    AnimsTableDebug(Arc<FileAnimsTableDebugView>),
    Audio(Arc<FileAudioView>),
    Bmd(Arc<FileBMDView>),
    Decoder(Arc<PackedFileDecoderView>),
    DependenciesManager(Arc<DependenciesManagerView>),
    Esf(Arc<PackedFileESFView>),
    Image(PackedFileImageView),
    MatchedCombatDebug(Arc<FileMatchedCombatDebugView>),
    PackFile(Arc<PackFileExtraView>),
    PackSettings(Arc<PackFileSettingsView>),
    PortraitSettings(Arc<PortraitSettingsView>),

    #[cfg(feature = "support_rigidmodel")]
    RigidModel(Arc<PackedFileRigidModelView>),
    Table(Arc<PackedFileTableView>),
    Text(Arc<PackedFileTextView>),

    #[cfg(feature = "support_uic")]
    UIC(Arc<FileUICView>),
    UnitVariant(Arc<UnitVariantView>),
    Video(Arc<PackedFileVideoView>),
    None,
}

pub enum SpecialView {
    Decoder(String),
    Pack(String),
    PackSettings,
    PackDependencies,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Default implementation for `FileView`.
impl Default for FileView {
    fn default() -> Self {
        let path = Arc::new(RwLock::new(String::new()));

        let main_widget_ptr = unsafe { QWidget::new_0a() };
        let main_layout = unsafe { create_grid_layout(main_widget_ptr.static_upcast()) };
        let main_widget = Arc::new(main_widget_ptr);

        let notes_widget_ptr = unsafe { QWidget::new_0a() };
        unsafe { create_grid_layout(notes_widget_ptr.static_upcast()); }
        unsafe { main_layout.add_widget_5a(&notes_widget_ptr, 0, 99, 1, 1); }
        let notes_widget = Arc::new(notes_widget_ptr);
        let notes_view = unsafe { NotesView::new_view(&notes_widget, path.clone()) };

        // Hide it by default.
        unsafe { notes_widget.set_visible(false) };

        let is_preview = AtomicBool::new(false);
        let is_read_only = AtomicBool::new(false);
        let data_source = Arc::new(RwLock::new(DataSource::PackFile));
        let view = ViewType::Internal(View::None);
        let packed_file_type = FileType::Unknown;
        Self {
            path,
            main_widget,
            notes_widget,
            notes_view,
            is_preview,
            is_read_only,
            data_source,
            view_type: view,
            file_type: packed_file_type,
        }
    }
}

/// Wacky fix for the "You cannot put a pointer in a static" problem.
unsafe impl Send for FileView {}
unsafe impl Sync for FileView {}

/// Implementation for `FileView`.
impl FileView {

    /// This function returns a copy of the path of this `FileView`.
    pub fn path_copy(&self) -> String {
        self.path.read().unwrap().to_owned()
    }

    /// This function returns a copy of the path of this `FileView`.
    pub fn path_raw(&self) -> Arc<RwLock<String>> {
        self.path.clone()
    }

    /// This function returns a reference to the path of this `FileView`.
    pub fn path_read(&self) -> RwLockReadGuard<String> {
        self.path.read().unwrap()
    }

    /// This function allows you to set a `FileView` as a preview or normal view.
    pub fn set_path(&self, path: &str) {
        *self.path.write().unwrap() = path.to_owned();

        // TODO: Move notes on this exact path to the new path.
        unsafe { self.notes_view.load_data() };
    }

    /// This function returns the DataSource of the specific `PackedFile`.
    pub fn data_source(&self) -> DataSource {
        self.data_source.read().unwrap().clone()
    }

    /// This function sets the DataSource of the specific `PackedFile`.
    pub fn set_data_source(&mut self, data_source: DataSource) {
        *self.data_source.write().unwrap() = data_source;
    }

    /// This function returns if the `FileView` is a preview or not.
    pub fn is_preview(&self) -> bool {
        self.is_preview.load(Ordering::SeqCst)
    }

    /// This function allows you to set a `FileView` as a preview or normal view.
    pub fn set_is_preview(&self, is_preview: bool) {
        self.is_preview.store(is_preview, Ordering::SeqCst);
    }

    /// This function returns if the `FileView` is read-only or not.
    pub fn is_read_only(&self) -> bool {
        self.is_read_only.load(Ordering::SeqCst)
    }

    /// This function allows you to set a `FileView` as a read-only or normal view.
    pub fn set_is_read_only(&self, is_read_only: bool) {
        self.is_read_only.store(is_read_only, Ordering::SeqCst);
    }

    /// This function returns a mutable reference to the ViewType of the specific `PackedFile`.
    pub fn view_type_mut(&mut self) -> &mut ViewType {
        &mut self.view_type
    }

    /// This function returns a mutable pointer to the `Widget` of the `FileView`.
    pub fn main_widget(&self) -> &QBox<QWidget> {
        &self.main_widget
    }

    /// This function returns a mutable pointer to the `Widget` of the `FileView`.
    pub fn notes_widget(&self) -> &QBox<QWidget> {
        &self.notes_widget
    }

    /// This function allows you to save a `FileView` to his corresponding `PackedFile`.
    pub unsafe fn save(&self, app_ui: &Rc<AppUI>, pack_file_contents_ui: &Rc<PackFileContentsUI>) -> Result<()> {

        // Only save non-read-only, local files.
        if let DataSource::PackFile = self.data_source() {
            if !self.is_read_only() {
                match self.view_type() {
                    ViewType::Internal(view) => {

                        let data = match view {
                            View::AnimFragmentBattle(view) => RFileDecoded::AnimFragmentBattle(view.save_view()?),
                            View::AnimPack(_) => return Ok(()),
                            View::AnimsTableDebug(_) => return Ok(()),
                            View::Audio(_) => return Ok(()),
                            View::Decoder(_) => return Ok(()),
                            View::DependenciesManager(view) => {
                                let mut entries = vec![];
                                let model = view.get_ref_table().table_model_ptr();
                                if model.is_null() {
                                    warn!("
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
                                        self.is_preview(),
                                        self.is_read_only(),
                                        view.get_ref_table().table_model_ptr().is_null(),
                                        view.get_ref_table().table_view_ptr().is_null()
                                    );

                                    show_dialog(app_ui.main_window(),
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
                                        let item = model.item_1a(row).text().to_std_string();
                                        entries.push(item);
                                    }

                                    // Save the new list and return Ok.
                                    let _ = CENTRAL_COMMAND.send_background(Command::SetDependencyPackFilesList(entries));

                                    // Set the packfile as modified. This one is special, as this is a "simulated PackedFile", so we have to mark the PackFile manually.
                                    pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::MarkAlwaysModified(vec![ContainerPath::Folder(String::new())]), DataSource::PackFile);
                                    UI_STATE.set_is_modified(true, app_ui, pack_file_contents_ui);
                                }
                                return Ok(())
                            },
                            View::Esf(view) => RFileDecoded::ESF(view.save_view()),
                            View::Image(_) => return Ok(()),
                            View::MatchedCombatDebug(_) => return Ok(()),
                            View::PackFile(_) => return Ok(()),
                            View::PackSettings(view) => {
                                let _ = CENTRAL_COMMAND.send_background(Command::SetPackSettings(view.save_view()));
                                return Ok(())
                            },

                            #[cfg(feature = "support_rigidmodel")]
                            View::RigidModel(view) => {
                                let data = view.save_view()?;
                                RFileDecoded::RigidModel(data)
                            }

                            View::Table(view) => {
                                let new_table = get_table_from_view(&view.get_ref_table().table_model_ptr().static_upcast(), &view.get_ref_table().table_definition())?;
                                match self.file_type {
                                    FileType::Atlas => {
                                        let table = Atlas::from(new_table);
                                        RFileDecoded::Atlas(table)
                                    }
                                    FileType::DB => {

                                        // If this crashes, it's a bug somewhere else.
                                        let table_name = view.get_ref_table().table_name().as_ref().unwrap();
                                        let mut table = DB::new(&view.get_ref_table().table_definition(), None, table_name);
                                        table.set_data(&new_table.data())?;
                                        RFileDecoded::DB(table)
                                    }
                                    FileType::Loc => {
                                        let table = Loc::from(new_table);
                                        RFileDecoded::Loc(table)
                                    }
                                    _ => return Err(anyhow!("{}{}", RFILE_SAVED_ERROR, self.path_copy()))
                                }
                            },
                            View::PortraitSettings(view) => RFileDecoded::PortraitSettings(view.save_view()),
                            View::Text(view) => {
                                let mut text = Text::default();
                                let widget = view.get_mut_editor();
                                let string = get_text_safe(widget).to_std_string();
                                text.set_contents(string);
                                RFileDecoded::Text(text)
                            },

                            // TODO: move this out of here, as it can (and will) fail on broken jsons.
                            View::Bmd(view) => {
                                let string = get_text_safe(view.get_mut_editor()).to_std_string();
                                let data = serde_json::from_str(&string)?;
                                RFileDecoded::BMD(data)
                            },
                            #[cfg(feature = "support_uic")]
                            View::UIC(view) => {
                                RFileDecoded::UIC(view.save_view())
                            },
                            View::UnitVariant(view) => RFileDecoded::UnitVariant(view.save_view()),
                            View::Video(view) => {
                                let _ = CENTRAL_COMMAND.send_background(Command::SetVideoFormat(self.path_copy(), view.get_current_format()));
                                return Ok(());
                            }

                            View::None => todo!(),
                        };

                        // Save the PackedFile, and trigger the stuff that needs to be triggered after a save.
                        let receiver = CENTRAL_COMMAND.send_background(Command::SavePackedFileFromView(self.path_copy(), data));
                        let response = CENTRAL_COMMAND.recv_try(&receiver);
                        match response {
                            Response::Success => {
                                Ok(())
                            }

                            // In ANY other situation, it's a message problem.
                            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
                        }
                    },
                    ViewType::External(view) => {
                        let receiver = CENTRAL_COMMAND.send_background(Command::SavePackedFileFromExternalView(self.path_copy(), view.get_external_path()));
                        let response = CENTRAL_COMMAND.recv_try(&receiver);
                        match response {
                            Response::Success => {},
                            Response::Error(error) => show_dialog(pack_file_contents_ui.packfile_contents_tree_view(), error, false),
                            _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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

        let data_source = self.data_source();
        if data_source != DataSource::ExternalFile {
            match self.view_type_mut() {
                ViewType::Internal(view) => {
                    let receiver = CENTRAL_COMMAND.send_background(Command::DecodePackedFile(path.to_owned(), data_source));
                    let response = CentralCommand::recv(&receiver);

                    match response {

                        Response::AnimFragmentBattleRFileInfo(fragment, packed_file_info) => {
                            if let View::AnimFragmentBattle(old_fragment) = view {
                                if old_fragment.reload_view(fragment).is_err() {
                                    return Err(anyhow!(RFILE_RELOAD_ERROR));
                                }
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::AnimPackRFileInfo(files_info, file_info) => {
                            if let View::AnimPack(old_anim_pack) = view {
                                old_anim_pack.reload_view((&file_info, files_info));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), DataSource::PackFile);

                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::AnimsTableRFileInfo(table, file_info) => {
                            if let View::AnimsTableDebug(old_table) = view {
                                if old_table.reload_view(table).is_err() {
                                    return Err(anyhow!(RFILE_RELOAD_ERROR));
                                }
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::AtlasRFileInfo(table, packed_file_info) => {
                            if let View::Table(old_table) = view {
                                let old_table = old_table.get_ref_table();
                                old_table.reload_view(TableType::Atlas(From::from(table)));
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::AudioRFileInfo(audio, packed_file_info) => {
                            if let View::Audio(old_audio) = view {
                                old_audio.reload_view(&audio);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::BmdRFileInfo(data, packed_file_info) => {
                            if let View::Bmd(old_data) = view {
                                old_data.reload_view(&data);
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

                        Response::ESFRFileInfo(esf, packed_file_info) => {
                            if let View::Esf(old_esf) = view {
                                old_esf.reload_view(&esf);
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

                        Response::MatchedCombatRFileInfo(data, file_info) => {
                            if let View::MatchedCombatDebug(old_data) = view {
                                old_data.reload_view(data)?;
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::PortraitSettingsRFileInfo(mut portrait_settings, packed_file_info) => {
                            if let View::PortraitSettings(old_portrait_settings) = view {
                                old_portrait_settings.reload_view(&mut portrait_settings)?;
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        #[cfg(feature = "support_rigidmodel")]
                        Response::RigidModelRFileInfo(rigidmodel, packed_file_info) => {
                            if let View::RigidModel(old_rigidmodel) = view {
                                old_rigidmodel.reload_view(&rigidmodel)?;
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
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

                        Response::Text(text) => {
                            if let View::Text(old_text) = view {
                                old_text.reload_view(&text);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::UnitVariantRFileInfo(mut variant, file_info) => {
                            if let View::UnitVariant(old_variant) = view {
                                let _ = old_variant.reload_view(&mut variant);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        }

                        Response::VideoInfoRFileInfo(video, packed_file_info) => {
                            if let View::Video(old_video) = view {
                                old_video.reload_view(&video);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::UpdateTooltip(vec![packed_file_info;1]), DataSource::PackFile);
                            }
                            else {
                                return Err(anyhow!(RFILE_RELOAD_ERROR));
                            }
                        },

                        Response::Error(error) => return Err(error),
                        Response::Unknown => return Err(anyhow!("File Type Unknown.")),
                        _ => panic!("{THREADS_COMMUNICATION_ERROR}{response:?}"),
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
        if let DataSource::PackFile = self.data_source() {
            if !self.is_read_only() {
                if let ViewType::Internal(view) = self.view_type() {

                    match self.file_type {
                        FileType::AnimsTable |
                        FileType::DB |
                        FileType::Loc |
                        FileType::MatchedCombat => if let View::Table(view) = view {
                            view.get_ref_table().clear_markings();
                        }

                        /*else if let View::AnimFragment(view) = view {
                            view.table_view().clear_markings();
                        }*/
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
            _ => unreachable!("from data source {}", value)
        }
    }
}

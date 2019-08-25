//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
// 
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
// 
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `AppUISlot`.
!*/

use qt_widgets::action::Action;
use qt_widgets::message_box::MessageBox;
use qt_widgets::widget::Widget;

use qt_gui::desktop_services::DesktopServices;

use qt_core::qt::FocusReason;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef};

use rpfm_lib::DOCS_BASE_URL;
use rpfm_lib::GAME_SELECTED;
use rpfm_lib::PATREON_URL;
use rpfm_lib::SETTINGS;

use crate::QString;
use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::command_palette;
use crate::communications::{THREADS_COMMUNICATION_ERROR, Command, Response};
use crate::settings_ui::SettingsUI;
use crate::utils::show_dialog;
use crate::UI_STATE;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action created at the start of the program.
///
/// This means everything you can do with the stuff you have in the `AppUI` goes here.
pub struct AppUISlots {

	//-----------------------------------------------//
    // Command Palette slots.
    //-----------------------------------------------//
    pub command_palette_show: SlotNoArgs<'static>,
    pub command_palette_hide: SlotNoArgs<'static>,
    pub command_palette_trigger: SlotStringRef<'static>,

    //-----------------------------------------------//
    // `PackFile` menu slots.
    //-----------------------------------------------//
    pub packfile_new_packfile: SlotBool<'static>,
    pub packfile_preferences: SlotBool<'static>,
    pub packfile_quit: SlotBool<'static>,

    //-----------------------------------------------//
    // `View` menu slots.
    //-----------------------------------------------//
    pub view_toggle_packfile_contents: SlotBool<'static>,
    pub view_toggle_global_search_panel: SlotBool<'static>,

    //-----------------------------------------------//
    // `About` menu slots.
    //-----------------------------------------------//
    pub about_about_qt: SlotBool<'static>,
    pub about_open_manual: SlotBool<'static>,
    pub about_patreon_link: SlotBool<'static>,

    //-----------------------------------------------//
    // PackFile Contents TreeView's context menu slots.
    //-----------------------------------------------//
    pub packfile_contents_tree_view_expand_all: SlotNoArgs<'static>,
    pub packfile_contents_tree_view_collapse_all: SlotNoArgs<'static>,

}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUISlots`.
impl AppUISlots {

	/// This function creates an entire `AppUISlots` struct. Used to create the logic of the starting UI.
	pub fn new(app_ui: AppUI) -> Self {

		//-----------------------------------------------//
        // Command Palette logic.
        //-----------------------------------------------//
		
        // This one puts the command palette in the top center part of the window, make it appear and gives it the focus.
		let command_palette_show = SlotNoArgs::new(move || {
			let width = (unsafe { app_ui.main_window.as_mut().unwrap().width() / 2 }) - (unsafe { app_ui.command_palette.as_mut().unwrap().width() / 2 });
			let height = 80;
            unsafe { app_ui.command_palette.as_mut().unwrap().move_((width, height)); }

            command_palette::load_actions(&app_ui);
            unsafe { app_ui.command_palette.as_mut().unwrap().show(); }
			unsafe { app_ui.command_palette_line_edit.as_mut().unwrap().set_focus(FocusReason::Shortcut); }
        });

		// This one hides the command palette.
        let command_palette_hide = SlotNoArgs::new(move || { 
            unsafe { app_ui.command_palette.as_mut().unwrap().hide(); }
        });

        // This is the fun one. This one triggers any command you type in the command palette.
        let command_palette_trigger = SlotStringRef::new(move |command| {
        	unsafe { app_ui.command_palette.as_mut().unwrap().hide(); }
            command_palette::exec_action(&app_ui, command);
        });

        //-----------------------------------------------//
        // `PackFile` menu logic.
        //-----------------------------------------------//

        // What happens when we trigger the "New PackFile" action.
        let packfile_new_packfile = SlotBool::new(
            /*clone!(
            mymod_stuff,
            mode,
            table_state_data,
            packedfiles_open_in_packedfile_view =>*/ move |_| {
                
                // Check first if there has been changes in the PackFile.
                if app_ui.are_you_sure(false) {

                    // Tell the Background Thread to create a new PackFile.
                    CENTRAL_COMMAND.send_message_qt(Command::NewPackFile);
                    
                    // Disable the main window, so the user can't interrupt the process or iterfere with it.
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    // Close any open PackedFile and clear the global search pannel.
                    // TODO: Clear the global search panel.
                    app_ui.purge_them_all();
                    unsafe { app_ui.global_search_dock_widget.as_mut().unwrap().hide(); }
                    //if !SETTINGS.lock().unwrap().settings_bool["remember_table_state_permanently"] { TABLE_STATES_UI.lock().unwrap().clear(); }

                    // Show the "Tips".
                    //display_help_tips(&app_ui);


                    // New PackFiles are always of Mod type.
                    unsafe { app_ui.change_packfile_type_mod.as_mut().unwrap().set_checked(true); }

                    // By default, the four bitmask should be false.
                    unsafe { app_ui.change_packfile_type_data_is_encrypted.as_mut().unwrap().set_checked(false); }
                    unsafe { app_ui.change_packfile_type_index_includes_timestamp.as_mut().unwrap().set_checked(false); }
                    unsafe { app_ui.change_packfile_type_index_is_encrypted.as_mut().unwrap().set_checked(false); }
                    unsafe { app_ui.change_packfile_type_header_is_extended.as_mut().unwrap().set_checked(false); }

                    // We also disable compression by default.
                    unsafe { app_ui.change_packfile_type_data_is_compressed.as_mut().unwrap().set_checked(false); }
/*
                    // Update the TreeView.
                    update_treeview(
                        &sender_qt,
                        &sender_qt_data,
                        &receiver_qt,
                        &app_ui,
                        app_ui.folder_tree_view,
                        Some(app_ui.folder_tree_filter),
                        app_ui.folder_tree_model,
                        TreeViewOperation::Build(false),
                    );
*/
                    // Re-enable the Main Window.
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }

                    // Enable the actions available for the PackFile from the `MenuBar`.
                    //enable_packfile_actions(&app_ui, &mymod_stuff, true);

                    // Set the current "Operational Mode" to Normal, as this is a "New" mod.
                    //set_my_mod_mode(&mymod_stuff, &mode, None);

                    // Clean the TableStateData.
                    //*table_state_data.borrow_mut() = TableStateData::new(); 
                }
            }
        );

        // What happens when we trigger the "Preferences" action.
        let packfile_preferences = SlotBool::new(move |_| {

                // We store a copy of the old settings (for checking changes) and trigger the new settings dialog.
                let old_settings = SETTINGS.lock().unwrap().clone();
                if let Some(settings) = SettingsUI::new(&app_ui) {

                    // If we returned new settings, save them and wait for confirmation.
                    CENTRAL_COMMAND.send_message_qt(Command::SetSettings(settings.clone()));
                    match CENTRAL_COMMAND.recv_message_qt() {

                        // If it worked, do some checks to ensure the UI keeps his consistency.
                        Response::Success => {

                            // If we changed the "MyMod's Folder" path, disable the MyMod mode and set it so the MyMod menu will be re-built
                            // next time we open the MyMod menu.
                            if settings.paths["mymods_base_path"] != old_settings.paths["mymods_base_path"] {
                                UI_STATE.set_operational_mode(&app_ui, None);
                                UI_STATE.set_mymod_menu_needs_rebuild(true);
                            }

                            // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                            // re-select the current `GameSelected` to force it to reload the game's files.
                            let has_game_selected_path_changed = settings.paths.iter()
                                .filter(|x| x.0 != "mymods_base_path" && &old_settings.paths[x.0] != x.1)
                                .any(|x| x.0 == &*GAME_SELECTED.lock().unwrap());

                            if has_game_selected_path_changed {
                                unsafe { Action::trigger(app_ui.game_selected_group.as_mut().unwrap().checked_action().as_mut().unwrap()); }
                            }
                        }

                        // If we got an error, report it.
                        Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),

                        // In ANY other situation, it's a message problem.
                        _ => panic!(THREADS_COMMUNICATION_ERROR)
                    }
                }
            }
        );

        // What happens when we trigger the "Quit" action.
        let packfile_quit = SlotBool::new(clone!(
            app_ui => move |_| {
                if app_ui.are_you_sure(false) {
                    unsafe { app_ui.main_window.as_mut().unwrap().close(); }
                }
            }
        ));

        //-----------------------------------------------//
        // `View` menu logic.
        //-----------------------------------------------//
        let view_toggle_packfile_contents = SlotBool::new(move |_| { 
            let is_visible = unsafe { app_ui.packfile_contents_dock_widget.as_mut().unwrap().is_visible() };
            if is_visible { unsafe { app_ui.packfile_contents_dock_widget.as_mut().unwrap().hide(); }}
            else {unsafe { app_ui.packfile_contents_dock_widget.as_mut().unwrap().show(); }}
        });

        let view_toggle_global_search_panel = SlotBool::new(move |_| { 
            let is_visible = unsafe { app_ui.global_search_dock_widget.as_mut().unwrap().is_visible() };
            if is_visible { unsafe { app_ui.global_search_dock_widget.as_mut().unwrap().hide(); }}
            else {unsafe { app_ui.global_search_dock_widget.as_mut().unwrap().show(); }}
        });

		//-----------------------------------------------//
        // `About` menu logic.
        //-----------------------------------------------//
        
        // What happens when we trigger the "About Qt" action.
        let about_about_qt = SlotBool::new(move |_| { unsafe { MessageBox::about_qt(app_ui.main_window as *mut Widget); }});

        // What happens when we trigger the "Open Manual" action.
        let about_open_manual = SlotBool::new(|_| { DesktopServices::open_url(&qt_core::url::Url::new(&QString::from_std_str(DOCS_BASE_URL))); });

        // What happens when we trigger the "Support me on Patreon" action.
        let about_patreon_link = SlotBool::new(|_| { DesktopServices::open_url(&qt_core::url::Url::new(&QString::from_std_str(PATREON_URL))); });

        //-----------------------------------------------//
        // PackFile Contents TreeView's context menu logic.
        //-----------------------------------------------//
        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { app_ui.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }});
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { app_ui.packfile_contents_tree_view.as_mut().unwrap().collapse_all(); }});

        // And here... we return all the slots.
		Self {
		
			//-----------------------------------------------//
	        // Command Palette slots.
	        //-----------------------------------------------//
			command_palette_show,
    		command_palette_hide,
    		command_palette_trigger,

            //-----------------------------------------------//
            // `PackFile` menu slots.
            //-----------------------------------------------//
            packfile_new_packfile,
            packfile_preferences,
            packfile_quit,

            //-----------------------------------------------//
            // `View` menu slots.
            //-----------------------------------------------//
            view_toggle_packfile_contents,
            view_toggle_global_search_panel,

    		//-----------------------------------------------//
	        // `About` menu slots.
	        //-----------------------------------------------//
    		about_about_qt,
            about_open_manual,
            about_patreon_link,

            //-----------------------------------------------//
            // PackFile Contents TreeView's context menu slots.
            //-----------------------------------------------//
            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}
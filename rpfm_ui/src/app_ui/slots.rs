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

use qt_widgets::message_box::MessageBox;
use qt_widgets::widget::Widget;

use qt_gui::desktop_services::DesktopServices;

use qt_core::qt::FocusReason;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef};

use rpfm_lib::DOCS_BASE_URL;
use rpfm_lib::PATREON_URL;

use crate::QString;
use crate::app_ui::AppUI;
use crate::command_palette;

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

        // What happens when we trigger the "Quit" action.
        let slot_quit = SlotBool::new(clone!(
            app_ui => move |_| {
                /*if are_you_sure(&app_ui, false) {
                    unsafe { app_ui.window.as_mut().unwrap().close(); }
                }*/
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
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
Module with all the code for the template view system, because it was too complex to fit a simple function in the AppUI.

Unlike other views, this one is triggered and destroyed in a dialog here, so the big struct
is not really neccesary, but it makes things easier to understand.
!*/

use qt_widgets::QTextEdit;
use qt_widgets::QTableView;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDialog;
use qt_widgets::QGroupBox;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QLabel;
use qt_widgets::QWidget;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QPtr;

use cpp_core::Ref;

use std::cell::RefCell;
use std::rc::Rc;

use rpfm_lib::schema::Definition;
use rpfm_lib::template::*;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::locale::qtr;
use crate::QString;
use crate::utils::create_grid_layout;
use crate::views::table::utils::*;

mod connections;
mod slots;

//-------------------------------------------------------------------------------//
//                             Structs & Enums
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to all the items in a dinamically-created Template view.
#[derive(Debug)]
pub struct TemplateUI {

    pub template: Rc<Template>,
    pub options: Rc<RefCell<Vec<(String, QPtr<QCheckBox>)>>>,
    pub params: Rc<RefCell<Vec<(String, QPtr<QWidget>)>>>,


    pub dialog: QBox<QDialog>,
    pub accept_button: QBox<QPushButton>,
    //pub menu_bar: QPtr<QMenuBar>,
    //pub status_bar: QPtr<QStatusBar>,
}

/// This struct contains all the pointers we need to access to all the items in a `Save Template to PackFile` dialog.
#[derive(Debug)]
pub struct SaveTemplateUI {
    pub step_1_tableview: QBox<QTableView>,
    pub step_1_model: QBox<QStandardItemModel>,
    pub step_1_add_button: QBox<QPushButton>,
    pub step_1_remove_button: QBox<QPushButton>,
    pub step_2_tableview: QBox<QTableView>,
    pub step_2_model: QBox<QStandardItemModel>,
    pub step_2_add_button: QBox<QPushButton>,
    pub step_2_remove_button: QBox<QPushButton>,
    pub step_3_tableview: QBox<QTableView>,
    pub step_3_model: QBox<QStandardItemModel>,
    pub step_3_add_button: QBox<QPushButton>,
    pub step_3_remove_button: QBox<QPushButton>,

    pub name_line_edit: QBox<QLineEdit>,
    pub description_line_edit: QBox<QLineEdit>,
    pub author_line_edit: QBox<QLineEdit>,
    pub post_message_line_edit: QBox<QTextEdit>,

}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TemplateUI`.
impl TemplateUI {

    /// This function creates the entire "Load Template" dialog. It returns a vector with the stuff set in it.
    pub unsafe fn load(app_ui: &Rc<AppUI>, template: &Template) -> Option<(Vec<(String, bool)>, Vec<(String, String)>)> {

        let dialog = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("load_templates_dialog_title"));
        dialog.set_modal(true);

        // Create the main Grid.
        let main_grid = create_grid_layout(dialog.static_upcast());
        main_grid.set_contents_margins_4a(4, 0, 4, 4);
        main_grid.set_spacing(4);

        // Buttons
        let accept_button = QPushButton::from_q_string(&qtr("load_templates_dialog_accept"));
        main_grid.add_widget_5a(&accept_button, 99, 0, 1, 2);

        let ui = Rc::new(Self {
            template: Rc::new(template.clone()),
            options: Rc::new(RefCell::new(vec![])),
            params: Rc::new(RefCell::new(vec![])),

            dialog,
            accept_button,
        });

        let info_section = Self::load_info_section(&ui);
        main_grid.add_widget_5a(&info_section, 0, 0, 1, 1);

        let mut h = 1;
        let mut v = 0;
        for section in ui.template.get_sections() {
            let section = Self::load_section(&ui, section);
            main_grid.add_widget_5a(&section, v, h, 1, 1);
            h += 1;

            if h == 6 {
                h = 0;
                v += 1;
            }
        }

        // Pass for sectionless items.
        let empty_section = TemplateSection::default();
        let section = Self::load_section(&ui, &empty_section);
        if section.children().is_empty() {
            section.set_visible(false);
        }
        main_grid.add_widget_5a(&section, h, v, 1, 1);

        // Slots and connections.
        let slots = slots::TemplateUISlots::new(&ui);
        connections::set_connections_template(&ui, &slots);

        ui.update_template_view();

        // Execute the dialog.
        if ui.dialog.exec() == 1 {
            Some(ui.get_data_from_view())
        }

        // Otherwise, return None.
        else { None }
    }

    /// This function loads the info section into the view.
    ///
    /// This section is usually static, so no complex stuff here.
    unsafe fn load_info_section(ui: &Rc<Self>) -> QBox<QGroupBox> {

        let widget = QGroupBox::from_q_string(&QString::from_std_str("Template Info"));
        let grid = create_grid_layout(widget.static_upcast());

        let author_label = QLabel::from_q_string_q_widget(&QString::from_std_str("By: ".to_owned() + &ui.template.author), &widget);
        let description_label = QLabel::from_q_string_q_widget(&QString::from_std_str(&ui.template.description), &widget);

        grid.add_widget_5a(&author_label, 0, 0, 1, 2);
        grid.add_widget_5a(&description_label, 1 , 0, 1, 2);

        widget
    }

    /// This function loads the info section into the view.
    ///
    /// This section is usually static, so no complex stuff here.
    unsafe fn load_section(ui: &Rc<Self>, section: &TemplateSection) -> QBox<QGroupBox> {

        let widget = QGroupBox::from_q_string(&QString::from_std_str(section.get_ref_name()));
        let grid = create_grid_layout(widget.static_upcast());

        let mut count = 0;
        ui.template.get_options().iter().filter(|x| x.get_ref_section() == section.get_ref_key()).for_each(|z| {
            let widget = Self::load_option_data(ui, z);
            grid.add_widget_5a(&widget, count as i32, 0, 1, 1);
            count += 1;
        });
        count += 2;
        ui.template.get_params().iter().filter(|x| x.get_ref_section() == section.get_ref_key()).for_each(|z| {
            let widget = Self::load_field_data(ui, z);
            grid.add_widget_5a(&widget, count as i32, 0, 1, 1);
            count += 1;
        });

        widget
    }

    unsafe fn load_option_data(ui: &Rc<Self>, option: &TemplateOption) -> QBox<QWidget> {
        let widget = QWidget::new_0a();
        let grid = create_grid_layout(widget.static_upcast());

        let label = QLabel::from_q_string_q_widget(&QString::from_std_str(option.get_ref_name()), &widget);
        let checkbox = QCheckBox::from_q_widget(&widget);
        label.set_minimum_width(100);

        grid.add_widget_5a(&label, 0, 0, 1, 1);
        grid.add_widget_5a(&checkbox, 0, 1, 1, 1);

        ui.options.borrow_mut().push((option.get_ref_key().to_owned(), checkbox.static_upcast()));

        widget
    }

    unsafe fn load_field_data(ui: &Rc<Self>, param: &TemplateParam) -> QBox<QWidget> {
        let widget = QWidget::new_0a();
        let grid = create_grid_layout(widget.static_upcast());

        let label = QLabel::from_q_string_q_widget(&QString::from_std_str(param.get_ref_name()), &widget);
        label.set_minimum_width(100);

        match param.get_ref_param_type() {
            ParamType::Text => {
                let field_widget = QLineEdit::from_q_widget(&widget);
                field_widget.set_minimum_width(250);
                grid.add_widget_5a(&label, 0, 0, 1, 1);
                grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast()));
            }

            ParamType::TableField(field) => {
                let mut definition = Definition::new(-1);
                *definition.get_ref_mut_fields() = vec![field.clone()];

                let ref_data = get_reference_data(&definition);

                match ref_data {
                    Ok(ref_data) => {
                        if ref_data.is_empty() {
                            let field_widget = QLineEdit::from_q_widget(&widget);
                            field_widget.set_minimum_width(250);
                            grid.add_widget_5a(&label, 0, 0, 1, 1);
                            grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                            ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast()));
                        }
                        else {

                            let field_widget = QComboBox::new_1a(&widget);
                            field_widget.set_minimum_width(250);
                            grid.add_widget_5a(&label, 0, 0, 1, 1);
                            grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                            ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast()));

                            for ref_data in ref_data.get(&0).unwrap().keys() {
                                field_widget.add_item_q_string(&QString::from_std_str(ref_data));
                            }
                        }
                    }
                    Err(_) => {
                        let field_widget = QLineEdit::from_q_widget(&widget);
                        field_widget.set_minimum_width(250);
                        grid.add_widget_5a(&label, 0, 0, 1, 1);
                        grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                        ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast()));
                    }
                }

            }

            ParamType::Table(definition) => {
                unimplemented!()
            }
        }

        widget
    }

    /// This function returns the options/parameters from the view.
    pub unsafe fn get_data_from_view(&self) -> (Vec<(String, bool)>, Vec<(String, String)>) {
        let options = self.options.borrow().iter().map(|(key, widget)| (key.to_owned(), widget.is_checked())).collect();

        let params = self.params.borrow().iter().map(|(key, widget)| (key.to_owned(),
            if !widget.dynamic_cast::<QComboBox>().is_null() {
                widget.static_downcast::<QComboBox>().current_text().to_std_string()
            } else if !widget.dynamic_cast::<QLineEdit>().is_null() {
                widget.static_downcast::<QLineEdit>().text().to_std_string()
            } else {
                todo!()
            }
        )).collect();

        (options, params)
    }

    /// This function updates the state of the UI when we enable/disable parts of the template.
    pub unsafe fn update_template_view(&self) {
        let options_enabled = self.options.borrow().iter().filter_map(|(x, y)| if y.is_checked() { Some(x.to_owned()) } else { None }).collect::<Vec<String>>();
        for option in self.template.get_options() {
            if option.has_required_options(&options_enabled) {
                if let Some(option) = self.options.borrow().iter().find(|(x, _)| x == option.get_ref_key()) {
                    option.1.set_enabled(true);
                }
            }
            else {
                if let Some(option) = self.options.borrow().iter().find(|(x, _)| x == option.get_ref_key()) {
                    option.1.set_enabled(false);
                }
            }
        }

        for param in self.template.get_params() {
            if param.has_required_options(&options_enabled) {
                if let Some(param) = self.params.borrow().iter().find(|(x, _)| x == param.get_ref_key()) {
                    param.1.set_enabled(true);
                }
            }
            else {
                if let Some(param) = self.params.borrow().iter().find(|(x, _)| x == param.get_ref_key()) {
                    param.1.set_enabled(false);
                }
            }
        }
    }
}

/// Implamentation of `SaveTemplateUI`.
impl SaveTemplateUI {

    /// This function creates the "New Template" dialog when saving the currently open PackFile into a Template.
    ///
    /// It returns the new name of the Template, or `None` if the dialog is canceled or closed.
    pub unsafe fn load(app_ui: &Rc<AppUI>) -> Option<(String, String, String, String, Vec<(String, String)>, Vec<(String, String, String)>, Vec<(String, String, String, String)>)> {

        // Create and configure the dialog.
        let dialog: QBox<QDialog> = QDialog::new_1a(&app_ui.main_window);
        dialog.set_window_title(&qtr("save_template"));
        dialog.set_modal(true);
        dialog.resize_2a(1000, 400);

        let main_grid = create_grid_layout(dialog.static_upcast());

        //-----------------------------------------//
        // Step 1: Sections.
        //-----------------------------------------//
        let step_1_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_step_1"), &dialog);
        let step_1_grid = create_grid_layout(step_1_groupbox.static_upcast());
        step_1_groupbox.set_minimum_width(300);

        let step_1_description_label = QLabel::from_q_string_q_widget(&qtr("new_template_step_1_description"), &step_1_groupbox);
        let step_1_tableview = QTableView::new_1a(&step_1_groupbox);
        let step_1_model = QStandardItemModel::new_1a(&step_1_tableview);
        let step_1_add_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &step_1_groupbox);
        let step_1_remove_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &step_1_groupbox);
        step_1_tableview.set_model(&step_1_model);
        step_1_tableview.horizontal_header().set_stretch_last_section(true);
        step_1_description_label.set_word_wrap(true);

        step_1_grid.add_widget_5a(&step_1_description_label, 0, 0, 1, 2);
        step_1_grid.add_widget_5a(&step_1_tableview, 1, 0, 1, 2);
        step_1_grid.add_widget_5a(&step_1_add_button, 2, 0, 1, 1);
        step_1_grid.add_widget_5a(&step_1_remove_button, 2, 1, 1, 1);
        step_1_grid.set_row_stretch(1, 99);

        //-----------------------------------------//
        // Step 2: Options.
        //-----------------------------------------//
        let step_2_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_step_2"), &dialog);
        let step_2_grid = create_grid_layout(step_2_groupbox.static_upcast());
        step_2_groupbox.set_minimum_width(450);

        let step_2_description_label = QLabel::from_q_string_q_widget(&qtr("new_template_step_2_description"), &step_2_groupbox);
        let step_2_tableview = QTableView::new_1a(&step_2_groupbox);
        let step_2_model = QStandardItemModel::new_1a(&step_2_tableview);
        let step_2_add_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &step_2_groupbox);
        let step_2_remove_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &step_2_groupbox);
        step_2_tableview.set_model(&step_2_model);
        step_2_tableview.horizontal_header().set_stretch_last_section(true);
        step_2_description_label.set_word_wrap(true);

        step_2_grid.add_widget_5a(&step_2_description_label, 0, 0, 1, 2);
        step_2_grid.add_widget_5a(&step_2_tableview, 1, 0, 1, 2);
        step_2_grid.add_widget_5a(&step_2_add_button, 2, 0, 1, 1);
        step_2_grid.add_widget_5a(&step_2_remove_button, 2, 1, 1, 1);
        step_2_grid.set_row_stretch(1, 99);

        //-----------------------------------------//
        // Step 3: Parameters.
        //-----------------------------------------//
        let step_3_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_step_3"), &dialog);
        let step_3_grid = create_grid_layout(step_3_groupbox.static_upcast());
        step_3_groupbox.set_minimum_width(450);

        let step_3_description_label = QLabel::from_q_string_q_widget(&qtr("new_template_step_3_description"), &step_3_groupbox);
        let step_3_tableview = QTableView::new_1a(&step_3_groupbox);
        let step_3_model = QStandardItemModel::new_1a(&step_3_tableview);
        let step_3_add_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &step_3_groupbox);
        let step_3_remove_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &step_3_groupbox);
        step_3_tableview.set_model(&step_3_model);
        step_3_tableview.horizontal_header().set_stretch_last_section(true);
        step_3_description_label.set_word_wrap(true);

        step_3_grid.add_widget_5a(&step_3_description_label, 0, 0, 1, 2);
        step_3_grid.add_widget_5a(&step_3_tableview, 1, 0, 1, 2);
        step_3_grid.add_widget_5a(&step_3_add_button, 2, 0, 1, 1);
        step_3_grid.add_widget_5a(&step_3_remove_button, 2, 1, 1, 1);
        step_3_grid.set_row_stretch(1, 99);

        //-----------------------------------------//
        // Step 4: Finish.
        //-----------------------------------------//
        let step_4_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_step_4"), &dialog);
        let step_4_grid = create_grid_layout(step_4_groupbox.static_upcast());
        step_4_groupbox.set_minimum_width(300);

        let step_4_description_label = QLabel::from_q_string_q_widget(&qtr("new_template_step_4_description"), &step_4_groupbox);

        let name_label = QLabel::from_q_string_q_widget(&qtr("template_name"), &step_4_groupbox);
        let name_line_edit = QLineEdit::from_q_widget(&step_4_groupbox);

        let description_label = QLabel::from_q_string_q_widget(&qtr("template_description"), &step_4_groupbox);
        let description_line_edit = QLineEdit::from_q_widget(&step_4_groupbox);

        let author_label = QLabel::from_q_string_q_widget(&qtr("template_author"), &step_4_groupbox);
        let author_line_edit = QLineEdit::from_q_widget(&step_4_groupbox);

        let post_message_label = QLabel::from_q_string_q_widget(&qtr("template_post_message"), &step_4_groupbox);
        let post_message_line_edit = QTextEdit::from_q_widget(&step_4_groupbox);

        let accept_button = QPushButton::from_q_string_q_widget(&qtr("gen_loc_accept"), &step_4_groupbox);

        step_4_description_label.set_word_wrap(true);

        step_4_grid.add_widget_5a(&step_4_description_label, 0, 0, 1, 2);
        step_4_grid.add_widget_5a(&name_label, 1, 0, 1, 1);
        step_4_grid.add_widget_5a(&name_line_edit, 1, 1, 1, 1);
        step_4_grid.add_widget_5a(&description_label, 2, 0, 1, 1);
        step_4_grid.add_widget_5a(&description_line_edit, 2, 1, 1, 1);
        step_4_grid.add_widget_5a(&author_label, 3, 0, 1, 1);
        step_4_grid.add_widget_5a(&author_line_edit, 3, 1, 1, 1);
        step_4_grid.add_widget_5a(&post_message_label, 4, 0, 1, 1);
        step_4_grid.add_widget_5a(&post_message_line_edit, 4, 1, 1, 1);
        step_4_grid.add_widget_5a(&accept_button, 11, 0, 1, 2);
        step_4_grid.set_row_stretch(1, 99);

        //-----------------------------------------//
        // Finishing layouts and execution.
        //-----------------------------------------//
        main_grid.add_widget_5a(&step_1_groupbox, 0, 0, 1, 1);
        main_grid.add_widget_5a(&step_2_groupbox, 0, 1, 1, 1);
        main_grid.add_widget_5a(&step_3_groupbox, 0, 2, 1, 1);
        main_grid.add_widget_5a(&step_4_groupbox, 0, 3, 1, 1);

        accept_button.released().connect(dialog.slot_accept());

        let ui = Rc::new(Self{
            step_1_tableview,
            step_1_model,
            step_1_add_button,
            step_1_remove_button,
            step_2_tableview,
            step_2_model,
            step_2_add_button,
            step_2_remove_button,
            step_3_tableview,
            step_3_model,
            step_3_add_button,
            step_3_remove_button,
            name_line_edit,
            description_line_edit,
            author_line_edit,
            post_message_line_edit,
        });

        ui.populate_template_view();

        let slots = slots::SaveTemplateUISlots::new(&ui);
        connections::set_connections_save_template(&ui, &slots);

        if dialog.exec() == 1 {
            ui.get_data_from_view()
        } else { None }
    }

    /// This function updates the state of the UI when we enable/disable parts of the template.
    pub unsafe fn populate_template_view(&self) {

        // First, get the definition list from the backend.
        CENTRAL_COMMAND.send_message_qt(Command::GetDefinitionList);
        let response = CENTRAL_COMMAND.recv_message_qt();
        let definitions = match response {
            Response::VecDefinition(definitions) => definitions,
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // If there are definitions, use them to fill the sections/params views.
        if !definitions.is_empty() {
            for (index, definition) in definitions.iter().enumerate() {
                let section = format!("section_{}", index);
                let qlist_boi = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(&section));
                let value = QStandardItem::from_q_string(&QString::from_std_str(&section));

                qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
                self.step_1_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

                for field in definition.get_ref_fields() {
                    let qlist_boi = QListOfQStandardItem::new();
                    let key = QStandardItem::from_q_string(&QString::from_std_str(field.get_name()));
                    let value = QStandardItem::from_q_string(&QString::from_std_str(field.get_name()));
                    let section = QStandardItem::from_q_string(&QString::from_std_str(&section));
                    let param_type = QStandardItem::from_q_string(&QString::from_std_str(serde_json::to_string(&ParamType::TableField(field.clone())).unwrap()));

                    qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&param_type.into_ptr().as_mut_raw_ptr());
                    self.step_3_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
                }
            }
        }

        // Otherwise, leave them empty.
        else {
            let qlist_boi = QListOfQStandardItem::new();
            let key = QStandardItem::new();
            let value = QStandardItem::new();

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
            self.step_1_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

            let qlist_boi = QListOfQStandardItem::new();
            let key = QStandardItem::new();
            let value = QStandardItem::new();
            let section = QStandardItem::new();
            let param_type = QStandardItem::new();

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&param_type.into_ptr().as_mut_raw_ptr());
            self.step_3_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
        }

        let qlist_boi = QListOfQStandardItem::new();
        let key = QStandardItem::new();
        let value = QStandardItem::new();
        let section = QStandardItem::new();

        qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
        qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
        qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());
        self.step_2_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

        let step_1_columm_1 = QStandardItem::from_q_string(&qtr("key"));
        let step_1_columm_2 = QStandardItem::from_q_string(&qtr("value"));
        self.step_1_model.set_horizontal_header_item(0, step_1_columm_1.into_ptr());
        self.step_1_model.set_horizontal_header_item(1, step_1_columm_2.into_ptr());

        let step_2_columm_1 = QStandardItem::from_q_string(&qtr("key"));
        let step_2_columm_2 = QStandardItem::from_q_string(&qtr("value"));
        let step_2_columm_3 = QStandardItem::from_q_string(&qtr("section"));
        self.step_2_model.set_horizontal_header_item(0, step_2_columm_1.into_ptr());
        self.step_2_model.set_horizontal_header_item(1, step_2_columm_2.into_ptr());
        self.step_2_model.set_horizontal_header_item(2, step_2_columm_3.into_ptr());

        let step_3_columm_1 = QStandardItem::from_q_string(&qtr("key"));
        let step_3_columm_2 = QStandardItem::from_q_string(&qtr("value"));
        let step_3_columm_3 = QStandardItem::from_q_string(&qtr("section"));
        let step_3_columm_4 = QStandardItem::from_q_string(&qtr("type"));
        self.step_3_model.set_horizontal_header_item(0, step_3_columm_1.into_ptr());
        self.step_3_model.set_horizontal_header_item(1, step_3_columm_2.into_ptr());
        self.step_3_model.set_horizontal_header_item(2, step_3_columm_3.into_ptr());
        self.step_3_model.set_horizontal_header_item(2, step_3_columm_4.into_ptr());
    }

    /// This function returns the options/parameters from the view.
    pub unsafe fn get_data_from_view(&self) -> Option<(String, String, String, String, Vec<(String, String)>, Vec<(String, String, String)>, Vec<(String, String, String, String)>)> {
        // Get all the data from the view.
        let name = self.name_line_edit.text().to_std_string();
        let description = self.description_line_edit.text().to_std_string();
        let author = self.author_line_edit.text().to_std_string();
        let post_message = self.post_message_line_edit.to_plain_text().to_std_string();

        let mut sections = vec![];
        for row in 0..self.step_1_model.row_count_0a() {
            let section = (self.step_1_model.item_2a(row, 0).text().to_std_string(), self.step_1_model.item_2a(row, 1).text().to_std_string());
            if !section.0.is_empty() && !section.1.is_empty() {
                sections.push(section);
            }
        }

        let mut options = vec![];
        for row in 0..self.step_2_model.row_count_0a() {
            let option = (self.step_2_model.item_2a(row, 0).text().to_std_string(), self.step_2_model.item_2a(row, 1).text().to_std_string(), self.step_2_model.item_2a(row, 2).text().to_std_string());
            if !option.0.is_empty() && !option.1.is_empty() {
                options.push(option);
            }
        }

        let mut params = vec![];
        for row in 0..self.step_3_model.row_count_0a() {
            let param = (self.step_3_model.item_2a(row, 0).text().to_std_string(), self.step_3_model.item_2a(row, 1).text().to_std_string(), self.step_3_model.item_2a(row, 2).text().to_std_string(), self.step_3_model.item_2a(row, 3).text().to_std_string());
            if !param.0.is_empty() && !param.1.is_empty() {
                params.push(param);
            }
        }

        if !name.is_empty() && !description.is_empty() && !author.is_empty() {
            Some((
                name,
                description,
                author,
                post_message,
                sections,
                options,
                params
            ))
        } else { None }
    }
}

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

use qt_widgets::q_header_view::ResizeMode;
use qt_widgets::QTextEdit;
use qt_widgets::QTableView;
use qt_widgets::QCheckBox;
use qt_widgets::QComboBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QGroupBox;
use qt_widgets::QLineEdit;
use qt_widgets::QPushButton;
use qt_widgets::QLabel;
use qt_widgets::QSpinBox;
use qt_widgets::QWidget;
use qt_widgets::{QWizard, q_wizard::{WizardButton, WizardOption}};
use qt_widgets::QWizardPage;

use qt_gui::QListOfQStandardItem;
use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::CheckState;
use qt_core::QBox;
use qt_core::QModelIndex;
use qt_core::QPtr;

use cpp_core::Ref;

use std::sync::{Arc, RwLock};
use std::cell::RefCell;
use std::rc::Rc;

use rpfm_lib::packedfile::table::Table;
use rpfm_lib::schema::{Definition, FieldType};
use rpfm_lib::template::*;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::*;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::qtr;
use crate::packedfile_views::DataSource;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::QString;
use crate::utils::create_grid_layout;
use crate::views::table::{TableView, TableType, utils::*};

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
    pub params: Rc<RefCell<Vec<(String, QPtr<QWidget>, ParamType, bool)>>>,

    pub wazard: QBox<QWizard>,
}

/// This struct contains all the pointers we need to access to all the items in a `Save Template to PackFile` dialog.
#[derive(Debug)]
pub struct SaveTemplateUI {
    pub sections_tableview: QBox<QTableView>,
    pub sections_model: QBox<QStandardItemModel>,
    pub sections_add_button: QBox<QPushButton>,
    pub sections_remove_button: QBox<QPushButton>,

    pub options_tableview: QBox<QTableView>,
    pub options_model: QBox<QStandardItemModel>,
    pub options_add_button: QBox<QPushButton>,
    pub options_remove_button: QBox<QPushButton>,

    pub params_tableview: QBox<QTableView>,
    pub params_model: QBox<QStandardItemModel>,
    pub params_add_button: QBox<QPushButton>,
    pub params_remove_button: QBox<QPushButton>,

    pub info_name_line_edit: QBox<QLineEdit>,
    pub info_description_line_edit: QBox<QLineEdit>,
    pub info_author_line_edit: QBox<QLineEdit>,
    pub info_post_message_line_edit: QBox<QTextEdit>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `TemplateUI`.
impl TemplateUI {

    /// This function creates the entire "Load Template" dialog. It returns a vector with the stuff set in it.
    pub unsafe fn load(
        template: &Template,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>
    ) -> Option<(Vec<(String, bool)>, Vec<(String, String)>)> {

        let wazard = QWizard::new_1a(&app_ui.main_window);
        wazard.set_option_2a(WizardOption::IndependentPages, true);
        wazard.set_window_title(&qtr("load_templates_dialog_title"));
        wazard.set_modal(true);

        let ui = Rc::new(Self {
            template: Rc::new(template.clone()),
            options: Rc::new(RefCell::new(vec![])),
            params: Rc::new(RefCell::new(vec![])),

            wazard
        });

        // Load the initial info page.
        let info_page = Self::load_info_section(&ui);
        ui.wazard.add_page(&info_page);

        // Load the named sections, one per page.
        for section in ui.template.get_sections() {
            let page = Self::load_section(&ui, &section, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui);
            ui.wazard.add_page(&page);
        }

        // Load the sectionless items.
        let page = Self::load_post_section(&ui);
        ui.wazard.add_page(&page);

        // Slots and connections.
        let slots = slots::TemplateUISlots::new(&ui);
        connections::set_connections_template(&ui, &slots);

        ui.update_template_view();

        // Execute the wazard.
        if ui.wazard.exec() == 1 {
            Some(ui.get_data_from_view())
        }

        // Otherwise, return None.
        else { None }
    }

    /// This function loads the info section into the view.
    ///
    /// This section is usually static, so no complex stuff here.
    unsafe fn load_info_section(ui: &Rc<Self>) -> QBox<QWizardPage> {

        let page = QWizardPage::new_1a(&ui.wazard);
        let grid = create_grid_layout(page.static_upcast());
        page.set_title(&QString::from_std_str("By: ".to_owned() + &ui.template.author));

        let description_label = QLabel::from_q_string_q_widget(&QString::from_std_str(&ui.template.description), &page);

        grid.add_widget_5a(&description_label, 0, 0, 1, 2);

        page
    }

    /// This function loads the info section into the view.
    ///
    /// This section is usually static, so no complex stuff here.
    unsafe fn load_section(ui: &Rc<Self>, section: &TemplateSection,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>
    ) -> QBox<QWizardPage> {

        let page = QWizardPage::new_1a(&ui.wazard);
        let grid = create_grid_layout(page.static_upcast());
        page.set_title(&QString::from_std_str(section.get_ref_name()));

        let description_label = QLabel::from_q_string_q_widget(&QString::from_std_str(section.get_ref_description()), &page);
        grid.add_widget_5a(&description_label, 0, 0, 1, 2);

        let column = 0;
        let mut count = 1;
        ui.template.get_options().iter().filter(|x| x.get_ref_section() == section.get_ref_key()).for_each(|z| {
            let field = Self::load_option_data(ui, z);
            grid.add_widget_5a(&field, count as i32, column, 1, 1);
            count += 1;
        });

        count += 2;
        ui.template.get_params().iter().filter(|x| x.get_ref_section() == section.get_ref_key()).for_each(|z| {
            let field = Self::load_field_data(ui, z, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui);
            grid.add_widget_5a(&field, count as i32, column, 1, 1);
            count += 1;
        });

        page
    }

    /// This function loads the info section into the view.
    ///
    /// This section is usually static, so no complex stuff here.
    unsafe fn load_post_section(ui: &Rc<Self>) -> QBox<QWizardPage> {

        let page = QWizardPage::new_1a(&ui.wazard);
        let grid = create_grid_layout(page.static_upcast());

        let post_message_label = QLabel::from_q_string_q_widget(&QString::from_std_str(&ui.template.post_message), &page);
        let final_message_label = QLabel::from_q_string_q_widget(&qtr("template_load_final_message"), &page);

        grid.add_widget_5a(&post_message_label, 0, 0, 1, 1);
        grid.add_widget_5a(&final_message_label, 1, 0, 1, 1);

        page
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

    unsafe fn load_field_data(ui: &Rc<Self>,
        param: &TemplateParam,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>
    ) -> QBox<QWidget> {

        let widget = QWidget::new_0a();
        let grid = create_grid_layout(widget.static_upcast());

        let is_required = if *param.get_ref_is_required() { "*" } else { "" };
        let name = param.get_ref_name().to_owned() + is_required;
        let label = QLabel::from_q_string_q_widget(&QString::from_std_str(&name), &widget);
        label.set_minimum_width(200);
        label.set_maximum_width(200);

        match param.get_ref_param_type() {
            ParamType::Checkbox => {
                let field_widget = QCheckBox::from_q_widget(&widget);
                field_widget.set_minimum_width(250);
                grid.add_widget_5a(&label, 0, 0, 1, 1);
                grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
            }
            ParamType::Integer => {
                let field_widget = QSpinBox::new_1a(&widget);
                field_widget.set_minimum_width(250);
                grid.add_widget_5a(&label, 0, 0, 1, 1);
                grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
            }
            ParamType::Float => {
                let field_widget = QDoubleSpinBox::new_1a(&widget);
                field_widget.set_minimum_width(250);
                grid.add_widget_5a(&label, 0, 0, 1, 1);
                grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
            }
            ParamType::Text => {
                let field_widget = QLineEdit::from_q_widget(&widget);
                field_widget.set_minimum_width(250);
                field_widget.set_maximum_width(250);
                grid.add_widget_5a(&label, 0, 0, 1, 1);
                grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
            }

            ParamType::TableField((table_name, field)) => {
                let mut definition = Definition::new(-100);
                *definition.get_ref_mut_fields() = vec![field.clone()];

                match field.get_field_type() {

                    FieldType::Boolean => {
                        let field_widget = QCheckBox::from_q_widget(&widget);

                        let check_state = if let Some(default_value) = field.get_default_value() {
                            default_value.to_lowercase() == "true"
                        } else { false };

                        if check_state {
                            field_widget.set_check_state(CheckState::Checked);
                        } else {
                            field_widget.set_check_state(CheckState::Unchecked);
                        }

                        field_widget.set_minimum_width(250);
                        grid.add_widget_5a(&label, 0, 0, 1, 1);
                        grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                        ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
                    }
                    FieldType::I16 |
                    FieldType::I32 |
                    FieldType::I64 => {
                        let field_widget = QSpinBox::new_1a(&widget);

                        let data = if let Some(default_value) = field.get_default_value() {
                            if let Ok(default_value) = default_value.parse::<i32>() {
                                default_value
                            } else {
                                0i32
                            }
                        } else {
                            0i32
                        };
                        field_widget.set_value(data);

                        field_widget.set_minimum_width(250);
                        grid.add_widget_5a(&label, 0, 0, 1, 1);
                        grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                        ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
                    }
                    FieldType::F32 => {
                        let field_widget = QDoubleSpinBox::new_1a(&widget);

                        let data = if let Some(default_value) = field.get_default_value() {
                            if let Ok(default_value) = default_value.parse::<f32>() {
                                default_value
                            } else {
                                0.0f32
                            }
                        } else {
                            0.0f32
                        };

                        field_widget.set_value(data.into());

                        field_widget.set_minimum_width(250);
                        grid.add_widget_5a(&label, 0, 0, 1, 1);
                        grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                        ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
                    }

                    FieldType::StringU8 |
                    FieldType::StringU16 |
                    FieldType::OptionalStringU8 |
                    FieldType::OptionalStringU16 => {
                        let ref_data = get_reference_data(&(table_name.to_owned() + field.get_name()), &definition);
                        match ref_data {
                            Ok(ref_data) => {
                                if ref_data.is_empty() || ref_data.get(&0).unwrap().data.is_empty() {
                                    let field_widget = QLineEdit::from_q_widget(&widget);

                                    let text = if let Some(default_value) = field.get_default_value() {
                                        default_value.to_owned()
                                    } else {
                                        String::new()
                                    };
                                    field_widget.set_text(&QString::from_std_str(&text));
                                    field_widget.set_minimum_width(250);
                                    grid.add_widget_5a(&label, 0, 0, 1, 1);
                                    grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                                    ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
                                }
                                else {

                                    let field_widget = QComboBox::new_1a(&widget);
                                    field_widget.set_editable(true);
                                    field_widget.set_minimum_width(250);
                                    field_widget.set_maximum_width(250);
                                    grid.add_widget_5a(&label, 0, 0, 1, 1);
                                    grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                                    ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));

                                    for ref_data in ref_data.get(&0).unwrap().data.keys() {
                                        field_widget.add_item_q_string(&QString::from_std_str(ref_data));
                                    }
                                }
                            }
                            Err(_) => {
                                let field_widget = QLineEdit::from_q_widget(&widget);

                                let text = if let Some(default_value) = field.get_default_value() {
                                    default_value.to_owned()
                                } else {
                                    String::new()
                                };
                                field_widget.set_text(&QString::from_std_str(&text));

                                field_widget.set_minimum_width(250);
                                grid.add_widget_5a(&label, 0, 0, 1, 1);
                                grid.add_widget_5a(&field_widget, 0, 1, 1, 1);
                                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), field_widget.static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
                            }
                        }
                    }

                    FieldType::SequenceU16(_) |
                    FieldType::SequenceU32(_) => unimplemented!()
                }
            }

            // These are semi-full tables, without cells referencing params.
            ParamType::Table(definition) => {
                let table_data = TableType::NormalTable(Table::new(definition));
                let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();
                ui.params.borrow_mut().push((param.get_ref_key().to_owned(), table_view.get_mut_ptr_table_view_primary().static_upcast(), param.get_ref_param_type().clone(), *param.get_ref_is_required()));
            }
        }

        widget
    }

    /// This function returns the options/parameters from the view.
    pub unsafe fn get_data_from_view(&self) -> (Vec<(String, bool)>, Vec<(String, String)>) {
        let options = self.options.borrow().iter().map(|(key, widget)| (key.to_owned(), widget.is_checked())).collect();

        let params = self.params.borrow().iter().map(|(key, widget, _, _)| (key.to_owned(),
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
        let options_enabled = self.options.borrow().iter().filter_map(|(x, y)|
            if y.is_checked() { Some(x.to_owned()) } else { None }
        ).collect::<Vec<String>>();

        // Check all sections that are enable/disabled as expected.
        for template_section in self.template.get_sections() {
            for template_option in self.template.get_options() {
                if template_option.get_ref_section() == template_section.get_ref_key() {
                    if let Some(option) = self.options.borrow().iter().find(|(x, _)| x == template_option.get_ref_key()) {
                        if template_section.has_required_options(&options_enabled) && template_option.has_required_options(&options_enabled) {
                            option.1.set_enabled(true);
                        } else {
                            option.1.set_enabled(false);
                        }
                    }
                }
            }

            for template_param in self.template.get_params() {
                if template_param.get_ref_section() == template_section.get_ref_key() {
                    if let Some((_, widget, _, _)) = self.params.borrow().iter().find(|(param_key, _, _, _)| param_key == template_param.get_ref_key()) {
                        if template_section.has_required_options(&options_enabled) && template_param.has_required_options(&options_enabled) {
                            widget.set_enabled(true);
                        } else {
                            widget.set_enabled(false);
                        }
                    }
                }
            }
        }
    }

    /// This function updates the state of the UI when we enable/disable parts of the template.
    pub unsafe fn check_required_fields(&self) {

        // Check all params that are enable/disabled as expected.
        let mut are_required_params_fulfilled = true;
        for template_param in self.template.get_params() {
            if let Some((_, widget, param_type, is_required)) = self.params.borrow().iter().find(|(param_key, _, _, _)| param_key == template_param.get_ref_key()) {
                if *is_required {

                    match param_type {
                        ParamType::Checkbox => continue,
                        ParamType::Integer => { are_required_params_fulfilled &= !widget.static_downcast::<QSpinBox>().text().to_std_string().is_empty(); },
                        ParamType::Float => { are_required_params_fulfilled &= !widget.static_downcast::<QDoubleSpinBox>().text().to_std_string().is_empty(); },
                        ParamType::Text => { are_required_params_fulfilled &= !widget.static_downcast::<QLineEdit>().text().to_std_string().is_empty(); },

                        // For these types, first ensure what type of field do we have!!!!
                        ParamType::TableField(_) => {
                            if !widget.dynamic_cast::<QComboBox>().is_null() {
                               are_required_params_fulfilled &= !widget.static_downcast::<QComboBox>().current_text().to_std_string().is_empty();
                            } else if !widget.dynamic_cast::<QLineEdit>().is_null() {
                                are_required_params_fulfilled &= !widget.static_downcast::<QLineEdit>().text().to_std_string().is_empty();
                            } else if !widget.dynamic_cast::<QCheckBox>().is_null() {
                                are_required_params_fulfilled &= !widget.static_downcast::<QCheckBox>().is_checked();
                            }

                            // The rest of the types cannot be required, so we skip them.
                            else if !widget.dynamic_cast::<QSpinBox>().is_null() {
                            } else if !widget.dynamic_cast::<QDoubleSpinBox>().is_null() {
                            } else if !widget.dynamic_cast::<QTableView>().is_null() {
                            } else {
                                unimplemented!()
                            };
                        },
                        ParamType::Table(_) => {},
                    }
                }
            }
        }

        // Check that the finish button is enabled/disabled as expected.
        let finish_button = self.wazard.button(WizardButton::FinishButton);
        if are_required_params_fulfilled && finish_button.is_visible() {
            finish_button.set_enabled(true);
        } else {
            finish_button.set_enabled(false);
        }
    }
}

/// Implamentation of `SaveTemplateUI`.
impl SaveTemplateUI {

    /// This function creates the "New Template" dialog when saving the currently open PackFile into a Template.
    ///
    /// It returns the new name of the Template, or `None` if the dialog is canceled or closed.
    pub unsafe fn load(app_ui: &Rc<AppUI>) -> Option<Template> {

        // Create and configure the dialog.
        let wazard: QBox<QWizard> = QWizard::new_1a(&app_ui.main_window);
        wazard.set_window_title(&qtr("save_template"));
        wazard.set_modal(true);
        wazard.resize_2a(1000, 700);

        //-----------------------------------------//
        // Info.
        //-----------------------------------------//
        let info_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_info"), &wazard);
        let info_grid = create_grid_layout(info_groupbox.static_upcast());
        info_groupbox.set_minimum_width(300);

        let info_description = QLabel::from_q_string_q_widget(&qtr("new_template_info_description"), &info_groupbox);
        info_description.set_word_wrap(true);

        let info_name_label = QLabel::from_q_string_q_widget(&qtr("template_name"), &info_groupbox);
        let info_name_line_edit = QLineEdit::from_q_widget(&info_groupbox);

        let info_description_label = QLabel::from_q_string_q_widget(&qtr("template_description"), &info_groupbox);
        let info_description_line_edit = QLineEdit::from_q_widget(&info_groupbox);

        let info_author_label = QLabel::from_q_string_q_widget(&qtr("template_author"), &info_groupbox);
        let info_author_line_edit = QLineEdit::from_q_widget(&info_groupbox);

        let info_post_message_label = QLabel::from_q_string_q_widget(&qtr("template_post_message"), &info_groupbox);
        let info_post_message_line_edit = QTextEdit::from_q_widget(&info_groupbox);

        info_grid.add_widget_5a(&info_description, 0, 0, 1, 2);
        info_grid.add_widget_5a(&info_name_label, 1, 0, 1, 1);
        info_grid.add_widget_5a(&info_name_line_edit, 1, 1, 1, 1);
        info_grid.add_widget_5a(&info_description_label, 2, 0, 1, 1);
        info_grid.add_widget_5a(&info_description_line_edit, 2, 1, 1, 1);
        info_grid.add_widget_5a(&info_author_label, 3, 0, 1, 1);
        info_grid.add_widget_5a(&info_author_line_edit, 3, 1, 1, 1);
        info_grid.add_widget_5a(&info_post_message_label, 4, 0, 1, 1);
        info_grid.add_widget_5a(&info_post_message_line_edit, 4, 1, 1, 1);
        info_grid.set_row_stretch(1, 99);

        let info_page = QWizardPage::new_1a(&wazard);
        let info_grid = create_grid_layout(info_page.static_upcast());
        info_grid.set_contents_margins_4a(4, 0, 4, 4);
        info_grid.set_spacing(4);
        info_grid.add_widget_5a(&info_groupbox, 0, 0, 1, 1);

        wazard.add_page(&info_page);

        //-----------------------------------------//
        // Sections.
        //-----------------------------------------//
        let sections_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_sections"), &wazard);
        let sections_grid = create_grid_layout(sections_groupbox.static_upcast());
        sections_groupbox.set_minimum_width(300);

        let sections_description = QLabel::from_q_string_q_widget(&qtr("new_template_sections_description"), &sections_groupbox);
        let sections_tableview = QTableView::new_1a(&sections_groupbox);
        let sections_model = QStandardItemModel::new_1a(&sections_tableview);
        let sections_add_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &sections_groupbox);
        let sections_remove_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &sections_groupbox);
        sections_tableview.set_model(&sections_model);
        sections_tableview.horizontal_header().set_stretch_last_section(true);
        sections_description.set_word_wrap(true);

        sections_grid.add_widget_5a(&sections_description, 0, 0, 1, 2);
        sections_grid.add_widget_5a(&sections_tableview, 1, 0, 1, 2);
        sections_grid.add_widget_5a(&sections_add_button, 2, 0, 1, 1);
        sections_grid.add_widget_5a(&sections_remove_button, 2, 1, 1, 1);
        sections_grid.set_row_stretch(1, 99);

        let sections_page = QWizardPage::new_1a(&wazard);
        let sections_grid = create_grid_layout(sections_page.static_upcast());
        sections_grid.set_contents_margins_4a(4, 0, 4, 4);
        sections_grid.set_spacing(4);
        sections_grid.add_widget_5a(&sections_groupbox, 0, 0, 1, 1);

        wazard.add_page(&sections_page);

        //-----------------------------------------//
        // Options.
        //-----------------------------------------//
        let options_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_options"), &wazard);
        let options_grid = create_grid_layout(options_groupbox.static_upcast());
        options_groupbox.set_minimum_width(450);

        let options_description = QLabel::from_q_string_q_widget(&qtr("new_template_options_description"), &options_groupbox);
        let options_tableview = QTableView::new_1a(&options_groupbox);
        let options_model = QStandardItemModel::new_1a(&options_tableview);
        let options_add_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &options_groupbox);
        let options_remove_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &options_groupbox);
        options_tableview.set_model(&options_model);
        options_tableview.horizontal_header().set_stretch_last_section(true);
        options_description.set_word_wrap(true);

        options_grid.add_widget_5a(&options_description, 0, 0, 1, 2);
        options_grid.add_widget_5a(&options_tableview, 1, 0, 1, 2);
        options_grid.add_widget_5a(&options_add_button, 2, 0, 1, 1);
        options_grid.add_widget_5a(&options_remove_button, 2, 1, 1, 1);
        options_grid.set_row_stretch(1, 99);

        let options_page = QWizardPage::new_1a(&wazard);
        let options_grid = create_grid_layout(options_page.static_upcast());
        options_grid.set_contents_margins_4a(4, 0, 4, 4);
        options_grid.set_spacing(4);
        options_grid.add_widget_5a(&options_groupbox, 0, 0, 1, 1);

        wazard.add_page(&options_page);

        //-----------------------------------------//
        // Parameters.
        //-----------------------------------------//
        let params_groupbox = QGroupBox::from_q_string_q_widget(&qtr("new_template_params"), &wazard);
        let params_grid = create_grid_layout(params_groupbox.static_upcast());
        params_groupbox.set_minimum_width(450);

        let params_description_label = QLabel::from_q_string_q_widget(&qtr("new_template_params_description"), &params_groupbox);
        let params_tableview = QTableView::new_1a(&params_groupbox);
        let params_model = QStandardItemModel::new_1a(&params_tableview);
        let params_add_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("+"), &params_groupbox);
        let params_remove_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("-"), &params_groupbox);
        params_tableview.set_model(&params_model);
        params_tableview.horizontal_header().set_stretch_last_section(true);
        params_description_label.set_word_wrap(true);

        params_grid.add_widget_5a(&params_description_label, 0, 0, 1, 2);
        params_grid.add_widget_5a(&params_tableview, 1, 0, 1, 2);
        params_grid.add_widget_5a(&params_add_button, 2, 0, 1, 1);
        params_grid.add_widget_5a(&params_remove_button, 2, 1, 1, 1);
        params_grid.set_row_stretch(1, 99);

        let params_page = QWizardPage::new_1a(&wazard);
        let params_grid = create_grid_layout(params_page.static_upcast());
        params_grid.set_contents_margins_4a(4, 0, 4, 4);
        params_grid.set_spacing(4);
        params_grid.add_widget_5a(&params_groupbox, 0, 0, 1, 1);

        wazard.add_page(&params_page);

        //-----------------------------------------//
        // Finishing layouts and execution.
        //-----------------------------------------//

        let ui = Rc::new(Self{
            sections_tableview,
            sections_model,
            sections_add_button,
            sections_remove_button,

            options_tableview,
            options_model,
            options_add_button,
            options_remove_button,

            params_tableview,
            params_model,
            params_add_button,
            params_remove_button,

            info_name_line_edit,
            info_description_line_edit,
            info_author_line_edit,
            info_post_message_line_edit,
        });

        ui.populate_template_view();

        let slots = slots::SaveTemplateUISlots::new(&ui);
        connections::set_connections_save_template(&ui, &slots);

        if wazard.exec() == 1 {
            ui.get_data_from_view()
        } else { None }
    }

    /// This function updates the state of the UI when we enable/disable parts of the template.
    pub unsafe fn populate_template_view(&self) {

        // First, get the definition list from the backend.
        CENTRAL_COMMAND.send_message_qt(Command::GetDefinitionList);
        let response = CENTRAL_COMMAND.recv_message_qt();
        let definitions = match response {
            Response::VecStringDefinition(definitions) => definitions,
            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
        };

        // If there are definitions, use them to fill the sections/params views.
        if !definitions.is_empty() {
            let mut already_added = vec![];
            for (index, (name, definition)) in definitions.iter().enumerate() {
                let section = format!("section_{}_{}", index, name);
                let qlist_boi = QListOfQStandardItem::new();
                let key = QStandardItem::from_q_string(&QString::from_std_str(&section));
                let value = QStandardItem::from_q_string(&QString::from_std_str(&name));
                let required_options = QStandardItem::new();
                let description = QStandardItem::new();

                qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&required_options.into_ptr().as_mut_raw_ptr());
                qlist_boi.append_q_standard_item(&description.into_ptr().as_mut_raw_ptr());
                self.sections_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

                for field in definition.get_ref_fields() {

                    // When adding fields that reference each other, make sure to only add one to the list!!!
                    // NOTE: this is not accurate. We need a better way of doing this.
                    if let Some(ref_data) = field.get_is_reference() {
                        if already_added.contains(ref_data) {
                            continue;
                        }
                    }

                    let qlist_boi = QListOfQStandardItem::new();
                    let key = QStandardItem::from_q_string(&QString::from_std_str(name.to_owned() + field.get_name()));
                    let value = QStandardItem::from_q_string(&QString::from_std_str(field.get_name()));
                    let section = QStandardItem::from_q_string(&QString::from_std_str(&section));
                    let param_type = QStandardItem::from_q_string(&QString::from_std_str(serde_json::to_string(&ParamType::TableField((name.to_owned(), field.clone()))).unwrap()));
                    let is_required = QStandardItem::new();
                    is_required.set_checkable(true);
                    is_required.set_check_state(CheckState::Checked);

                    qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&is_required.into_ptr().as_mut_raw_ptr());
                    qlist_boi.append_q_standard_item(&param_type.into_ptr().as_mut_raw_ptr());
                    self.params_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

                    // Remove the tables in the table.
                    if name.len() > 7 {
                        already_added.push((name.to_owned().drain(..name.len() - 7).collect(), field.get_name().to_owned()));
                    }
                }
            }
        }

        // Otherwise, leave them empty.
        else {
            let qlist_boi = QListOfQStandardItem::new();
            let key = QStandardItem::new();
            let value = QStandardItem::new();
            let required_options = QStandardItem::new();
            let description = QStandardItem::new();

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&required_options.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&description.into_ptr().as_mut_raw_ptr());
            self.sections_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

            let qlist_boi = QListOfQStandardItem::new();
            let key = QStandardItem::new();
            let value = QStandardItem::new();
            let section = QStandardItem::new();
            let param_type = QStandardItem::new();
            let is_required = QStandardItem::new();
            is_required.set_checkable(true);

            qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&is_required.into_ptr().as_mut_raw_ptr());
            qlist_boi.append_q_standard_item(&param_type.into_ptr().as_mut_raw_ptr());
            self.params_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
        }

        let qlist_boi = QListOfQStandardItem::new();
        let key = QStandardItem::new();
        let value = QStandardItem::new();
        let section = QStandardItem::new();

        qlist_boi.append_q_standard_item(&key.into_ptr().as_mut_raw_ptr());
        qlist_boi.append_q_standard_item(&value.into_ptr().as_mut_raw_ptr());
        qlist_boi.append_q_standard_item(&section.into_ptr().as_mut_raw_ptr());
        self.options_model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());

        let sections_columm_1 = QStandardItem::from_q_string(&qtr("key"));
        let sections_columm_2 = QStandardItem::from_q_string(&qtr("name"));
        let sections_columm_3 = QStandardItem::from_q_string(&qtr("required_options"));
        let sections_columm_4 = QStandardItem::from_q_string(&qtr("description"));
        self.sections_model.set_horizontal_header_item(0, sections_columm_1.into_ptr());
        self.sections_model.set_horizontal_header_item(1, sections_columm_2.into_ptr());
        self.sections_model.set_horizontal_header_item(2, sections_columm_3.into_ptr());
        self.sections_model.set_horizontal_header_item(2, sections_columm_4.into_ptr());

        let options_columm_1 = QStandardItem::from_q_string(&qtr("key"));
        let options_columm_2 = QStandardItem::from_q_string(&qtr("name"));
        let options_columm_3 = QStandardItem::from_q_string(&qtr("section"));
        self.options_model.set_horizontal_header_item(0, options_columm_1.into_ptr());
        self.options_model.set_horizontal_header_item(1, options_columm_2.into_ptr());
        self.options_model.set_horizontal_header_item(2, options_columm_3.into_ptr());

        let params_columm_1 = QStandardItem::from_q_string(&qtr("key"));
        let params_columm_2 = QStandardItem::from_q_string(&qtr("name"));
        let params_columm_3 = QStandardItem::from_q_string(&qtr("section"));
        let params_columm_4 = QStandardItem::from_q_string(&qtr("is_required"));
        let params_columm_5 = QStandardItem::from_q_string(&qtr("param_type"));
        self.params_model.set_horizontal_header_item(0, params_columm_1.into_ptr());
        self.params_model.set_horizontal_header_item(1, params_columm_2.into_ptr());
        self.params_model.set_horizontal_header_item(2, params_columm_3.into_ptr());
        self.params_model.set_horizontal_header_item(3, params_columm_4.into_ptr());
        self.params_model.set_horizontal_header_item(4, params_columm_5.into_ptr());

        self.sections_tableview.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        self.options_tableview.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
        self.params_tableview.horizontal_header().resize_sections(ResizeMode::ResizeToContents);
    }

    /// This function returns the options/parameters from the view.
    pub unsafe fn get_data_from_view(&self) -> Option<Template> {

        // Get all the data from the view.
        let name = self.info_name_line_edit.text().to_std_string();
        let description = self.info_description_line_edit.text().to_std_string();
        let author = self.info_author_line_edit.text().to_std_string();
        let post_message = self.info_post_message_line_edit.to_plain_text().to_std_string();

        let mut sections = vec![];
        for row in 0..self.sections_model.row_count_0a() {
            let key = self.sections_model.item_2a(row, 0).text().to_std_string();
            let name = self.sections_model.item_2a(row, 1).text().to_std_string();
            let description = self.sections_model.item_2a(row, 3).text().to_std_string();

            let required_options_str = self.sections_model.item_2a(row, 2).text().to_std_string();
            let required_options = if required_options_str.is_empty() { vec![] } else {
                required_options_str.split(',').map(|x| x.to_owned()).collect::<Vec<String>>()
            };

            if !key.is_empty() && !name.is_empty() {
                sections.push(TemplateSection::new_from_key_name_required_options_description(&key, &name, &required_options, &description));
            }
        }

        let mut options = vec![];
        for row in 0..self.options_model.row_count_0a() {
            let key = self.options_model.item_2a(row, 0).text().to_std_string();
            let name = self.options_model.item_2a(row, 1).text().to_std_string();
            let section = self.options_model.item_2a(row, 2).text().to_std_string();
            if !key.is_empty() && !name.is_empty() {
                options.push(TemplateOption::new_from_key_name_section(&key, &name, &section));
            }
        }

        let mut params = vec![];
        for row in 0..self.params_model.row_count_0a() {
            let key = self.params_model.item_2a(row, 0).text().to_std_string();
            let name = self.params_model.item_2a(row, 1).text().to_std_string();
            let section = self.params_model.item_2a(row, 2).text().to_std_string();
            let is_required = self.params_model.item_2a(row, 3).check_state() == CheckState::Checked;
            let param_type = self.params_model.item_2a(row, 4).text().to_std_string();
            if !key.is_empty() && !name.is_empty() {
                params.push(TemplateParam::new_from_key_name_section_param_type_check_state(&key, &name, &section, &param_type, is_required));
            }
        }

        if !name.is_empty() && !description.is_empty() && !author.is_empty() {
            Some(Template {
                version: 0,
                name,
                description,
                author,
                post_message,
                sections,
                options,
                params,
                dbs: vec![],
                locs: vec![],
                assets: vec![],
            })
        } else { None }
    }

    pub unsafe fn add_empty_row(model: &QBox<QStandardItemModel>) {
        let qlist_boi = QListOfQStandardItem::new();

        for _ in 0..model.column_count_0a() {
            let item = QStandardItem::new();
            qlist_boi.append_q_standard_item(&item.into_ptr().as_mut_raw_ptr());
        }

        model.append_row_q_list_of_q_standard_item(qlist_boi.as_ref());
    }

    pub unsafe fn remove_rows(model: &QBox<QStandardItemModel>, table_view: &QBox<QTableView>) {
        let indexes = table_view.selection_model().selection().indexes();
        let indexes_sorted = (0..indexes.count_0a()).map(|x| indexes.at(x)).collect::<Vec<Ref<QModelIndex>>>();
        let rows_sorted = indexes_sorted.iter().map(|x| x.row()).collect::<Vec<i32>>();

        crate::views::table::utils::delete_rows(&model.static_upcast(), &rows_sorted);
    }
}

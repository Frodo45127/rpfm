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
Module with all the code for managing the ESF Detailed Views.
!*/

use qt_widgets::QCheckBox;
use qt_widgets::QDoubleSpinBox;
use qt_widgets::QLabel;
use qt_widgets::QLineEdit;
use qt_widgets::QGridLayout;
use qt_widgets::QSpinBox;
use qt_widgets::QTreeView;
use qt_widgets::QWidget;

use qt_gui::QStandardItem;
use qt_gui::QStandardItemModel;

use qt_core::QBox;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QSortFilterProxyModel;
use qt_core::QVariant;

use cpp_core::Ptr;
use rpfm_lib::packedfile::esf::{Coordinates2DNode, Coordinates3DNode};

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec;

use rpfm_lib::packedfile::esf::NodeType;
use rpfm_lib::packedfile::table::DecodedData;
use rpfm_lib::packedfile::table::Table;
use rpfm_lib::schema::*;

use crate::AppUI;
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::global_search_ui::GlobalSearchUI;
use crate::packedfile_views::DataSource;
use crate::packedfile_views::esf::esftree::*;
use crate::packedfile_views::PackFileContentsUI;
use crate::utils::create_grid_layout;
use crate::views::table::{*, utils::*};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the detailed view of an ESF Tree Node.
pub struct ESFDetailedView {
    path: Arc<RwLock<Vec<String>>>,
    data_types: Vec<DataType>,
}

/// DataTypes supported by the detailed view.
enum DataType {
    Boolean(QBox<QCheckBox>),
    I8(QBox<QSpinBox>),
    I16(QBox<QSpinBox>),
    I32(QBox<QSpinBox>),
    I64(QBox<QSpinBox>),
    U8(QBox<QSpinBox>),
    U16(QBox<QSpinBox>),
    U32(QBox<QSpinBox>),
    U64(QBox<QSpinBox>),
    F32(QBox<QDoubleSpinBox>),
    F64(QBox<QDoubleSpinBox>),
    Coord2d((QBox<QDoubleSpinBox>, QBox<QDoubleSpinBox>)),
    Coord3d((QBox<QDoubleSpinBox>, QBox<QDoubleSpinBox>, QBox<QDoubleSpinBox>)),
    UTF8(QBox<QLineEdit>),
    UTF16(QBox<QLineEdit>),

    Angle(QBox<QSpinBox>),

    Unknown21(QBox<QSpinBox>),
    Unknown23(QBox<QSpinBox>),
    Unknown25(QBox<QSpinBox>),
    Unknown26(Arc<TableView>),

    BoolArray(Arc<TableView>),
    I8Array(Arc<TableView>),
    I16Array(Arc<TableView>),
    I32Array((Arc<TableView>, bool)),
    I64Array(Arc<TableView>),
    U8Array(Arc<TableView>),
    U16Array(Arc<TableView>),
    U32Array((Arc<TableView>, bool)),
    U64Array(Arc<TableView>),
    F32Array(Arc<TableView>),
    F64Array(Arc<TableView>),
    Coord2dArray(Arc<TableView>),
    Coord3dArray(Arc<TableView>),
    Utf16Array(Arc<TableView>),
    AsciiArray(Arc<TableView>),
    AngleArray(Arc<TableView>),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `ESFDetailedView`.
impl Default for ESFDetailedView {
    fn default() -> Self {
        Self {
            path: Arc::new(RwLock::new(vec![])),
            data_types: vec![],
        }
    }
}

/// Implementation of `ESFDetailedView`.
impl ESFDetailedView {

    /// This function loads the provided subnodes to the detailed TreeView, saving and removing those who were before.
    pub unsafe fn load_subnodes_to_details_view(
        &mut self,
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        parent_widget: &QBox<QWidget>,
        tree_view: &QBox<QTreeView>,
        nodes: &[NodeType],
        item: Ptr<QStandardItem>
    ) {
        let layout: QPtr<QGridLayout> = parent_widget.layout().static_downcast();

        // Save the current data to its node before loading new data.
        self.save_to_tree_node(tree_view);
        while !layout.item_at(0).is_null() {
            let widget = layout.take_at(0).widget();
            widget.delete_later();
        }

        // Reset the detailed view's data.
        self.data_types.clear();

        let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
        let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
        let item_path = <QBox<QTreeView> as ESFTree>::get_path_from_item(item, &model);

        *self.path.write().unwrap() = item_path;

        for (row, node) in nodes.iter().enumerate() {
            match node {
                NodeType::Invalid => unimplemented!(),
                NodeType::Bool(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QCheckBox::from_q_widget(parent_widget);
                    widget.set_checked(*value.get_ref_value());
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Boolean(widget));
                },
                NodeType::I8(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(i8::MAX.into());
                    widget.set_minimum(i8::MIN.into());
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I8(widget));
                },
                NodeType::I16(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(i16::MAX.into());
                    widget.set_minimum(i16::MIN.into());
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I16(widget));
                },
                NodeType::I32(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    widget.set_maximum(i32::MAX);
                    widget.set_minimum(i32::MIN);
                    widget.set_value(*value.get_ref_value());
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I32(widget));
                },
                NodeType::I64(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(i32::MAX);
                    widget.set_minimum(i32::MIN);
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I64(widget));
                },
                NodeType::U8(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(u8::MAX.into());
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U8(widget));
                },
                NodeType::U16(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(u16::MAX.into());
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U16(widget));
                },
                NodeType::U32(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(u32::MAX as i32);
                    widget.set_value(*value.get_ref_value() as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U32(widget));
                },
                NodeType::U64(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_maximum(u32::MAX as i32);
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U64(widget));
                },
                NodeType::F32(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QDoubleSpinBox::new_1a(parent_widget);
                    widget.set_maximum(f32::MAX.into());
                    widget.set_minimum(f32::MIN.into());
                    widget.set_value(*value.get_ref_value() as f64);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::F32(widget));
                },
                NodeType::F64(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QDoubleSpinBox::new_1a(parent_widget);
                    widget.set_maximum(f64::MAX);
                    widget.set_minimum(f64::MIN);
                    widget.set_value(*value as f64);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::F64(widget));
                },
                NodeType::Coord2d(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let widget_layout = create_grid_layout(widget.static_upcast());

                    let x_label = QLabel::from_q_string_q_widget(&QString::from_std_str("X"), &widget);
                    let y_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Y"), &widget);
                    let x_spinbox = QDoubleSpinBox::new_1a(&widget);
                    let y_spinbox = QDoubleSpinBox::new_1a(&widget);

                    x_spinbox.set_value(*value.get_ref_x() as f64);
                    y_spinbox.set_value(*value.get_ref_y() as f64);

                    widget_layout.add_widget_5a(&x_label, 0, 0, 1, 1);
                    widget_layout.add_widget_5a(&y_label, 1, 0, 1, 1);
                    widget_layout.add_widget_5a(&x_spinbox, 0, 1, 1, 1);
                    widget_layout.add_widget_5a(&y_spinbox, 1, 1, 1, 1);

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Coord2d((x_spinbox, y_spinbox)));
                },
                NodeType::Coord3d(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let widget_layout = create_grid_layout(widget.static_upcast());

                    let x_label = QLabel::from_q_string_q_widget(&QString::from_std_str("X"), &widget);
                    let y_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Y"), &widget);
                    let z_label = QLabel::from_q_string_q_widget(&QString::from_std_str("Z"), &widget);
                    let x_spinbox = QDoubleSpinBox::new_1a(&widget);
                    let y_spinbox = QDoubleSpinBox::new_1a(&widget);
                    let z_spinbox = QDoubleSpinBox::new_1a(&widget);

                    x_spinbox.set_value(*value.get_ref_x() as f64);
                    y_spinbox.set_value(*value.get_ref_y() as f64);
                    z_spinbox.set_value(*value.get_ref_z() as f64);

                    widget_layout.add_widget_5a(&x_label, 0, 0, 1, 1);
                    widget_layout.add_widget_5a(&y_label, 1, 0, 1, 1);
                    widget_layout.add_widget_5a(&z_label, 2, 0, 1, 1);
                    widget_layout.add_widget_5a(&x_spinbox, 0, 1, 1, 1);
                    widget_layout.add_widget_5a(&y_spinbox, 1, 1, 1, 1);
                    widget_layout.add_widget_5a(&z_spinbox, 2, 1, 1, 1);

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Coord3d((x_spinbox, y_spinbox, z_spinbox)));
                },
                NodeType::Utf16(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QLineEdit::from_q_widget(parent_widget);
                    widget.set_text(&QString::from_std_str(&value));
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::UTF16(widget));
                },
                NodeType::Ascii(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QLineEdit::from_q_widget(parent_widget);
                    widget.set_text(&QString::from_std_str(&value));
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::UTF8(widget));
                },
                NodeType::Angle(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Angle(widget));
                },
                NodeType::Unknown21(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Unknown21(widget));
                },
                NodeType::Unknown23(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Unknown23(widget));
                },
                NodeType::Unknown25(value) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QSpinBox::new_1a(parent_widget);
                    widget.set_value(*value as i32);
                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Unknown25(widget));
                },
                NodeType::Unknown26(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I32((*x).into())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Unknown26(table_view));
                },
                NodeType::BoolArray(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::Boolean, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::Boolean(*x)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::BoolArray(table_view));
                },
                NodeType::I8Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I16, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I16((*x).into())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I8Array(table_view));
                },
                NodeType::I16Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I16, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I16(*x)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I16Array(table_view));
                },
                NodeType::I32Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.get_ref_value().iter().map(|x| vec![DecodedData::I32(*x)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I32Array((table_view, *values.get_ref_optimized())));
                },
                NodeType::I64Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I64, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I64(*x)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::I64Array(table_view));
                },
                NodeType::U8Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I16, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I16((*x).into())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U8Array(table_view));
                },
                NodeType::U16Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I16, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I16(*x as i16)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U16Array(table_view));
                },
                NodeType::U32Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.get_ref_value().iter().map(|x| vec![DecodedData::I32(*x as i32)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U32Array((table_view, *values.get_ref_optimized())));
                },
                NodeType::U64Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I64, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I64(*x as i64)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::U64Array(table_view));
                },
                NodeType::F32Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::F32(*x)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::F32Array(table_view));
                },
                NodeType::F64Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::F32(*x as f32)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::F64Array(table_view));
                },
                NodeType::Coord2dArray(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let x_field = Field::new("X".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);
                    let y_field = Field::new("Y".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(x_field);
                    definition.get_ref_mut_fields().push(y_field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::F32(*x.get_ref_x()), DecodedData::F32(*x.get_ref_y())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Coord2dArray(table_view));
                },
                NodeType::Coord3dArray(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let x_field = Field::new("X".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);
                    let y_field = Field::new("Y".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);
                    let z_field = Field::new("Z".to_owned(), FieldType::F32, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(x_field);
                    definition.get_ref_mut_fields().push(y_field);
                    definition.get_ref_mut_fields().push(z_field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::F32(*x.get_ref_x()), DecodedData::F32(*x.get_ref_y()), DecodedData::F32(*x.get_ref_z())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Coord3dArray(table_view));
                },
                NodeType::Utf16Array(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::StringU8, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::StringU8(x.to_owned())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::Utf16Array(table_view));
                },
                NodeType::AsciiArray(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::StringU8, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::StringU8(x.to_owned())]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::AsciiArray(table_view));
                },
                NodeType::AngleArray(values) => {
                    let label = QLabel::from_q_string_q_widget(&QString::from_std_str("label"), parent_widget);
                    let widget = QWidget::new_1a(parent_widget);
                    let _ = create_grid_layout(widget.static_upcast());

                    let field = Field::new("Value".to_owned(), FieldType::I16, false, None, false, None, None, None, String::new(), 0, 0, BTreeMap::new(), None);

                    let mut definition = Definition::new(0);
                    definition.get_ref_mut_fields().push(field);

                    let mut table = Table::new(&definition);
                    let _ = table.set_table_data(&values.iter().map(|x| vec![DecodedData::I16(*x)]).collect::<Vec<Vec<DecodedData>>>());

                    let table_data = TableType::NormalTable(table);
                    let table_view = TableView::new_view(&widget, app_ui, global_search_ui, pack_file_contents_ui, diagnostics_ui, dependencies_ui, table_data, None, Arc::new(RwLock::new(DataSource::PackFile))).unwrap();

                    layout.add_widget_5a(&label, row as i32, 0, 1, 1);
                    layout.add_widget_5a(&widget, row as i32, 1, 1, 1);

                    self.data_types.push(DataType::AngleArray(table_view));
                },

                // Skip record nodes.
                NodeType::Record(_) => continue,
            }
        }
    }

    /// This function saves the subnodes of a detailed view into their item in the TreeView.
    pub unsafe fn save_to_tree_node(&self, tree_view: &QBox<QTreeView>) {
        if !self.path.read().unwrap().is_empty() {
            let filter: QPtr<QSortFilterProxyModel> = tree_view.model().static_downcast();
            let model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
            let item = <QBox<QTreeView> as ESFTree>::get_item_from_path(&self.path.read().unwrap(), &model);
            let data = <QBox<QTreeView> as ESFTree>::get_child_nodes_from_item(&item);

            if !data.is_empty() {
                let mut nodes: Vec<NodeType> = serde_json::from_str(&data).unwrap();
                let mut index = 0;
                for node in &mut nodes {
                    match node {
                        NodeType::Invalid => unimplemented!(),
                        NodeType::Bool(value) => if let DataType::Boolean(widget) = &self.data_types[index] {
                            *value.get_ref_mut_value() = widget.is_checked();
                            index += 1;
                        },
                        NodeType::I8(value) => if let DataType::I8(widget) = &self.data_types[index] {
                            *value = widget.value() as i8;
                            index += 1;
                        },
                        NodeType::I16(value) => if let DataType::I16(widget) = &self.data_types[index] {
                            *value = widget.value() as i16;
                            index += 1;
                        },
                        NodeType::I32(value) => if let DataType::I32(widget) = &self.data_types[index] {
                            *value.get_ref_mut_value() = widget.value() as i32;
                            index += 1;
                        },
                        NodeType::I64(value) => if let DataType::I64(widget) = &self.data_types[index] {
                            *value = widget.value() as i64;
                            index += 1;
                        },
                        NodeType::U8(value) => if let DataType::U8(widget) = &self.data_types[index] {
                            *value = widget.value() as u8;
                            index += 1;
                        },
                        NodeType::U16(value) => if let DataType::U16(widget) = &self.data_types[index] {
                            *value = widget.value() as u16;
                            index += 1;
                        },
                        NodeType::U32(value) => if let DataType::U32(widget) = &self.data_types[index] {
                            *value.get_ref_mut_value() = widget.value() as u32;
                            index += 1;
                        },
                        NodeType::U64(value) => if let DataType::U64(widget) = &self.data_types[index] {
                            *value = widget.value() as u64;
                            index += 1;
                        },
                        NodeType::F32(value) => if let DataType::F32(widget) = &self.data_types[index] {
                            *value.get_ref_mut_value() = widget.value() as f32;
                            index += 1;
                        },
                        NodeType::F64(value) => if let DataType::F64(widget) = &self.data_types[index] {
                            *value = widget.value();
                            index += 1;
                        },
                        NodeType::Coord2d(value) => if let DataType::Coord2d((x, y)) = &self.data_types[index] {
                            *value.get_ref_mut_x() = x.value() as f32;
                            *value.get_ref_mut_y() = y.value() as f32;
                            index += 1;
                        },
                        NodeType::Coord3d(value) => if let DataType::Coord3d((x, y, z)) = &self.data_types[index] {
                            *value.get_ref_mut_x() = x.value() as f32;
                            *value.get_ref_mut_y() = y.value() as f32;
                            *value.get_ref_mut_z() = z.value() as f32;
                            index += 1;
                        },
                        NodeType::Utf16(value) => if let DataType::UTF16(widget) = &self.data_types[index] {
                            *value = widget.text().to_std_string();
                            index += 1;
                        },
                        NodeType::Ascii(value) => if let DataType::UTF8(widget) = &self.data_types[index] {
                            *value = widget.text().to_std_string();
                            index += 1;
                        },
                        NodeType::Angle(value) => if let DataType::Angle(widget) = &self.data_types[index] {
                            *value = widget.value() as i16;
                            index += 1;
                        },
                        NodeType::Unknown21(value) => if let DataType::Unknown21(widget) = &self.data_types[index] {
                            *value = widget.value() as u32;
                            index += 1;
                        },
                        NodeType::Unknown23(value) => if let DataType::Unknown23(widget) = &self.data_types[index] {
                            *value = widget.value() as u8;
                            index += 1;
                        },
                        NodeType::Unknown25(value) => if let DataType::Unknown25(widget) = &self.data_types[index] {
                            *value = widget.value() as u32;
                            index += 1;
                        },
                        NodeType::Unknown26(values) => if let DataType::Unknown26(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I32(value) = &x[0] { Some(*value as u8) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::BoolArray(values) => if let DataType::BoolArray(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::Boolean(value) = &x[0] { Some(*value) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::I8Array(values) => if let DataType::I8Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I16(value) = &x[0] { Some(*value as i8) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::I16Array(values) => if let DataType::I16Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I16(value) = &x[0] { Some(*value) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::I32Array(values) => if let DataType::I32Array((table_view, _)) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values.get_ref_mut_value() = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I32(value) = &x[0] { Some(*value) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::I64Array(values) => if let DataType::I64Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I64(value) = &x[0] { Some(*value) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::U8Array(values) => if let DataType::U8Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I16(value) = &x[0] { Some(*value as u8) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::U16Array(values) => if let DataType::U16Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I16(value) = &x[0] { Some(*value as u16) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::U32Array(values) => if let DataType::U32Array((table_view, _)) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values.get_ref_mut_value() = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I32(value) = &x[0] { Some(*value as u32) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::U64Array(values) => if let DataType::U64Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I64(value) = &x[0] { Some(*value as u64) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::F32Array(values) => if let DataType::F32Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::F32(value) = &x[0] { Some(*value) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::F64Array(values) => if let DataType::F64Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::F32(value) = &x[0] { Some(*value as f64) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::Coord2dArray(values) => if let DataType::Coord2dArray(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|row|
                                if let DecodedData::F32(x) = &row[0] {
                                    if let DecodedData::F32(y) = &row[1] {
                                        let mut coords = Coordinates2DNode::default();
                                        *coords.get_ref_mut_x() = *x;
                                        *coords.get_ref_mut_y() = *y;
                                        Some(coords)
                                    } else { None }
                                } else { None }).collect();
                            index += 1;
                        },
                        NodeType::Coord3dArray(values) => if let DataType::Coord3dArray(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|row|
                                if let DecodedData::F32(x) = &row[0] {
                                    if let DecodedData::F32(y) = &row[1] {
                                        if let DecodedData::F32(z) = &row[2] {
                                            let mut coords = Coordinates3DNode::default();
                                            *coords.get_ref_mut_x() = *x;
                                            *coords.get_ref_mut_y() = *y;
                                            *coords.get_ref_mut_z() = *z;
                                            Some(coords)
                                        } else { None }
                                    } else { None }
                                } else { None }).collect();
                            index += 1;
                        },
                        NodeType::Utf16Array(values) => if let DataType::Utf16Array(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::StringU8(value) = &x[0] { Some(value.to_owned()) } else { None }).collect();
                            index += 1;
                        },
                        NodeType::AsciiArray(values) => if let DataType::AsciiArray(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::StringU8(value) = &x[0] { Some(value.to_owned()) } else { None }).collect();
                            index += 1;
                        },

                        NodeType::AngleArray(values) => if let DataType::AngleArray(table_view) = &self.data_types[index] {
                            let filter: QPtr<QSortFilterProxyModel> = table_view.get_mut_ptr_table_view_primary().model().static_downcast();
                            let table_model: QPtr<QStandardItemModel> = filter.source_model().static_downcast();
                            let data = get_table_from_view(&table_model, &table_view.get_ref_table_definition()).unwrap();
                            *values = data.get_ref_table_data().iter().filter_map(|x| if let DecodedData::I16(value) = &x[0] { Some(*value) } else { None }).collect();
                            index += 1;
                        },

                        // Skip record nodes.
                        NodeType::Record(_) => continue,
                    }
                }

                item.set_data_2a(&QVariant::from_q_string(&QString::from_std_str(&serde_json::to_string(&nodes).unwrap())), 42);
            }
        }
    }
}

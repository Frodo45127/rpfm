#include "combobox_item_delegate.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QSettings>
#include <QStandardItem>
#include <QStandardItemModel>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QComboBoxItemDelegate,
// with the specified values. We have to tell it too if the combo will be editable or not.
extern "C" void new_combobox_item_delegate(QObject *parent, const int column, const QStringList* values, const QStringList* lookups, const bool is_editable, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark) {
    QComboBoxItemDelegate* delegate = new QComboBoxItemDelegate(parent, *values, *lookups, is_editable, timer, is_dark_theme_enabled, has_filter, right_side_mark);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of the QComboBoxItemDelegate. We use it to store the values and if the user should be able to write his own value.
QComboBoxItemDelegate::QComboBoxItemDelegate(QObject *parent, const QStringList provided_values, const QStringList provided_lookups, bool is_editable, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark): QExtendedStyledItemDelegate(parent, timer, is_dark_theme_enabled, has_filter, right_side_mark)
{
    editable = is_editable;
    values = provided_values;
    lookups = provided_lookups;
    diag_timer = timer;
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;
    use_right_side_mark = right_side_mark;

    QSettings* q_settings = new QSettings("FrodoWazEre", "rpfm");

    if (dark_theme) {
        colour_table_added = QColor(q_settings->value("colour_dark_table_added").toString());
        colour_table_modified = QColor(q_settings->value("colour_dark_table_modified").toString());
        colour_diagnostic_error = QColor(q_settings->value("colour_dark_diagnostic_error").toString());
        colour_diagnostic_warning = QColor(q_settings->value("colour_dark_diagnostic_warning").toString());
        colour_diagnostic_info = QColor(q_settings->value("colour_dark_diagnostic_info").toString());
    } else {
        colour_table_added = QColor(q_settings->value("colour_light_table_added").toString());
        colour_table_modified = QColor(q_settings->value("colour_light_table_modified").toString());
        colour_diagnostic_error = QColor(q_settings->value("colour_light_diagnostic_error").toString());
        colour_diagnostic_warning = QColor(q_settings->value("colour_light_diagnostic_warning").toString());
        colour_diagnostic_info = QColor(q_settings->value("colour_light_diagnostic_info").toString());
    }
}

// Function called when the combo it's created. It just put the values into the combo and returns it.
QWidget* QComboBoxItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    QComboBox* comboBox = new QComboBox(parent);
    QStandardItemModel* model = new QStandardItemModel(comboBox);
    comboBox->setModel(model);
    comboBox->setEditable(editable);

    if (lookups.isEmpty() || lookups.count() != values.count()) {
        comboBox->addItems(values);
    } else {
        for (int i = 0; i < lookups.count(); ++i) {
            QStandardItem* item = new QStandardItem();
            item->setData(values.at(i), 2);
            item->setData(lookups.at(i), 40);
            model->appendRow(item);
        }

        // Set the same delegate used for tables.
        comboBox->setItemDelegate(new QExtendedStyledItemDelegate(comboBox, nullptr, dark_theme, false, false));
    }

    return comboBox;
}

// Function called after the combo it's created. It just select the default value shown in the combo.
void QComboBoxItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QString value = index.model()->data(index, Qt::EditRole).toString();
    QComboBox* comboBox = static_cast<QComboBox*>(editor);

    // If no item has been found with that text, we add it and select it.
    // This fixes the "the text vanished when I double clicked the cell" bug.
    int pos = comboBox->findText(value);
    if (pos != -1) { comboBox->setCurrentIndex(pos); }
    else {
        comboBox->insertItem(0, value);
        comboBox->setCurrentIndex(0);
    }
}

// Function to be called when we're done. It just takes the selected value and saves it in the Table Model.
void QComboBoxItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QComboBox* comboBox = static_cast<QComboBox*>(editor);
    QString value = comboBox->currentText();
    model->setData(index, value, Qt::EditRole);
}

// Function for the combo to show up properly.
void QComboBoxItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}

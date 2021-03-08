#include "combobox_item_delegate.h"
#include <QDebug>
#include <QAbstractItemView>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QComboBoxItemDelegate,
// with the specified values. We have to tell it too if the combo will be editable or not.
extern "C" void new_combobox_item_delegate(QObject *parent, const int column, const QStringList* values, const bool is_editable, const int max_lenght, QTimer* timer) {
    QComboBoxItemDelegate* delegate = new QComboBoxItemDelegate(parent, *values, is_editable, max_lenght, timer);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of the QComboBoxItemDelegate. We use it to store the values and if the user should be able to write his own value.
QComboBoxItemDelegate::QComboBoxItemDelegate(QObject *parent, const QStringList provided_values, bool is_editable, int lenght, QTimer* timer): QStyledItemDelegate(parent)
{
    editable = is_editable;
    values = provided_values;
    max_lenght = lenght;
    diag_timer = timer;
}

// Function called when the combo it's created. It just put the values into the combo and returns it.
QWidget* QComboBoxItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    QComboBox* comboBox = new QComboBox(parent);
    comboBox->setEditable(editable);
    comboBox->addItems(values);
    if (this->max_lenght > 0) {
        //comboBox->setMaxLength(max_lenght);
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

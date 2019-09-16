#include "spinbox_item_delegate.h"
#include "limits.h"
#include <QDebug>
#include <QTableView>
#include <QSpinBox>
#include <QLineEdit>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QSpinBoxItemDelegate.
// We have to pass it the integer type (32 or 64) too for later checks.
extern "C" void new_spinbox_item_delegate(QObject *parent, const int column, const int integer_type, const bool is_optional) {
    QSpinBoxItemDelegate* delegate = new QSpinBoxItemDelegate(parent, integer_type, is_optional);
    dynamic_cast<QTableView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of QSpinBoxItemDelegate. We use it to store the integer type of the value in the delegate.
QSpinBoxItemDelegate::QSpinBoxItemDelegate(QObject *parent, const int integer_type, const bool is_optional): QStyledItemDelegate(parent)
{
    type = integer_type;
    optional = is_optional;
}

// Function called when the widget it's created. Here we configure the spinbox/linedit.
QWidget* QSpinBoxItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const {

    // SpinBoxes only support i32, not i64, so for i64 we use a linedit with validation.
    if (type == 64) {
        QLineEdit* lineEdit = new QLineEdit(parent);
        return lineEdit;
    }

    // For the rest, we use a normal spinbox.
    else {
        if (optional) {
            QLineEdit* lineEdit = new QLineEdit(parent);
            return lineEdit;
        }
        else {
            QSpinBox* spinBox = new QSpinBox(parent);
            spinBox->setRange(-2147483648, 2147483647);
            return spinBox;
        }
    }
}

// Function called after the spinbox/linedit it's created. It just gives it his initial value (the one currently in the model).
void QSpinBoxItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    if (type == 64) {
        QLineEdit* lineEdit = static_cast<QLineEdit*>(editor);
        QString value = index.model()->data(index, Qt::EditRole).toString();
        lineEdit->setText(value);
    }
    else {
        if (optional) {
            QLineEdit* lineEdit = static_cast<QLineEdit*>(editor);
            QString value = index.model()->data(index, Qt::EditRole).toString();
            lineEdit->setText(value);
        }
        else {
            QSpinBox* spinBox = static_cast<QSpinBox*>(editor);
            signed int value = index.model()->data(index, Qt::EditRole).toInt();
            spinBox->setValue(value);
        }
    }
}

// Function to be called when we're done. It just takes the value in the spinbox/linedit and saves it in the Table Model.
void QSpinBoxItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {

    // For i64, we need to check before that the data is valid. Otherwise, we don't pass it to the model.
    if (type == 64) {
        QLineEdit* lineEdit = static_cast<QLineEdit*>(editor);
        bool ok;
        signed long long value = lineEdit->text().toLongLong(&ok);
        if (ok) { model->setData(index, value, Qt::EditRole); }
    }
    else {
        if (optional) {
            QLineEdit* lineEdit = static_cast<QLineEdit*>(editor);
            bool ok;
            signed int value = lineEdit->text().toInt(&ok);
            if (ok) { model->setData(index, value, Qt::EditRole); }
            else { model->setData(index, "", Qt::EditRole); }
        }
        else {
            QSpinBox* spinBox = static_cast<QSpinBox*>(editor);
            signed int value = spinBox->value();
            model->setData(index, value, Qt::EditRole);
        }
    }
}

// Function for the spinbox to show up properly.
void QSpinBoxItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    editor->setGeometry(option.rect);
}

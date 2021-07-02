#include "spinbox_item_delegate.h"
#include "limits.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QSpinBox>
#include <QLineEdit>
#include <QSettings>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QSpinBoxItemDelegate.
// We have to pass it the integer type (16, 32 or 64) too for later checks.
extern "C" void new_spinbox_item_delegate(QObject *parent, const int column, const int integer_type, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark) {
    QSpinBoxItemDelegate* delegate = new QSpinBoxItemDelegate(parent, integer_type, timer, is_dark_theme_enabled, has_filter, right_side_mark);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of QSpinBoxItemDelegate. We use it to store the integer type of the value in the delegate.
QSpinBoxItemDelegate::QSpinBoxItemDelegate(QObject *parent, const int integer_type, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark): QExtendedStyledItemDelegate(parent)
{
    type = integer_type;
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

// Function called when the widget it's created. Here we configure the spinbox/linedit.
QWidget* QSpinBoxItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    // SpinBoxes only support i16, i32, not i64, so for i64 we use a linedit with validation.
    if (type == 64) {
        QLineEdit* lineEdit = new QLineEdit(parent);
        return lineEdit;
    }

    // For the rest, we use a normal spinbox.
    else {
        QSpinBox* spinBox = new QSpinBox(parent);
        if (type == 32) {
            spinBox->setRange(-2147483648, 2147483647);
        }
        else if (type == 16) {
            spinBox->setRange(-32768, 32767);
        }
        return spinBox;
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
        QSpinBox* spinBox = static_cast<QSpinBox*>(editor);
        signed int value = index.model()->data(index, Qt::EditRole).toInt();
        spinBox->setValue(value);
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
        QSpinBox* spinBox = static_cast<QSpinBox*>(editor);
        signed int value = spinBox->value();
        model->setData(index, value, Qt::EditRole);
    }
}

// Function for the spinbox to show up properly.
void QSpinBoxItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}

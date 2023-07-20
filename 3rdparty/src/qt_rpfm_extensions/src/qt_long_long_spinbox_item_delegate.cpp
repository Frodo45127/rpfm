#include "qt_long_long_spinbox_item_delegate.h"
#include <qt_long_long_spinbox.h>
#include <QDebug>
#include <QAbstractItemView>
#include <QSettings>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QtLongLongSpinBoxItemDelegate.
extern "C" void new_i64_spinbox_item_delegate(QObject *parent, const int column, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark) {
    QtLongLongSpinBoxItemDelegate* delegate = new QtLongLongSpinBoxItemDelegate(parent, timer, is_dark_theme_enabled, has_filter, right_side_mark);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of the QtLongLongSpinBoxItemDelegate. Empty, as we don't need to do anything special with it.
QtLongLongSpinBoxItemDelegate::QtLongLongSpinBoxItemDelegate(QObject *parent, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark): QExtendedStyledItemDelegate(parent) {
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

// Function called when the spinbox it's created. Here we configure the limits and decimals of the spinbox.
QWidget* QtLongLongSpinBoxItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    QtLongLongSpinBox* spinBox = new QtLongLongSpinBox(parent);
    return spinBox;
}

// Function called after the spinbox it's created. It just gives it his initial value (the one currently in the model).
void QtLongLongSpinBoxItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QtLongLongSpinBox* spinBox = static_cast<QtLongLongSpinBox*>(editor);
    qlonglong value = index.model()->data(index, Qt::EditRole).toLongLong();
    spinBox->setValue(value);
}

// Function to be called when we're done. It just takes the value in the spinbox and saves it in the Table Model.
void QtLongLongSpinBoxItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QtLongLongSpinBox* spinBox = static_cast<QtLongLongSpinBox*>(editor);
    qlonglong value = spinBox->value();
    model->setData(index, value, Qt::EditRole);
}

// Function for the spinbox to show up properly.
void QtLongLongSpinBoxItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}

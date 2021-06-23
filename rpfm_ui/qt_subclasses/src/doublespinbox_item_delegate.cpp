#include "doublespinbox_item_delegate.h"
#include "float.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QDoubleSpinBox>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QDoubleSpinBoxItemDelegate.
extern "C" void new_doublespinbox_item_delegate(QObject *parent, const int column, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark) {
    QDoubleSpinBoxItemDelegate* delegate = new QDoubleSpinBoxItemDelegate(parent, timer, is_dark_theme_enabled, has_filter, right_side_mark);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of the QDoubleSpinBoxItemDelegate. Empty, as we don't need to do anything special with it.
QDoubleSpinBoxItemDelegate::QDoubleSpinBoxItemDelegate(QObject *parent, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark): QExtendedStyledItemDelegate(parent) {
    diag_timer = timer;
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;
    use_right_side_mark = right_side_mark;
}

// Function called when the spinbox it's created. Here we configure the limits and decimals of the spinbox.
QWidget* QDoubleSpinBoxItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    QDoubleSpinBox* spinBox = new QDoubleSpinBox(parent);
    spinBox->setRange(-3.402823e+38, 3.402823e+38);
    spinBox->setDecimals(3);
    return spinBox;
}

// Function called after the spinbox it's created. It just gives it his initial value (the one currently in the model).
void QDoubleSpinBoxItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QDoubleSpinBox* spinBox = static_cast<QDoubleSpinBox*>(editor);
    double value = index.model()->data(index, Qt::EditRole).toDouble();
    spinBox->setValue(value);
}

// Function to be called when we're done. It just takes the value in the spinbox and saves it in the Table Model.
void QDoubleSpinBoxItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QDoubleSpinBox* spinBox = static_cast<QDoubleSpinBox*>(editor);
    double value = spinBox->value();
    model->setData(index, value, Qt::EditRole);
}

// Function for the spinbox to show up properly.
void QDoubleSpinBoxItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}

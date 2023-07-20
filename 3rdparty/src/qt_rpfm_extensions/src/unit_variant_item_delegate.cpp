#include "unit_variant_item_delegate.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QSortFilterProxyModel>
#include <QPen>
#include <QColor>
#include <QPainter>
#include <QStandardItem>
#include <QStyle>
#include <QSpinBox>
#include <qt_long_long_spinbox.h>

extern "C" void new_unit_variant_item_delegate(QObject *parent, int column) {
    UnitVariantItemDelegate* delegate = new UnitVariantItemDelegate(parent);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of QExtendedStyledItemDelegate. We use it to store the integer type of the value in the delegate.
UnitVariantItemDelegate::UnitVariantItemDelegate(QObject *parent): QStyledItemDelegate(parent) {}

QWidget* UnitVariantItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &index) const {
    QtLongLongSpinBox* spinBox = new QtLongLongSpinBox(parent);
    spinBox->setMinimum(0);

    if (index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());

        QList<qlonglong> values;

        for (int row = 0; row < standardModel->rowCount(); row++) {
            if (row != index.row()) {
                values.push_back(standardModel->item(row)->data(Qt::EditRole).toLongLong());
            }
        }

        spinBox->setInvalidValues(values);
    }

    return spinBox;
}

// Function called after the spinbox/linedit it's created. It just gives it his initial value (the one currently in the model).
void UnitVariantItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QtLongLongSpinBox* spinBox = static_cast<QtLongLongSpinBox*>(editor);
    qlonglong value = index.model()->data(index, Qt::EditRole).toLongLong();
    spinBox->setValue(value);
}

// Function to be called when we're done. It just takes the value in the spinbox/linedit and saves it in the Table Model.
void UnitVariantItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QtLongLongSpinBox* spinBox = static_cast<QtLongLongSpinBox*>(editor);
    qlonglong value = spinBox->value();
    model->setData(index, value, Qt::EditRole);
}

// Function for the spinbox to show up properly.
void UnitVariantItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}

void UnitVariantItemDelegate::initStyleOption(QStyleOptionViewItem *option, const QModelIndex &index) const {
    QStyledItemDelegate::initStyleOption(option,index);

    if (index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());
        QStandardItem* item = standardModel->itemFromIndex(filterModel->mapToSource(index));

        QVariant subDataVariant = item->data(40);
        QString subData = !subDataVariant.isNull() ? subDataVariant.toString(): QString();

        // Format numbers to have leading zeros if there are too many.
        if (standardModel->rowCount() > 10 && option->text.toInt() < 10) {
            option->text = '0' + option->text ;
        }

        // If we have subdata, put it on the right side of the normal data, as a dimmed text.
        if (!subData.isEmpty()) {
            option->text += QString(" - ") + subData;
        }
    }
}

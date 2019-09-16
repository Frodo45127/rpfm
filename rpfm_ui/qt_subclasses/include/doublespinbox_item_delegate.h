#ifndef DOUBLESPINBOX_ITEM_DELEGATE_H
#define DOUBLESPINBOX_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QDoubleSpinBox>

extern "C" void new_double_spinbox_item_delegate(QObject *parent = 0, const int column = 0);

class QDoubleSpinBoxItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QDoubleSpinBoxItemDelegate(QObject *parent = 0);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
};

#endif // DOUBLESPINBOX_ITEM_DELEGATE_H

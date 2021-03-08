#ifndef SPINBOX_ITEM_DELEGATE_H
#define SPINBOX_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QSpinBox>
#include <QTimer>

extern "C" void new_spinbox_item_delegate(QObject *parent = nullptr, const int column = 0, const int integer_type = 0, const bool is_optional = false, QTimer* timer = nullptr);

class QSpinBoxItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QSpinBoxItemDelegate(QObject *parent = nullptr, const int integer_type = 0, const bool is_optional = false, QTimer* timer = nullptr);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
    int type;
    bool optional;
    QTimer *diag_timer;
};

#endif // SPINBOX_ITEM_DELEGATE_H

#ifndef COMBOBOX_ITEM_DELEGATE_H
#define COMBOBOX_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QComboBox>

extern "C" void new_combobox_item_delegate(QObject *parent = 0, const int column = 0, const QStringList *values = NULL, const bool is_editable = false);

class QComboBoxItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QComboBoxItemDelegate(QObject *parent = 0, const QStringList list = {""}, bool is_editable = false);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
    QStringList values;
    bool editable;
};
#endif // COMBOBOX_ITEM_DELEGATE_H

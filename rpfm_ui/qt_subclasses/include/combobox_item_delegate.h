#ifndef COMBOBOX_ITEM_DELEGATE_H
#define COMBOBOX_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QComboBox>
#include <QTimer>

extern "C" void new_combobox_item_delegate(QObject *parent = nullptr, const int column = 0, const QStringList *values = nullptr, const bool is_editable = false, const int max_lenght = 0, QTimer* timer = nullptr);

class QComboBoxItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QComboBoxItemDelegate(QObject *parent = nullptr, const QStringList list = {""}, bool is_editable = false, int max_lenght = 0, QTimer* timer = nullptr);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
    QStringList values;
    bool editable;
    int max_lenght;
    QTimer* diag_timer;
};
#endif // COMBOBOX_ITEM_DELEGATE_H

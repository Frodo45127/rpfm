#ifndef QSTRING_ITEM_DELEGATE_H
#define QSTRING_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>

extern "C" void new_qstring_item_delegate(QObject *parent = nullptr, const int column = 0, const int max_lenght = 0);

class QStringItemDelegate : public QStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QStringItemDelegate(QObject *parent = nullptr, const int max_lenght = 0);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const;

signals:

private:
    int max_lenght;
};

#endif // QSTRING_ITEM_DELEGATE_H

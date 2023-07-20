#ifndef UNIT_VARIANT_ITEM_DELEGATE_H
#define UNIT_VARIANT_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QTimer>
#include <QColor>

extern "C" void new_generic_item_delegate(QObject *parent = nullptr, const int column = 0);

class UnitVariantItemDelegate : public QStyledItemDelegate {
    Q_OBJECT

public:

    explicit UnitVariantItemDelegate(QObject *parent = nullptr);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;
signals:

protected:
    void initStyleOption(QStyleOptionViewItem *option, const QModelIndex &index) const override;

private:
};

#endif // UNIT_VARIANT_ITEM_DELEGATE_H

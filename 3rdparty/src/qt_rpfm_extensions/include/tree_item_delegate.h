#ifndef TREE_ITEM_DELEGATE_H
#define TREE_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include "extended_q_styled_item_delegate.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QDoubleSpinBox>
#include <QTimer>

extern "C" void new_tree_item_delegate(QObject *parent = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);

class QTreeItemDelegate : public QExtendedStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QTreeItemDelegate(QObject *parent = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

protected:
    QColor colour_tree_added;
    QColor colour_tree_modified;
};

#endif // TREE_ITEM_DELEGATE_H

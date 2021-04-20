#ifndef EXTENDED_Q_STYLED_ITEM_DELEGATE_H
#define EXTENDED_Q_STYLED_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QTimer>

extern "C" void new_generic_item_delegate(QObject *parent = nullptr, const int column = 0, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);

class QExtendedStyledItemDelegate : public QStyledItemDelegate {
Q_OBJECT

public:

    explicit QExtendedStyledItemDelegate(QObject *parent = nullptr, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);
    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

protected:
    bool dark_theme;
    bool use_filter;

private:
    QTimer* diag_timer;
};

#endif // EXTENDED_Q_STYLED_ITEM_DELEGATE_H

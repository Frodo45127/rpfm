#ifndef QSTRING_ITEM_DELEGATE_H
#define QSTRING_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include "extended_q_styled_item_delegate.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QTimer>

extern "C" void new_qstring_item_delegate(QObject *parent = nullptr, const int column = 0, const int max_lenght = 0, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);

class QStringItemDelegate : public QExtendedStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QStringItemDelegate(QObject *parent = nullptr, const int max_lenght = 0, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const;

signals:

private:
    int max_lenght;
    QTimer* diag_timer;
};

#endif // QSTRING_ITEM_DELEGATE_H

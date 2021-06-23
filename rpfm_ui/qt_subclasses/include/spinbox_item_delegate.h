#ifndef SPINBOX_ITEM_DELEGATE_H
#define SPINBOX_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include "extended_q_styled_item_delegate.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QSpinBox>
#include <QTimer>

extern "C" void new_spinbox_item_delegate(QObject *parent = nullptr, const int column = 0, const int integer_type = 0, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false, bool right_side_mark = false);

class QSpinBoxItemDelegate : public QExtendedStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QSpinBoxItemDelegate(QObject *parent = nullptr, const int integer_type = 0, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false, bool right_side_mark = false);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
    int type;
    QTimer *diag_timer;
};

#endif // SPINBOX_ITEM_DELEGATE_H

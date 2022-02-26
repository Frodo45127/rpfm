#ifndef COMBOBOX_ITEM_DELEGATE_H
#define COMBOBOX_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include "extended_q_styled_item_delegate.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QComboBox>
#include <QTimer>

extern "C" void new_combobox_item_delegate(QObject *parent = nullptr, const int column = 0, const QStringList *values = nullptr, const bool is_editable = false, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false, bool right_side_mark = false);

class QComboBoxItemDelegate : public QExtendedStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QComboBoxItemDelegate(QObject *parent = nullptr, const QStringList list = {""}, bool is_editable = false, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false, bool right_side_mark = false);

    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    void setEditorData(QWidget *editor, const QModelIndex &index) const;
    void setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const;
    void updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const;

signals:

private:
    QStringList values;
    bool editable;
    QTimer* diag_timer;
};
#endif // COMBOBOX_ITEM_DELEGATE_H

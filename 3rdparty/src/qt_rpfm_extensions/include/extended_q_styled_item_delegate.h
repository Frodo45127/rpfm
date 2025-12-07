#ifndef EXTENDED_Q_STYLED_ITEM_DELEGATE_H
#define EXTENDED_Q_STYLED_ITEM_DELEGATE_H

#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QTimer>
#include <QColor>

extern "C" void new_generic_item_delegate(QObject *parent = nullptr, const int column = 0, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false, bool right_side_mark = false, bool enable_diff_markers = false);

class QExtendedStyledItemDelegate : public QStyledItemDelegate {
Q_OBJECT

public:

    explicit QExtendedStyledItemDelegate(QObject *parent = nullptr, QTimer* timer = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false, bool right_side_mark = false, bool enable_diff_markers = false);
    QWidget* createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const override;
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override;
signals:

protected:
    bool skipTextPainting;
    bool dark_theme;
    bool use_filter;
    bool use_right_side_mark;
    bool use_diff_markers;
    QColor colour_table_added;
    QColor colour_table_modified;
    QColor colour_diagnostic_error;
    QColor colour_diagnostic_warning;
    QColor colour_diagnostic_info;
    QTimer* diag_timer;

    void initStyleOption(QStyleOptionViewItem *option, const QModelIndex &index) const override;
};

#endif // EXTENDED_Q_STYLED_ITEM_DELEGATE_H

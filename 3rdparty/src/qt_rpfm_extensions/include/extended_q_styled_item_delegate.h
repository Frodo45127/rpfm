#ifndef EXTENDED_Q_STYLED_ITEM_DELEGATE_H
#define EXTENDED_Q_STYLED_ITEM_DELEGATE_H

#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QTimer>
#include <QColor>
#include <QGuiApplication>
#include <QPalette>

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
    mutable bool dark_theme;
    bool use_filter;
    bool use_right_side_mark;
    bool use_diff_markers;
    mutable QColor colour_table_added;
    mutable QColor colour_table_modified;
    mutable QColor colour_diagnostic_error;
    mutable QColor colour_diagnostic_warning;
    mutable QColor colour_diagnostic_info;
    QTimer* diag_timer;

    // Cached palette key to detect theme changes at paint time.
    mutable qint64 cached_palette_key;

    // Re-reads theme colors from QSettings if the system palette has changed.
    void refreshThemeColorsIfNeeded() const;

    void initStyleOption(QStyleOptionViewItem *option, const QModelIndex &index) const override;
};

#endif // EXTENDED_Q_STYLED_ITEM_DELEGATE_H

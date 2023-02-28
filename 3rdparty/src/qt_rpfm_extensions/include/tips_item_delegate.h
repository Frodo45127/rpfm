#ifndef TIPS_ITEM_DELEGATE_H
#define TIPS_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include "extended_q_styled_item_delegate.h"
#include <QStyledItemDelegate>
#include <QAbstractItemDelegate>
#include <QDoubleSpinBox>
#include <QTimer>

extern "C" void new_tips_item_delegate(QObject *parent = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);

class QTipsItemDelegate : public QExtendedStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QTipsItemDelegate(QObject *parent = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const;
    QSize sizeHint(QStyleOptionViewItem const &option, QModelIndex const &index) const;

signals:

protected:
    int d_radius;
    int d_toppadding;
    int d_bottompadding;
    int d_leftpadding;
    int d_rightpadding;
    int d_verticalmargin;
    int d_horizontalmargin;
    int d_pointerwidth;
    int d_pointerheight;
    float d_widthfraction;
    QColor colour = "#FFFFFF";
};
#endif // TIPS_ITEM_DELEGATE_H

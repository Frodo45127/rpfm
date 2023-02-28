#include "tips_item_delegate.h"

#include <QDebug>
#include <QAbstractItemView>
#include <QSortFilterProxyModel>
#include <QPen>
#include <QColor>
#include <QPainter>
#include <QStandardItem>
#include <QStyle>
#include <QSettings>
#include <QPainterPath>
#include <QAbstractTextDocumentLayout>
#include <QTextDocument>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QTreeItemDelegate.
extern "C" void new_tips_item_delegate(QObject *parent, bool is_dark_theme_enabled, bool has_filter) {
    QTipsItemDelegate* delegate = new QTipsItemDelegate(parent, is_dark_theme_enabled, has_filter);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(0, delegate);
}

QTipsItemDelegate::QTipsItemDelegate(QObject *parent, bool is_dark_theme_enabled, bool has_filter): QExtendedStyledItemDelegate(parent) {
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;

    d_radius = 5;
    d_toppadding = 5;
    d_bottompadding = 3;
    d_leftpadding = 5;
    d_rightpadding = 5;
    d_verticalmargin = 15;
    d_horizontalmargin = 10;
    d_pointerwidth = 10;
    d_pointerheight = 17;
    d_widthfraction = 0.7;

    // TODO: Move this to the main stylesheet or palette.
    colour = QColor("#363636");
}

// Function for the delegate to showup properly.
void QTipsItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    if (use_filter && index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());
        QStandardItem* item = standardModel->itemFromIndex(filterModel->mapToSource(index));

        if (item != nullptr) {
            QTextDocument bodydoc;
            QTextOption textOption(bodydoc.defaultTextOption());
            textOption.setWrapMode(QTextOption::WrapAtWordBoundaryOrAnywhere);
            bodydoc.setDefaultTextOption(textOption);

            QString bodytext(item->data(Qt::DisplayRole).toString());
            bodydoc.setHtml(bodytext);

            qreal contentswidth = option.rect.width() * d_widthfraction - d_horizontalmargin - d_pointerwidth - d_leftpadding - d_rightpadding;
            bodydoc.setTextWidth(contentswidth);
            qreal bodyheight = bodydoc.size().height();

            painter->save();
            painter->setRenderHint(QPainter::Antialiasing);
            painter->translate(option.rect.left() + d_horizontalmargin, option.rect.top() + ((item->row() == 0) ? d_verticalmargin : 0));

            // background color for chat bubble
            QColor bgcolor = colour;

            // create chat bubble
            QPainterPath pointie;

            // left bottom
            pointie.moveTo(0, bodyheight + d_toppadding + d_bottompadding);

            // right bottom
            pointie.lineTo(0 + contentswidth + d_pointerwidth + d_leftpadding + d_rightpadding - d_radius,
                           bodyheight + d_toppadding + d_bottompadding);
            pointie.arcTo(0 + contentswidth + d_pointerwidth + d_leftpadding + d_rightpadding - 2 * d_radius,
                          bodyheight + d_toppadding + d_bottompadding - 2 * d_radius,
                          2 * d_radius, 2 * d_radius, 270, 90);

            // right top
            pointie.lineTo(0 + contentswidth + d_pointerwidth + d_leftpadding + d_rightpadding, 0 + d_radius);
            pointie.arcTo(0 + contentswidth + d_pointerwidth + d_leftpadding + d_rightpadding - 2 * d_radius, 0,
                          2 * d_radius, 2 * d_radius, 0, 90);

            // left top
            pointie.lineTo(0 + d_pointerwidth + d_radius, 0);
            pointie.arcTo(0 + d_pointerwidth, 0, 2 * d_radius, 2 * d_radius, 90, 90);

            // left bottom almost (here is the pointie)
            pointie.lineTo(0 + d_pointerwidth, bodyheight + d_toppadding + d_bottompadding - d_pointerheight);
            pointie.closeSubpath();

            // now paint it!
            painter->setPen(QPen(bgcolor));
            painter->drawPath(pointie);
            painter->fillPath(pointie, QBrush(bgcolor));

            // set text color used to draw message body
            QAbstractTextDocumentLayout::PaintContext ctx;

            // draw body text
            painter->translate(d_pointerwidth + d_leftpadding, 0);
            bodydoc.documentLayout()->draw(painter, ctx);

            painter->restore();
        }
    }
}


QSize QTipsItemDelegate::sizeHint(QStyleOptionViewItem const &option, QModelIndex const &index) const {
    if (use_filter && index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());
        QStandardItem* item = standardModel->itemFromIndex(filterModel->mapToSource(index));

        if (item != nullptr) {
            QTextDocument bodydoc;
            QTextOption textOption(bodydoc.defaultTextOption());
            textOption.setWrapMode(QTextOption::WrapAtWordBoundaryOrAnywhere);
            bodydoc.setDefaultTextOption(textOption);

            QString bodytext(item->data(Qt::DisplayRole).toString());
            bodydoc.setHtml(bodytext);

            // the width of the contents are the (a fraction of the window width) minus (margins + padding + width of the bubble's tail)
            qreal contentswidth = option.rect.width() * d_widthfraction - d_horizontalmargin - d_pointerwidth - d_leftpadding - d_rightpadding;

            // set this available width on the text document
            bodydoc.setTextWidth(contentswidth);

            QSize size(
                bodydoc.idealWidth() + d_horizontalmargin + d_pointerwidth + d_leftpadding + d_rightpadding,
                bodydoc.size().height() + d_bottompadding + d_toppadding + d_verticalmargin + 1
            ); // I dont remember why +1, haha, might not be necessary

            if (item->row() == 0) // have extra margin at top of first item
                size += QSize(0, d_verticalmargin);

            return size;
        } else {
            return QSize(0, 0);
        }
    } else {
        return QSize(0, 0);
    }
}

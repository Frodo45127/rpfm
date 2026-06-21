#include "filter_header_view.h"

#include <QMouseEvent>
#include <QPainter>
#include <QPalette>
#include <QStyle>
#include <QStyleOptionHeader>

namespace {
    // Size of the funnel glyph, in header pixels.
    const int FUNNEL_SIZE = 12;

    // Sections narrower than this get no funnel, so it never collides with the label.
    const int MIN_SECTION = FUNNEL_SIZE + 28;

    // Width reserved on the right for the funnel, so the label and the sort indicator drawn by
    // the base class land to its left instead of underneath it. A few px past the glyph give a gap.
    const int FUNNEL_RESERVE = FUNNEL_SIZE + 4;
}

QFilterHeaderView::QFilterHeaderView(Qt::Orientation orientation, QWidget *parent)
    : QHeaderView(orientation, parent), hoveredSection(-1) {
    funnelIcon = QIcon::fromTheme("view-filter");

    // Set the header clickable so it can be sorted.
    setSectionsClickable(true);

    // Mouse tracking lets us repaint the hover tint without a button held down.
    setMouseTracking(true);
}

QRect QFilterHeaderView::funnelRect(const QRect &sectionRect) const {
    // Flush against the section's right edge with no padding. Any inset here pushes the glyph
    // left into the label, which on narrow columns ends up drawn over the text.
    int x = sectionRect.right() - FUNNEL_SIZE;
    int y = sectionRect.top() + (sectionRect.height() - FUNNEL_SIZE) / 2;
    return QRect(x, y, FUNNEL_SIZE, FUNNEL_SIZE);
}

int QFilterHeaderView::sectionAtFunnel(const QPoint &pos) const {
    int logical = logicalIndexAt(pos.x());
    if (logical < 0 || isSectionHidden(logical)) {
        return -1;
    }

    int sectionPos = sectionViewportPosition(logical);
    int sectionWidth = sectionSize(logical);
    if (sectionWidth < MIN_SECTION) {
        return -1;
    }

    QRect sectionRect(sectionPos, 0, sectionWidth, height());
    return funnelRect(sectionRect).contains(pos) ? logical : -1;
}

void QFilterHeaderView::paintSection(QPainter *painter, const QRect &rect, int logicalIndex) const {

    const bool showFunnel = rect.width() >= MIN_SECTION && !isSectionHidden(logicalIndex);

    // Without a funnel there's nothing to clear, so let the base class paint the full section.
    if (!showFunnel) {
        painter->save();
        QHeaderView::paintSection(painter, rect, logicalIndex);
        painter->restore();
        return;
    }

    // Paint the themed section background across the full width first, then let the base draw the
    // label and sort indicator into a rect narrowed on the right. That keeps the background under
    // the funnel intact while shifting the sort indicator left so it no longer overlaps the glyph.
    painter->save();
    QStyleOptionHeader opt;
    initStyleOptionForIndex(&opt, logicalIndex);
    opt.rect = rect;
    style()->drawControl(QStyle::CE_HeaderSection, &opt, painter, this);
    painter->restore();

    painter->save();
    QHeaderView::paintSection(painter, rect.adjusted(0, 0, -FUNNEL_RESERVE, 0), logicalIndex);
    painter->restore();

    QRect iconRect = funnelRect(rect);

    painter->save();
    if (logicalIndex == hoveredSection) {
        painter->setRenderHint(QPainter::Antialiasing, true);
        painter->setBrush(palette().color(QPalette::Midlight));
        painter->setPen(Qt::NoPen);
        painter->drawRoundedRect(iconRect.adjusted(-1, -1, 1, 1), 3, 3);
    }

    if (!funnelIcon.isNull()) {
        funnelIcon.paint(painter, iconRect, Qt::AlignCenter);
    }
    painter->restore();
}

void QFilterHeaderView::mousePressEvent(QMouseEvent *event) {

    // Consume the press when it lands on a funnel so it doesn't also trigger a sort.
    if (event->button() == Qt::LeftButton) {
        int logical = sectionAtFunnel(event->pos());
        if (logical >= 0) {
            emit funnelClicked(logical);
            return;
        }
    }
    QHeaderView::mousePressEvent(event);
}

void QFilterHeaderView::mouseMoveEvent(QMouseEvent *event) {

    int logical = sectionAtFunnel(event->pos());
    if (logical != hoveredSection) {
        int previous = hoveredSection;
        hoveredSection = logical;
        if (previous >= 0) {
            updateSection(previous);
        }
        if (logical >= 0) {
            updateSection(logical);
        }
    }
    QHeaderView::mouseMoveEvent(event);
}

void QFilterHeaderView::leaveEvent(QEvent *event) {

    if (hoveredSection >= 0) {
        int previous = hoveredSection;
        hoveredSection = -1;
        updateSection(previous);
    }
    QHeaderView::leaveEvent(event);
}

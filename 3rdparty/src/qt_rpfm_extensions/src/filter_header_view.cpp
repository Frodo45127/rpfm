#include "filter_header_view.h"

#include <QMouseEvent>
#include <QPainter>
#include <QPalette>

namespace {
    // Size of the funnel glyph, in header pixels.
    const int FUNNEL_SIZE = 12;

    // Sections narrower than this get no funnel, so it never collides with the label.
    const int MIN_SECTION = FUNNEL_SIZE + 28;
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

    // Let the base class draw the label, background and sort indicator first.
    painter->save();
    QHeaderView::paintSection(painter, rect, logicalIndex);
    painter->restore();

    if (rect.width() < MIN_SECTION || isSectionHidden(logicalIndex)) {
        return;
    }

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

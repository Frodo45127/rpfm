#ifndef FILTER_HEADER_VIEW_H
#define FILTER_HEADER_VIEW_H

#include "qt_subclasses_global.h"
#include <QHeaderView>
#include <QIcon>

// A horizontal QHeaderView that paints a small "filter funnel" affordance on the right
// edge of each section and emits funnelClicked() when it's pressed. Painting in the
// header means the affordance follows section resize/move/scroll/hide for free, with no
// overlay widgets or manual repositioning on the consumer side.
class QFilterHeaderView : public QHeaderView {
    Q_OBJECT

public:
    explicit QFilterHeaderView(Qt::Orientation orientation, QWidget *parent = nullptr);

signals:
    void funnelClicked(int logicalIndex);

protected:
    void paintSection(QPainter *painter, const QRect &rect, int logicalIndex) const override;
    void mousePressEvent(QMouseEvent *event) override;
    void mouseMoveEvent(QMouseEvent *event) override;
    void leaveEvent(QEvent *event) override;

private:
    // Funnel rect for a given section rect: square of FUNNEL_SIZE, vertically centered,
    // flush against the section's right edge (no padding).
    QRect funnelRect(const QRect &sectionRect) const;

    // Logical column whose funnel contains pos, or -1 if pos isn't over any funnel.
    int sectionAtFunnel(const QPoint &pos) const;

    QIcon funnelIcon;
    int hoveredSection;
};

#endif // FILTER_HEADER_VIEW_H

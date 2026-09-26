#include "group_formation_canvas.h"

#include <QFontMetricsF>
#include <QGraphicsLineItem>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QResizeEvent>
#include <QScrollBar>
#include <QStyleOptionGraphicsItem>
#include <QWheelEvent>
#include <QtMath>

// Distance between minor grid lines, in meters. Every fifth line is a major one.
static const qreal DEFAULT_GRID_STEP = 5.0;

// Minimum distance in pixels between drawn grid lines. Denser lines are skipped.
static const qreal MIN_GRID_PIXELS = 6.0;

// Margin around the blocks when fitting them in the view, in meters.
static const qreal FIT_MARGIN = 10.0;

// Z values of each kind of item, so spans are behind links, and links behind containers.
static const qreal Z_SPAN = -10.0;
static const qreal Z_LINK = 5.0;
static const qreal Z_CONTAINER = 10.0;

//---------------------------------------------------------------------------//
//                          Functions called from Rust
//---------------------------------------------------------------------------//

extern "C" QGraphicsView* new_group_formation_canvas(QWidget* parent) {
    return new GroupFormationCanvas(parent);
}

extern "C" void group_formation_canvas_clear(QGraphicsView* canvas) {
    dynamic_cast<GroupFormationCanvas*>(canvas)->clearBlocks();
}

extern "C" void group_formation_canvas_add_block(QGraphicsView* canvas, quint32 id, int kind, double center_x, double center_y, double width, double height, const QString* label, int red, int green, int blue) {
    QRectF rect(center_x - width / 2.0, center_y - height / 2.0, width, height);
    GroupFormationBlockItem* item = new GroupFormationBlockItem(id, kind, rect, *label, QColor(red, green, blue));
    dynamic_cast<GroupFormationCanvas*>(canvas)->addBlock(item);
}

extern "C" void group_formation_canvas_add_link(QGraphicsView* canvas, quint32 child_id, quint32 parent_id) {
    dynamic_cast<GroupFormationCanvas*>(canvas)->addLink(child_id, parent_id);
}

extern "C" void group_formation_canvas_set_selected_ids(QGraphicsView* canvas, const quint32* ids, int count) {
    QList<quint32> list;
    for (int i = 0; i < count; ++i) {
        list.append(ids[i]);
    }
    dynamic_cast<GroupFormationCanvas*>(canvas)->setSelectedIds(list);
}

extern "C" int group_formation_canvas_selected_count(QGraphicsView* canvas) {
    return dynamic_cast<GroupFormationCanvas*>(canvas)->selectedIds().count();
}

extern "C" quint32 group_formation_canvas_selected_id(QGraphicsView* canvas, int index) {
    return dynamic_cast<GroupFormationCanvas*>(canvas)->selectedIds().at(index);
}

extern "C" void group_formation_canvas_set_front_label(QGraphicsView* canvas, const QString* label) {
    dynamic_cast<GroupFormationCanvas*>(canvas)->setFrontLabel(*label);
}

extern "C" void group_formation_canvas_fit(QGraphicsView* canvas) {
    dynamic_cast<GroupFormationCanvas*>(canvas)->fitToBlocks();
}

//---------------------------------------------------------------------------//
//                                Block item
//---------------------------------------------------------------------------//

GroupFormationBlockItem::GroupFormationBlockItem(quint32 id, int kind, const QRectF& rect, const QString& label, const QColor& color):
    QGraphicsRectItem(rect),
    block_id(id),
    block_kind(kind),
    label(label),
    color(color)
{
    setFlag(QGraphicsItem::ItemIsSelectable, true);
    setZValue(kind == GroupFormationBlockKind::Span ? Z_SPAN : Z_CONTAINER);
    setToolTip(label);
}

quint32 GroupFormationBlockItem::blockId() const {
    return block_id;
}

int GroupFormationBlockItem::kind() const {
    return block_kind;
}

void GroupFormationBlockItem::paint(QPainter* painter, const QStyleOptionGraphicsItem* option, QWidget* widget) {
    const QPalette palette = widget ? widget->palette() : QPalette();
    const bool is_selected = option->state & QStyle::State_Selected;
    const QColor highlight = palette.color(QPalette::Highlight);

    // Cosmetic pens keep the same width in pixels at any zoom.
    QPen pen(is_selected ? highlight : color.darker(160));
    pen.setCosmetic(true);
    pen.setWidthF(is_selected ? 3.0 : (block_kind == GroupFormationBlockKind::AbsoluteContainer ? 2.0 : 1.0));

    painter->setRenderHint(QPainter::Antialiasing, true);
    if (block_kind == GroupFormationBlockKind::Span) {
        QColor fill = color;
        fill.setAlpha(40);
        pen.setStyle(Qt::DashLine);
        painter->setPen(pen);
        painter->setBrush(fill);
        painter->drawRoundedRect(rect(), 1.0, 1.0);
    } else {
        painter->setPen(pen);
        painter->setBrush(color);
        painter->drawRect(rect());
    }

    // Text is laid out in meters, so it scales with the zoom like the blocks do.
    QFont font = painter->font();
    font.setPointSizeF(qMin(rect().height() * 0.45, 3.0));
    painter->setFont(font);

    const QColor text_color = block_kind == GroupFormationBlockKind::Span ? palette.color(QPalette::Text) : (color.lightnessF() > 0.5 ? Qt::black : Qt::white);
    painter->setPen(text_color);

    const QRectF text_rect = rect().adjusted(0.5, 0.2, -0.5, -0.2);
    const QString text = QFontMetricsF(font).elidedText(label, Qt::ElideRight, text_rect.width());
    const Qt::Alignment alignment = block_kind == GroupFormationBlockKind::Span ? (Qt::AlignLeft | Qt::AlignTop) : Qt::AlignCenter;
    painter->drawText(text_rect, alignment, text);
}

//---------------------------------------------------------------------------//
//                                  Canvas
//---------------------------------------------------------------------------//

GroupFormationCanvas::GroupFormationCanvas(QWidget* parent):
    QGraphicsView(parent),
    formation_scene(new QGraphicsScene(this)),
    grid_step(DEFAULT_GRID_STEP),
    is_updating_selection(false),
    is_panning(false),
    is_fit_pending(false)
{
    setScene(formation_scene);
    setRenderHint(QPainter::Antialiasing, true);
    setDragMode(QGraphicsView::RubberBandDrag);
    setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
    setViewportUpdateMode(QGraphicsView::FullViewportUpdate);

    // The grid is infinite, so the scene rect is kept large enough to pan around freely.
    formation_scene->setSceneRect(-10000, -10000, 20000, 20000);

    connect(formation_scene, &QGraphicsScene::selectionChanged, this, [this]() {
        if (!is_updating_selection) {
            emit blockSelectionChanged();
        }
    });
}

void GroupFormationCanvas::clearBlocks() {
    is_updating_selection = true;
    formation_scene->clear();
    blocks.clear();
    is_updating_selection = false;
}

void GroupFormationCanvas::addBlock(GroupFormationBlockItem* item) {
    formation_scene->addItem(item);
    blocks.insert(item->blockId(), item);
}

void GroupFormationCanvas::addLink(quint32 child_id, quint32 parent_id) {
    GroupFormationBlockItem* child = blocks.value(child_id, nullptr);
    GroupFormationBlockItem* parent = blocks.value(parent_id, nullptr);
    if (child == nullptr || parent == nullptr) {
        return;
    }

    // The link goes between the borders of both blocks, not their centers, so it's not hidden under them.
    auto border_point = [](const QRectF& rect, const QPointF& towards) {
        const QPointF center = rect.center();
        const QPointF direction = towards - center;
        if (qFuzzyIsNull(direction.x()) && qFuzzyIsNull(direction.y())) {
            return center;
        }

        const qreal scale_x = qFuzzyIsNull(direction.x()) ? qInf() : (rect.width() / 2.0) / qAbs(direction.x());
        const qreal scale_y = qFuzzyIsNull(direction.y()) ? qInf() : (rect.height() / 2.0) / qAbs(direction.y());
        return center + direction * qMin(qMin(scale_x, scale_y), 1.0);
    };

    const QRectF child_rect = child->rect();
    const QRectF parent_rect = parent->rect();
    const QPointF start = border_point(child_rect, parent_rect.center());
    const QPointF end = border_point(parent_rect, child_rect.center());

    QPen pen(palette().color(QPalette::Text));
    pen.setCosmetic(true);
    pen.setWidthF(1.5);

    QPainterPath path(start);
    path.lineTo(end);

    // Arrow head pointing at the parent, sized in meters.
    const QLineF line(start, end);
    if (line.length() > 0.01) {
        const qreal head = qMin(1.5, line.length() / 2.0);
        const qreal angle = qDegreesToRadians(line.angle());
        const QPointF left = end + QPointF(-qCos(angle - M_PI / 6.0) * head, qSin(angle - M_PI / 6.0) * head);
        const QPointF right = end + QPointF(-qCos(angle + M_PI / 6.0) * head, qSin(angle + M_PI / 6.0) * head);
        path.moveTo(left);
        path.lineTo(end);
        path.lineTo(right);
    }

    QGraphicsPathItem* link = formation_scene->addPath(path, pen);
    link->setZValue(Z_LINK);
}

void GroupFormationCanvas::setSelectedIds(const QList<quint32>& ids) {
    is_updating_selection = true;
    formation_scene->clearSelection();
    for (quint32 id : ids) {
        GroupFormationBlockItem* item = blocks.value(id, nullptr);
        if (item != nullptr) {
            item->setSelected(true);
        }
    }
    is_updating_selection = false;
}

QList<quint32> GroupFormationCanvas::selectedIds() const {
    QList<quint32> ids;
    for (QGraphicsItem* item : formation_scene->selectedItems()) {
        GroupFormationBlockItem* block = dynamic_cast<GroupFormationBlockItem*>(item);
        if (block != nullptr) {
            ids.append(block->blockId());
        }
    }
    return ids;
}

void GroupFormationCanvas::setFrontLabel(const QString& label) {
    front_label = label;
    viewport()->update();
}

void GroupFormationCanvas::fitToBlocks() {

    // A view that's not shown yet has no real size, so the fit waits for the first resize.
    if (viewport()->width() < 50 || viewport()->height() < 50) {
        is_fit_pending = true;
        return;
    }

    is_fit_pending = false;
    QRectF bounds;
    for (GroupFormationBlockItem* item : std::as_const(blocks)) {
        bounds = bounds.united(item->rect());
    }

    if (bounds.isNull()) {
        bounds = QRectF(-25, -25, 50, 50);
    }

    fitInView(bounds.adjusted(-FIT_MARGIN, -FIT_MARGIN, FIT_MARGIN, FIT_MARGIN), Qt::KeepAspectRatio);
}

void GroupFormationCanvas::drawBackground(QPainter* painter, const QRectF& rect) {
    painter->fillRect(rect, palette().color(QPalette::Base));

    const qreal pixels_per_meter = transform().m11();
    QColor minor_color = palette().color(QPalette::Text);
    minor_color.setAlpha(25);
    QColor major_color = palette().color(QPalette::Text);
    major_color.setAlpha(60);
    QColor axis_color = palette().color(QPalette::Text);
    axis_color.setAlpha(120);

    // Skip the minor lines when zoomed out so much they'd turn into noise.
    const bool draw_minor = grid_step * pixels_per_meter >= MIN_GRID_PIXELS;
    const qreal major_step = grid_step * 5.0;
    const qreal step = draw_minor ? grid_step : major_step;
    if (step * pixels_per_meter < MIN_GRID_PIXELS) {
        return;
    }

    QPen pen;
    pen.setCosmetic(true);
    pen.setWidthF(1.0);

    const qreal left = qFloor(rect.left() / step) * step;
    const qreal top = qFloor(rect.top() / step) * step;
    for (qreal x = left; x <= rect.right(); x += step) {
        const bool is_axis = qFuzzyIsNull(x);
        const bool is_major = qFuzzyIsNull(std::fmod(qAbs(x), major_step));
        pen.setColor(is_axis ? axis_color : (is_major ? major_color : minor_color));
        painter->setPen(pen);
        painter->drawLine(QPointF(x, rect.top()), QPointF(x, rect.bottom()));
    }

    for (qreal y = top; y <= rect.bottom(); y += step) {
        const bool is_axis = qFuzzyIsNull(y);
        const bool is_major = qFuzzyIsNull(std::fmod(qAbs(y), major_step));
        pen.setColor(is_axis ? axis_color : (is_major ? major_color : minor_color));
        painter->setPen(pen);
        painter->drawLine(QPointF(rect.left(), y), QPointF(rect.right(), y));
    }
}

void GroupFormationCanvas::drawForeground(QPainter* painter, const QRectF& rect) {
    Q_UNUSED(rect);

    // Drawn in viewport pixels, so the arrow marking the front stays at the top of the view.
    painter->save();
    painter->resetTransform();
    painter->setRenderHint(QPainter::Antialiasing, true);

    const QColor color = palette().color(QPalette::Text);
    const qreal center_x = viewport()->width() / 2.0;

    QPainterPath arrow;
    arrow.moveTo(center_x, 6);
    arrow.lineTo(center_x - 8, 16);
    arrow.lineTo(center_x + 8, 16);
    arrow.closeSubpath();
    painter->fillPath(arrow, color);

    if (!front_label.isEmpty()) {
        painter->setPen(color);
        painter->drawText(QRectF(0, 18, viewport()->width(), 20), Qt::AlignHCenter | Qt::AlignTop, front_label);
    }

    painter->restore();
}

void GroupFormationCanvas::wheelEvent(QWheelEvent* event) {
    const qreal factor = event->angleDelta().y() > 0 ? 1.15 : 1.0 / 1.15;
    const qreal new_scale = transform().m11() * factor;

    // Limits so the view can't be zoomed into a single pixel or out to nothing.
    if (new_scale < 0.2 || new_scale > 200.0) {
        return;
    }

    scale(factor, factor);
}

void GroupFormationCanvas::mousePressEvent(QMouseEvent* event) {
    if (event->button() == Qt::MiddleButton) {
        is_panning = true;
        last_pan_position = event->pos();
        setCursor(Qt::ClosedHandCursor);
        event->accept();
        return;
    }

    QGraphicsView::mousePressEvent(event);
}

void GroupFormationCanvas::mouseMoveEvent(QMouseEvent* event) {
    if (is_panning) {
        const QPoint delta = event->pos() - last_pan_position;
        last_pan_position = event->pos();
        horizontalScrollBar()->setValue(horizontalScrollBar()->value() - delta.x());
        verticalScrollBar()->setValue(verticalScrollBar()->value() - delta.y());
        event->accept();
        return;
    }

    QGraphicsView::mouseMoveEvent(event);
}

void GroupFormationCanvas::resizeEvent(QResizeEvent* event) {
    QGraphicsView::resizeEvent(event);
    if (is_fit_pending) {
        fitToBlocks();
    }
}

void GroupFormationCanvas::mouseReleaseEvent(QMouseEvent* event) {
    if (event->button() == Qt::MiddleButton && is_panning) {
        is_panning = false;
        unsetCursor();
        event->accept();
        return;
    }

    QGraphicsView::mouseReleaseEvent(event);
}

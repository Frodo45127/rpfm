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

extern "C" void group_formation_canvas_add_block(QGraphicsView* canvas, quint32 id, int kind, double center_x, double center_y, double width, double height, double bend, const QString* label, int red, int green, int blue) {
    QRectF rect(center_x - width / 2.0, center_y - height / 2.0, width, height);
    GroupFormationBlockItem* item = new GroupFormationBlockItem(id, kind, rect, bend, *label, QColor(red, green, blue));
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

extern "C" void group_formation_canvas_set_grid_step(QGraphicsView* canvas, double step) {
    dynamic_cast<GroupFormationCanvas*>(canvas)->setGridStep(step);
}

extern "C" void group_formation_canvas_set_editable(QGraphicsView* canvas, bool editable) {
    dynamic_cast<GroupFormationCanvas*>(canvas)->setEditable(editable);
}

extern "C" double group_formation_canvas_moved_delta_x(QGraphicsView* canvas) {
    return dynamic_cast<GroupFormationCanvas*>(canvas)->movedDelta().x();
}

extern "C" double group_formation_canvas_moved_delta_y(QGraphicsView* canvas) {
    return dynamic_cast<GroupFormationCanvas*>(canvas)->movedDelta().y();
}

extern "C" quint32 group_formation_canvas_link_child(QGraphicsView* canvas) {
    return dynamic_cast<GroupFormationCanvas*>(canvas)->linkChild();
}

extern "C" quint32 group_formation_canvas_link_parent(QGraphicsView* canvas) {
    return dynamic_cast<GroupFormationCanvas*>(canvas)->linkParent();
}

//---------------------------------------------------------------------------//
//                                Block item
//---------------------------------------------------------------------------//

GroupFormationBlockItem::GroupFormationBlockItem(quint32 id, int kind, const QRectF& rect, qreal bend, const QString& label, const QColor& color):
    QGraphicsRectItem(rect),
    block_id(id),
    block_kind(kind),
    bend(bend),
    label(label),
    color(color)
{
    setFlag(QGraphicsItem::ItemIsSelectable, true);

    // Dragging a span is reported like any other move. The Rust side moves its members.
    setFlag(QGraphicsItem::ItemIsMovable, true);
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
    } else if (!qFuzzyIsNull(bend) && qAbs(bend) < rect().height()) {

        // Crescents are drawn as a curved band. The control point of each curve is placed so the curve's peak lands on the middle.
        const QRectF area = rect();
        const qreal thickness = area.height() - qAbs(bend);
        const qreal ends_y = area.top() + qMax(bend, 0.0);
        const qreal middle_y = area.top() + qMax(-bend, 0.0);
        const qreal control_y = 2.0 * middle_y - ends_y;

        QPainterPath band(QPointF(area.left(), ends_y));
        band.quadTo(QPointF(area.center().x(), control_y), QPointF(area.right(), ends_y));
        band.lineTo(area.right(), ends_y + thickness);
        band.quadTo(QPointF(area.center().x(), control_y + thickness), QPointF(area.left(), ends_y + thickness));
        band.closeSubpath();

        painter->setPen(pen);
        painter->setBrush(color);
        painter->drawPath(band);
    } else {
        painter->setPen(pen);
        painter->setBrush(color);
        painter->drawRect(rect());
    }

    // Text is drawn in pixels at the normal font size, as fonts scaled by the zoom fail to render when they get too big.
    const QRectF text_rect = painter->worldTransform().mapRect(rect()).adjusted(3.0, 1.0, -3.0, -1.0);
    const QFont font = widget ? widget->font() : painter->font();
    const QFontMetricsF metrics(font);
    if (text_rect.height() < metrics.height() || text_rect.width() < metrics.averageCharWidth() * 3.0) {
        return;
    }

    painter->save();
    painter->resetTransform();
    painter->setFont(font);

    const QColor text_color = block_kind == GroupFormationBlockKind::Span ? palette.color(QPalette::Text) : (color.lightnessF() > 0.5 ? Qt::black : Qt::white);
    painter->setPen(text_color);

    const QString text = metrics.elidedText(label, Qt::ElideRight, text_rect.width());
    const Qt::Alignment alignment = block_kind == GroupFormationBlockKind::Span ? (Qt::AlignLeft | Qt::AlignTop) : Qt::AlignCenter;
    painter->drawText(text_rect, alignment, text);
    painter->restore();
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
    is_fit_pending(false),
    is_editable(true),
    link_source(nullptr),
    link_preview(nullptr),
    link_child(0),
    link_parent(0)
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
    links.clear();
    link_source = nullptr;
    link_preview = nullptr;
    is_updating_selection = false;
}

void GroupFormationCanvas::addBlock(GroupFormationBlockItem* item) {
    if (!is_editable) {
        item->setFlag(QGraphicsItem::ItemIsMovable, false);
    }

    formation_scene->addItem(item);
    blocks.insert(item->blockId(), item);
}

void GroupFormationCanvas::addLink(quint32 child_id, quint32 parent_id) {
    GroupFormationBlockItem* child = blocks.value(child_id, nullptr);
    GroupFormationBlockItem* parent = blocks.value(parent_id, nullptr);
    if (child == nullptr || parent == nullptr) {
        return;
    }

    QPen pen(palette().color(QPalette::Text));
    pen.setCosmetic(true);
    pen.setWidthF(1.5);

    QGraphicsPathItem* path = formation_scene->addPath(QPainterPath(), pen);
    path->setZValue(Z_LINK);
    links.append(GroupFormationLink { child, parent, path });
    updateLinkPaths();
}

void GroupFormationCanvas::updateLinkPaths() {

    // Links go between the borders of both blocks, not their centers, so they're not hidden under them.
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

    for (const GroupFormationLink& link : std::as_const(links)) {

        // Dragged blocks keep their rect and get moved by their position, so the scene rect accounts for both.
        const QRectF child_rect = link.child->mapRectToScene(link.child->rect());
        const QRectF parent_rect = link.parent->mapRectToScene(link.parent->rect());
        const QPointF start = border_point(child_rect, parent_rect.center());
        const QPointF end = border_point(parent_rect, child_rect.center());

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

        link.path->setPath(path);
    }
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

void GroupFormationCanvas::setGridStep(qreal step) {
    if (step > 0.0) {
        grid_step = step;
        viewport()->update();
    }
}

void GroupFormationCanvas::setEditable(bool editable) {
    is_editable = editable;
    for (GroupFormationBlockItem* item : std::as_const(blocks)) {
        item->setFlag(QGraphicsItem::ItemIsMovable, editable);
    }
}

QPointF GroupFormationCanvas::movedDelta() const {
    return moved_delta;
}

quint32 GroupFormationCanvas::linkChild() const {
    return link_child;
}

quint32 GroupFormationCanvas::linkParent() const {
    return link_parent;
}

GroupFormationBlockItem* GroupFormationCanvas::blockAt(const QPoint& position, const GroupFormationBlockItem* ignored) const {

    // Items come sorted from top to bottom, so containers are found before the spans behind them.
    for (QGraphicsItem* item : items(position)) {
        GroupFormationBlockItem* block = dynamic_cast<GroupFormationBlockItem*>(item);
        if (block != nullptr && block != ignored) {
            return block;
        }
    }
    return nullptr;
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

    GroupFormationBlockItem* block = blockAt(event->pos());

    // Shift+drag from a block starts drawing a link instead of moving or selecting.
    if (is_editable && event->button() == Qt::LeftButton && (event->modifiers() & Qt::ShiftModifier) && block != nullptr) {
        link_source = block;
        const QPointF start = block->rect().center();
        QPen pen(palette().color(QPalette::Highlight));
        pen.setCosmetic(true);
        pen.setWidthF(2.0);
        pen.setStyle(Qt::DashLine);
        link_preview = formation_scene->addLine(QLineF(start, start), pen);
        link_preview->setZValue(Z_CONTAINER + 1.0);
        event->accept();
        return;
    }

    // Right-clicking a block that's not selected selects only it, so the context menu acts on it.
    if (event->button() == Qt::RightButton && block != nullptr && !block->isSelected()) {
        formation_scene->clearSelection();
        block->setSelected(true);
    }

    QGraphicsView::mousePressEvent(event);
}

void GroupFormationCanvas::mouseMoveEvent(QMouseEvent* event) {
    if (link_preview != nullptr) {
        link_preview->setLine(QLineF(link_preview->line().p1(), mapToScene(event->pos())));
        event->accept();
        return;
    }

    if (is_panning) {
        const QPoint delta = event->pos() - last_pan_position;
        last_pan_position = event->pos();
        horizontalScrollBar()->setValue(horizontalScrollBar()->value() - delta.x());
        verticalScrollBar()->setValue(verticalScrollBar()->value() - delta.y());
        event->accept();
        return;
    }

    QGraphicsView::mouseMoveEvent(event);

    // While dragging blocks, keep their links attached to them.
    if (event->buttons() & Qt::LeftButton) {
        updateLinkPaths();
    }
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

    if (event->button() == Qt::LeftButton && link_preview != nullptr) {
        GroupFormationBlockItem* target = blockAt(event->pos(), link_source);
        const quint32 child = link_source->blockId();

        formation_scene->removeItem(link_preview);
        delete link_preview;
        link_preview = nullptr;
        link_source = nullptr;

        if (target != nullptr) {
            link_child = child;
            link_parent = target->blockId();
            emit linkRequested();
        }

        event->accept();
        return;
    }

    QGraphicsView::mouseReleaseEvent(event);

    // Dragged items keep their rect and get an offset position, which is the same for all of them.
    if (event->button() == Qt::LeftButton) {
        for (QGraphicsItem* item : formation_scene->selectedItems()) {
            if (!item->pos().isNull()) {
                moved_delta = item->pos();
                emit blocksMoved();
                break;
            }
        }
    }
}

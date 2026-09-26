#ifndef GROUP_FORMATION_CANVAS_H
#define GROUP_FORMATION_CANVAS_H

#include "qt_subclasses_global.h"
#include <QGraphicsRectItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QHash>

// Kinds of blocks the canvas can draw. They must match the values used on the Rust side.
enum GroupFormationBlockKind {
    AbsoluteContainer = 0,
    RelativeContainer = 1,
    Span = 2,
};

extern "C" QGraphicsView* new_group_formation_canvas(QWidget* parent);
extern "C" void group_formation_canvas_clear(QGraphicsView* canvas);
extern "C" void group_formation_canvas_add_block(QGraphicsView* canvas, quint32 id, int kind, double center_x, double center_y, double width, double height, const QString* label, int red, int green, int blue);
extern "C" void group_formation_canvas_add_link(QGraphicsView* canvas, quint32 child_id, quint32 parent_id);
extern "C" void group_formation_canvas_set_selected_ids(QGraphicsView* canvas, const quint32* ids, int count);
extern "C" int group_formation_canvas_selected_count(QGraphicsView* canvas);
extern "C" quint32 group_formation_canvas_selected_id(QGraphicsView* canvas, int index);
extern "C" void group_formation_canvas_set_front_label(QGraphicsView* canvas, const QString* label);
extern "C" void group_formation_canvas_fit(QGraphicsView* canvas);

// A block of the formation. It only paints itself, the canvas owns the logic.
class GroupFormationBlockItem : public QGraphicsRectItem {
public:
    GroupFormationBlockItem(quint32 id, int kind, const QRectF& rect, const QString& label, const QColor& color);
    quint32 blockId() const;
    int kind() const;

protected:
    void paint(QPainter* painter, const QStyleOptionGraphicsItem* option, QWidget* widget) override;

private:
    quint32 block_id;
    int block_kind;
    QString label;
    QColor color;
};

// View that draws the simulated layout of a formation over a grid in meters.
//
// It knows nothing about formations: the Rust side sends it rects already positioned in scene
// coordinates, and reads back what the user selected.
class GroupFormationCanvas : public QGraphicsView {
    Q_OBJECT

signals:
    void blockSelectionChanged();

public:
    explicit GroupFormationCanvas(QWidget* parent = nullptr);
    void clearBlocks();
    void addBlock(GroupFormationBlockItem* item);
    void addLink(quint32 child_id, quint32 parent_id);
    void setSelectedIds(const QList<quint32>& ids);
    QList<quint32> selectedIds() const;
    void setFrontLabel(const QString& label);
    void fitToBlocks();

protected:
    void drawBackground(QPainter* painter, const QRectF& rect) override;
    void drawForeground(QPainter* painter, const QRectF& rect) override;
    void wheelEvent(QWheelEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;
    void resizeEvent(QResizeEvent* event) override;

private:
    QGraphicsScene* formation_scene;
    QHash<quint32, GroupFormationBlockItem*> blocks;
    QString front_label;
    qreal grid_step;
    bool is_updating_selection;
    bool is_panning;
    bool is_fit_pending;
    QPoint last_pan_position;
};

#endif // GROUP_FORMATION_CANVAS_H

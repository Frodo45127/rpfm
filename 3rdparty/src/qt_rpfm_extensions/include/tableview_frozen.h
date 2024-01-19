#ifndef TABLEVIEW_FROZEN_H
#define TABLEVIEW_FROZEN_H

#include "qt_subclasses_global.h"
#include <QTableView>
#include <QEvent>

extern "C" QTableView* new_tableview_frozen(QWidget* parent = nullptr, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY) = nullptr);
extern "C" void toggle_freezer(QTableView* tableView = nullptr, int column = 0);

class QTableViewFrozen : public QTableView {
     Q_OBJECT

public:
    QTableViewFrozen(QWidget* parent, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY) = nullptr);
    ~QTableViewFrozen() override;

    void setModel(QAbstractItemModel * model) override;
    void setUpdatesEnabled(bool enable);
    void setItemDelegateForColumn(int column, QAbstractItemDelegate* delegate);
    QTableView *tableViewFrozen;

protected:
    void resizeEvent(QResizeEvent *event) override;
    QModelIndex moveCursor(CursorAction cursorAction, Qt::KeyboardModifiers modifiers) override;
    void scrollTo (const QModelIndex & index, ScrollHint hint = EnsureVisible) override;
    int baseLeftMargin = -1;

private:
    QList<int> frozenColumns;
    QPoint _lastPosition;
    void (*generateTooltipMessage)(QTableView* view, int globalPosX, int globalPosY);
    void init();
    void sectionMoved(int logicalIndex, int oldVisualIndex, int newVisualIndex);
    void updateFrozenTableGeometry();

public slots:
    void toggleFreezer(int column = -1);

private slots:
    void updateSectionWidth(int logicalIndex, int oldSize, int newSize);
    void updateSectionHeight(int logicalIndex, int oldSize, int newSize);
    bool viewportEvent(QEvent *event) override;
};
#endif // TABLEVIEW_FROZEN_H

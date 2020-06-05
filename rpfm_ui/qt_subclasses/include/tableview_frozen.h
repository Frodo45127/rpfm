#ifndef TABLEVIEW_FROZEN_H
#define TABLEVIEW_FROZEN_H

#include "qt_subclasses_global.h"
#include <QTableView>

extern "C" QTableView* new_tableview_frozen(QWidget* parent = nullptr);
extern "C" void toggle_freezer(QTableView* tableView = nullptr, int column = 0);

class QTableViewFrozen : public QTableView {
     Q_OBJECT

public:
    QTableViewFrozen(QWidget* parent);
    ~QTableViewFrozen() override;

    void setDataModel(QAbstractItemModel * model);
    QTableView *tableViewFrozen;

protected:
    //void resizeEvent(QResizeEvent *event) override;
    //QModelIndex moveCursor(CursorAction cursorAction, Qt::KeyboardModifiers modifiers) override;
    //void scrollTo (const QModelIndex & index, ScrollHint hint = EnsureVisible) override;

private:
    QList<int> frozenColumns;
    void init();
    //void updateFrozenTableGeometry();

public slots:
    void toggleFreezer(int column = 0);

private slots:
    //void updateSectionWidth(int logicalIndex, int oldSize, int newSize);
    //void updateSectionHeight(int logicalIndex, int oldSize, int newSize);
};
#endif // TABLEVIEW_FROZEN_H

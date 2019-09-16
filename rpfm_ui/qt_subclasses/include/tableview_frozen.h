#ifndef TABLEVIEW_FROZEN_H
#define TABLEVIEW_FROZEN_H

#include "qt_subclasses_global.h"
#include <QTableView>

extern "C" QTableView* new_tableview_frozen(QAbstractItemModel* model = 0, QTableView* frozen_table = 0);

class QTableViewFrozen : public QTableView {
     Q_OBJECT

public:
      QTableViewFrozen(QAbstractItemModel* model, QTableView* tableview);
      ~QTableViewFrozen();

protected:
      void resizeEvent(QResizeEvent *event) override;
      QModelIndex moveCursor(CursorAction cursorAction, Qt::KeyboardModifiers modifiers) override;
      void scrollTo (const QModelIndex & index, ScrollHint hint = EnsureVisible) override;

private:
      QTableView *frozenTableView;
      void init();
      void updateFrozenTableGeometry();

private slots:
      void updateSectionWidth(int logicalIndex, int oldSize, int newSize);
      void updateSectionHeight(int logicalIndex, int oldSize, int newSize);

};
#endif // TABLEVIEW_FROZEN_H

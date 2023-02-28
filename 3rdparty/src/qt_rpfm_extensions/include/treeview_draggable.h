#ifndef TREEVIEW_DRAGGABLE_H
#define TREEVIEW_DRAGGABLE_H

#include "qt_subclasses_global.h"
#include <QTreeView>
#include <QDropEvent>

extern "C" QTreeView* new_packed_file_treeview(QWidget *parent = nullptr);

class TreeViewDraggable : public QTreeView {
    Q_OBJECT
signals:
    void itemDrop(QModelIndex const &,int);
public:
    explicit TreeViewDraggable(QWidget *parent = nullptr);

protected:
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dragMoveEvent(QDragMoveEvent *event) override;
    void dragLeaveEvent(QDragLeaveEvent *event) override;
    void dropEvent(QDropEvent *event) override;
};

#endif // TREEVIEW_DRAGGABLE_H

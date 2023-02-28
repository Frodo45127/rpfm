#include "treeview_draggable.h"

#include <QDebug>
#include <QMimeData>
#include <QStandardItem>
#include <QHeaderView>

extern "C" QTreeView* new_packed_file_treeview(QWidget *parent) {
    return dynamic_cast<QTreeView*>(new TreeViewDraggable(parent));
}

TreeViewDraggable::TreeViewDraggable(QWidget *parent) : QTreeView(parent) {
    setContextMenuPolicy(Qt::CustomContextMenu);
    setAlternatingRowColors(true);
    setSelectionMode(SelectionMode::ExtendedSelection);
    setSelectionBehavior(QAbstractItemView::SelectionBehavior::SelectItems);

    setUniformRowHeights(true);
    setAnimated(true);
    setHeaderHidden(true);
    setExpandsOnDoubleClick(true);
    header()->setStretchLastSection(true);

    setDragEnabled(true);
    setAcceptDrops(true);
    setDropIndicatorShown(true);
    setDragDropMode(DragDropMode::InternalMove);
    setDragDropOverwriteMode(false);
}

void TreeViewDraggable::dragEnterEvent(QDragEnterEvent *event) {
    event->acceptProposedAction();
}

void TreeViewDraggable::dragMoveEvent(QDragMoveEvent *event) {
    QTreeView::dragMoveEvent(event);
    if (!event->isAccepted())
        return;

    QModelIndex index = indexAt(event->pos());
    if (!index.isValid())
        event->ignore();
    else if (visualRect(index).adjusted(-1, -1, 1, 1).contains(event->pos(), false))
        event->accept();
    else
        event->ignore(); //Show 'forbidden' cursor.
}

void TreeViewDraggable::dragLeaveEvent(QDragLeaveEvent *event) {
    QTreeView::dragLeaveEvent(event);
}

void TreeViewDraggable::dropEvent(QDropEvent *event) {
    QModelIndex index = indexAt(event->pos());
    if (!index.isValid()) {
        return;
    }

    QModelIndex parent = index.parent();
    if (!parent.isValid()) {
        return;
    }

    emit itemDrop(parent, index.row());
}

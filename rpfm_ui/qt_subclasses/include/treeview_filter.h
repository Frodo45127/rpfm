#ifndef TREEVIEW_FILTER_H
#define TREEVIEW_FILTER_H

#include "qt_subclasses_global.h"
#include <QSortFilterProxyModel>
#include <QStandardItem>

extern "C" QSortFilterProxyModel* new_treeview_filter(QObject *parent = nullptr);
extern "C" void trigger_treeview_filter(QSortFilterProxyModel *filter = nullptr, QRegExp* pattern = nullptr);

class QTreeViewSortFilterProxyModel : public QSortFilterProxyModel
{
    Q_OBJECT

public:

    explicit QTreeViewSortFilterProxyModel(QObject *parent = nullptr);
    bool filterAcceptsRow(int source_row, const QModelIndex & source_parent) const;

signals:

private:
};

#endif // TREEVIEW_FILTER_H

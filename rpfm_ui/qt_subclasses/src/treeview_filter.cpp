#include "treeview_filter.h"
#include <QSortFilterProxyModel>
#include <QItemSelection>
#include <QRegExp>
#include <QStandardItem>
#include <QStandardItemModel>

// Function to create the filter in a way that we don't need to bother Rust with new types.
extern "C" QSortFilterProxyModel* new_treeview_filter(QObject *parent) {
    QTreeViewSortFilterProxyModel* filter = new QTreeViewSortFilterProxyModel(parent);
    return dynamic_cast<QSortFilterProxyModel*>(filter);
}

// Funtion to trigger the filter we want, instead of the default one, from Rust.
extern "C" void trigger_treeview_filter(QSortFilterProxyModel* filter, QRegExp* pattern) {
    QTreeViewSortFilterProxyModel* filter2 = static_cast<QTreeViewSortFilterProxyModel*>(filter);
    filter2->setFilterRegExp(*pattern);
}

// Constructor of QTreeViewSortFilterProxyModel.
QTreeViewSortFilterProxyModel::QTreeViewSortFilterProxyModel(QObject *parent): QSortFilterProxyModel(parent) {}

// Function called when the filter changes.
bool QTreeViewSortFilterProxyModel::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {

    // Check the current item. If it's a file, we just call the parent's filter.
    bool result = QSortFilterProxyModel::filterAcceptsRow(source_row, source_parent);
    QModelIndex currntIndex = sourceModel()->index(source_row, 0, source_parent);

    // If it has children, is a folder, so check each of his children.
    if (sourceModel()->hasChildren(currntIndex)) {
        for (int i = 0; i < sourceModel()->rowCount(currntIndex) && !result; ++i) {

            QString extraData1 = currntIndex.data(41).toString();
            QString extraData2 = currntIndex.data(42).toString();

            if (!extraData1.isEmpty()) {
                result |= extraData1.contains(filterRegExp());
            }

            if (!extraData2.isEmpty()) {
                result |= extraData2.contains(filterRegExp());
            }

            // Keep the parent if a children is shown.
            result |= filterAcceptsRow(i, currntIndex);
        }
    }

    // If it's a file, and it's not visible, there is a special behavior:
    // if the parent matches the filter, we assume all it's children do it too.
    // This is to avoid the "Show table folder, no table file" problem.
    else if (!result) {

        QModelIndex granpa = source_parent.parent();
        int granpa_row = source_parent.row();
        result = QSortFilterProxyModel::filterAcceptsRow(granpa_row, granpa);

        QString extraData1 = currntIndex.data(41).toString();
        QString extraData2 = currntIndex.data(42).toString();

        if (!extraData1.isEmpty()) {
            result |= extraData1.contains(filterRegExp());
        }

        if (!extraData2.isEmpty()) {
            result |= extraData2.contains(filterRegExp());
        }
    }

    return result;
}

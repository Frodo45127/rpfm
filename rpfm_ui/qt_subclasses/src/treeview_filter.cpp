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

// Funtion to trigger the filter we want, instead of the default one from Rust.
extern "C" void trigger_treeview_filter(QSortFilterProxyModel* filter, QRegExp* pattern, bool filter_by_folder) {
    QTreeViewSortFilterProxyModel* filter2 = static_cast<QTreeViewSortFilterProxyModel*>(filter);
    filter2->filter_by_folder = filter_by_folder;
    filter2->setFilterRegExp(*pattern);
}

// Constructor of QTreeViewSortFilterProxyModel.
QTreeViewSortFilterProxyModel::QTreeViewSortFilterProxyModel(QObject *parent): QSortFilterProxyModel(parent) {}

// Function called when the filter changes.
bool QTreeViewSortFilterProxyModel::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {

    // Check the current item.
    bool result = QSortFilterProxyModel::filterAcceptsRow(source_row, source_parent);
    QModelIndex currntIndex = sourceModel()->index(source_row, 0, source_parent);

    // If it has children, is a folder, so check each of his children.
    if (sourceModel()->hasChildren(currntIndex)) {
        for (int i = 0; i < sourceModel()->rowCount(currntIndex) && !result; ++i) {

            // Keep the parent if a children is shown.
            result = result || filterAcceptsRow(i, currntIndex);
        }
    }

    // Otherwise, it's a file. If we have the "Filter By Folder"...
    else if (filter_by_folder) {

        // Get his parent's parent, and check if that one is visible.
        QModelIndex parent = sourceModel()->parent(currntIndex);
        if (parent.isValid()) {
            QModelIndex parent2 = sourceModel()->parent(parent);
            if (parent2.isValid()) {
                result = QSortFilterProxyModel::filterAcceptsRow(parent.row(), parent2);
            }
        }
    }

    return result;
}

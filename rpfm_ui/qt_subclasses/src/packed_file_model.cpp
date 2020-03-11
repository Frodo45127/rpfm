#include "packed_file_model.h"

// Fuction to be able to create a PackedFileModel from other languages.
extern "C" QStandardItemModel* new_packed_file_model() {
    return dynamic_cast<QStandardItemModel*>(new PackedFileModel());
}

// Function to check if an item can be drag or drop into.
//
// TODO: Expand this to ensure only unique items can be drop into folders, so we don't have duplicate names in the same folder.
Qt::ItemFlags PackedFileModel::flags(const QModelIndex &index) const {
    Qt::ItemFlags defaultFlags = QStandardItemModel::flags(index);
    defaultFlags = defaultFlags &~ Qt::ItemIsDragEnabled;
    defaultFlags = defaultFlags &~ Qt::ItemIsDropEnabled;

    // In case of valid index, allow:
    // - Drag for everything except the PackFile
    // - Drop for eveything except files.
    if (index.isValid()) {
        QStandardItem* item = PackedFileModel::itemFromIndex(index);
        int item_type = item->data(20).toInt();
        if (item_type == 1) {
            return Qt::ItemIsDragEnabled | defaultFlags;
        }
        else if (item_type == 2) {
            return Qt::ItemIsDragEnabled | Qt::ItemIsDropEnabled | defaultFlags;
        }
        else if (item_type == 3) {
            return Qt::ItemIsDropEnabled | defaultFlags;
        }
        else {
            return defaultFlags ;
        }
    }

    // In case of invalid index, do not allow anything.
    else {
        return defaultFlags;
    }
}

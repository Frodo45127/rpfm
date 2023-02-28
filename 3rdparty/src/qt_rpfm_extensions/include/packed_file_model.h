#ifndef PACKED_FILE_MODEL_H
#define PACKED_FILE_MODEL_H

#include "qt_subclasses_global.h"
#include <QStandardItemModel>
#include <QStringListModel>
#include <QDropEvent>
#include <QDebug>
#include <QMimeData>

extern "C" QStandardItemModel* new_packed_file_model();

class PackedFileModel : public QStandardItemModel {
    Q_OBJECT
public:
    Qt::ItemFlags flags(const QModelIndex &index) const;
};

#endif // PACKED_FILE_MODEL_H

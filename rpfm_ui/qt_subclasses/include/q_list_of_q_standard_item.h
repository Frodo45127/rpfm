#ifndef Q_LIST_OF_Q_STANDARD_ITEM_H
#define Q_LIST_OF_Q_STANDARD_ITEM_H

#include "qt_subclasses_global.h"
#include <QList>
#include <QStandardItem>

extern "C" void add_to_q_list(QList<QStandardItem*>* list = nullptr, QStandardItem* item = nullptr);

#endif // Q_LIST_OF_Q_STANDARD_ITEM_H

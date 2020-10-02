#ifndef TABLEVIEW_FILTER_H
#define TABLEVIEW_FILTER_H

#include "qt_subclasses_global.h"
#include <QSortFilterProxyModel>
#include <QStandardItem>
#include <QList>
#include <QStringList>

extern "C" QSortFilterProxyModel* new_tableview_filter(QObject *parent = nullptr);
extern "C" void trigger_tableview_filter(
    QSortFilterProxyModel *filter = nullptr,
    QList<int> columns = QList<int>(),
    QStringList patterns = QStringList(),
    QList<int> case_sensitive = QList<int>()
);

class QTableViewSortFilterProxyModel : public QSortFilterProxyModel
{
    Q_OBJECT

public:
    QList<int> columns;
    QStringList patterns;
    QList<int> case_sensitive;

    explicit QTableViewSortFilterProxyModel(QObject *parent = nullptr);
    bool filterAcceptsRow(int source_row, const QModelIndex & source_parent) const;

signals:

private:
};

#endif // TABLEVIEW_FILTER_H

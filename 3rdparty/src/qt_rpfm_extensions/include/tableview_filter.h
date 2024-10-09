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
    QList<int> nott = QList<int>(),
    QList<int> regex = QList<int>(),
    QList<int> case_sensitive = QList<int>(),
    QList<int> show_blank_cells = QList<int>(),
    QList<int> match_groups_per_column = QList<int>(),
    QList<int> variant_to_search = QList<int>(),
    QList<int> show_edited_cells = QList<int>()
);

class QTableViewSortFilterProxyModel : public QSortFilterProxyModel
{
    Q_OBJECT

public:
    QList<int> columns;
    QStringList patterns;
    QList<int> nott;
    QList<int> regex;
    QList<int> case_sensitive;
    QList<int> show_blank_cells;
    QList<int> match_groups_per_column;
    QList<int> variant_to_search;
    QList<int> show_edited_cells;

    explicit QTableViewSortFilterProxyModel(QObject *parent = nullptr);
    bool filterAcceptsRow(int source_row, const QModelIndex & source_parent) const;


protected:
    bool lessThan(const QModelIndex &left, const QModelIndex &right) const;

private:

signals:

};

#endif // TABLEVIEW_FILTER_H

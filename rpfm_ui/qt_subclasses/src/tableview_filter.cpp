#include "tableview_filter.h"
#include <QSortFilterProxyModel>
#include <QItemSelection>
#include <QRegularExpression>
#include <QStandardItem>
#include <QStandardItemModel>

// Function to create the filter in a way that we don't need to bother Rust with new types.
extern "C" QSortFilterProxyModel* new_tableview_filter(QObject *parent) {
    QTableViewSortFilterProxyModel* filter = new QTableViewSortFilterProxyModel(parent);
    return dynamic_cast<QSortFilterProxyModel*>(filter);
}

// Funtion to trigger the filter we want, instead of the default one, from Rust.
extern "C" void trigger_tableview_filter(
    QSortFilterProxyModel* filter,
    QList<int> columns,
    QStringList patterns,
    QList<int> case_sensitive
) {
    QTableViewSortFilterProxyModel* filter2 = static_cast<QTableViewSortFilterProxyModel*>(filter);
    filter2->columns = columns;
    filter2->patterns = patterns;
    filter2->case_sensitive = case_sensitive;
    filter2->setFilterKeyColumn(0);
}

// Constructor of QTableViewSortFilterProxyModel.
QTableViewSortFilterProxyModel::QTableViewSortFilterProxyModel(QObject *parent): QSortFilterProxyModel(parent) {}

// Function called when the filter changes.
bool QTableViewSortFilterProxyModel::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {

    bool is_valid = true;

    for (int i = 0; i < columns.count(); ++i) {
        int column = columns.at(i);
        QString const pattern = patterns.at(i);
        Qt::CaseSensitivity case_sensitivity = static_cast<Qt::CaseSensitivity>(case_sensitive.at(i));

        QRegularExpression::PatternOptions options = QRegularExpression::PatternOptions();
        if (case_sensitivity == Qt::CaseSensitivity::CaseInsensitive) {
            options |= QRegularExpression::CaseInsensitiveOption;
        }

        QRegularExpression regex(pattern, options);
        QModelIndex currntIndex = sourceModel()->index(source_row, column, source_parent);
        QStandardItem *currntData = static_cast<QStandardItemModel*>(sourceModel())->itemFromIndex(currntIndex);
        if (currntIndex.isValid()) {

            // Checkbox matches.
            if (currntData->isCheckable()) {
                QString pattern_lower = pattern.toLower();
                bool isChecked = currntData->checkState() == Qt::CheckState::Checked;
                if ((pattern_lower == "true" || pattern_lower == "1") && !isChecked) {
                    is_valid = false;
                    break;
                } else if ((pattern_lower == "false" || pattern_lower == "0") && isChecked) {
                    is_valid = false;
                    break;
                }
            }

            // Text matches.
            else if (regex.isValid()) {
                QRegularExpressionMatch match = regex.match(currntIndex.data(2).toString());
                if (!match.hasMatch()) {
                    is_valid = false;
                    break;
                }
            }
            else {
                if (!currntIndex.data(2).toString().contains(pattern)) {
                    is_valid = false;
                    break;
                }
            }
        }
    }

    return is_valid;
}

// Function called when the filter changes.
bool QTableViewSortFilterProxyModel::lessThan(const QModelIndex &left, const QModelIndex &right) const {

    QStandardItem *leftData = static_cast<QStandardItemModel*>(sourceModel())->itemFromIndex(left);
    QStandardItem *rightData = static_cast<QStandardItemModel*>(sourceModel())->itemFromIndex(right);

    if (leftData->isCheckable() && rightData->isCheckable()) {
        if (leftData->checkState() == rightData->checkState()) {
            return false;
        } else if (leftData->checkState() == Qt::CheckState::Checked && rightData->checkState() == Qt::CheckState::Unchecked) {
            return false;
        } else {
            return true;
        }
    } else {
        return QSortFilterProxyModel::lessThan(left, right);
    }
}

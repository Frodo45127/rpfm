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
    QList<int> nott,
    QList<int> regex,
    QList<int> case_sensitive,
    QList<int> show_blank_cells,
    QList<int> match_groups_per_column,
    QList<int> variant_to_search,
    QList<int> show_edited_cells
) {
    QTableViewSortFilterProxyModel* filter2 = static_cast<QTableViewSortFilterProxyModel*>(filter);
    filter2->columns = columns;
    filter2->patterns = patterns;
    filter2->nott = nott;
    filter2->regex = regex;
    filter2->case_sensitive = case_sensitive;
    filter2->show_blank_cells = show_blank_cells;
    filter2->match_groups_per_column = match_groups_per_column;
    filter2->variant_to_search = variant_to_search;
    filter2->show_edited_cells = show_edited_cells;
    filter2->setFilterKeyColumn(0);
}

// Constructor of QTableViewSortFilterProxyModel.
QTableViewSortFilterProxyModel::QTableViewSortFilterProxyModel(QObject *parent): QSortFilterProxyModel(parent) {}

// Function called when the filter changes.
bool QTableViewSortFilterProxyModel::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {

    // If we have no filters, show everything.
    if (patterns.isEmpty()) {
        return true;
    }

    // First, split the matches in groups.
    QMap<int, QVector<int>> matches_per_group = QMap<int, QVector<int>>();

    // Initialize the groups so it doesn't explode later.
    for (int i = 0; i < match_groups_per_column.count(); ++i) {
        int group = match_groups_per_column.at(i);
        if (matches_per_group.contains(group)) {
            auto vec = QVector<int>();
            vec.append(i);
            matches_per_group.insert(group, vec);
        } else {
            matches_per_group[group].append(i);
        }
    }

    // Logic for groups:
    // - For a group to be valid, all matches on it must be valid (if one of them is not valid, the entire group is invalid).
    // - For a row to be valid, one of the group needs to be valid (we keep trying until we find a valid one).
    // This means we have to check one group at a time, and if one of them is valid, the full row is valid.
    for (const QVector<int>& group: qAsConst(matches_per_group)) {
        bool is_group_valid = true;

        // For each column, check if it's on the current group.
        for (int match: group) {

            // Ignore empty matches.
            QString pattern = patterns.at(match);
            if (pattern.isEmpty()) {
                continue;
            }

            int column = columns.at(match);
            bool use_regex = regex.at(match) == 1 ? true: false;
            bool use_nott = nott.at(match) == 1 ? true: false;
            Qt::CaseSensitivity case_sensitivity = static_cast<Qt::CaseSensitivity>(case_sensitive.at(match));
            bool show_blank_cells_in_column = show_blank_cells.at(match) == 1 ? true: false;
            bool show_edited_cells_in_column = show_edited_cells.at(match) == 1 ? true: false;

            QVector<int>* variants = new QVector<int>();
            if (variant_to_search.at(match) == 0) {
                variants->append(2);
            } else if (variant_to_search.at(match) == 1) {
                variants->append(40);
            } else {
                variants->append(2);
                variants->append(40);
            }

            QModelIndex currntIndex = sourceModel()->index(source_row, column, source_parent);
            QStandardItem *currntData = static_cast<QStandardItemModel*>(sourceModel())->itemFromIndex(currntIndex);

            QVariant isModifiedFromVanillaVariant = currntData->data(24);
            bool isModifiedFromVanilla = !isModifiedFromVanillaVariant.isNull() ? isModifiedFromVanillaVariant.toBool(): false;

            if (currntIndex.isValid()) {

                // If the variant is modified and we want to show modified cells, we let it pass the filters.
                if (show_edited_cells_in_column && isModifiedFromVanilla) {
                    continue;
                }

                // Checkbox matches.
                //
                // NOTE: isCheckable is broken if the cell is not editable.
                else if (currntData->data(Qt::CheckStateRole).isValid()) {
                    QString pattern_lower = pattern.toLower();
                    bool isChecked = currntData->checkState() == Qt::CheckState::Checked;

                    if (use_nott) {
                        isChecked = !isChecked;
                    }

                    if (
                        ((pattern_lower == "true" || pattern_lower == "1") && !isChecked) ||
                        ((pattern_lower == "false" || pattern_lower == "0") && isChecked)) {
                        is_group_valid = false;
                        break;
                    }
                }

                // In case of text, if it's empty we let it pass the filters.
                else if (show_blank_cells_in_column && currntData->data(2).toString().isEmpty()) {
                    continue;
                }

                // Float matches.
                // We need to do special stuff so they match against the formatted number, not against the unformatted one.
                //else if (currntData.data(2).toFloat(conversion_ok)) {

                //}

                // Text matches.
                else if (use_regex) {
                    if (use_nott) {
                        pattern = "^((?!" + pattern + ").)*$";
                    }

                    QRegularExpression::PatternOptions options = QRegularExpression::PatternOptions();
                    if (case_sensitivity == Qt::CaseSensitivity::CaseInsensitive) {
                        options |= QRegularExpression::CaseInsensitiveOption;
                    }

                    QRegularExpression regex(pattern, options);
                    if (regex.isValid()) {
                        for (int f = 0; f < variants->count(); ++f) {
                            QRegularExpressionMatch match = regex.match(currntData->data(variants->at(f)).toString());
                            if (!match.hasMatch()) {
                                is_group_valid = false;
                                break;
                            }
                        }

                        if (!is_group_valid) {
                            break;
                        }
                    }
                }
                else {
                    if (use_nott) {
                        for (int f = 0; f < variants->count(); ++f) {
                            if (currntData->data(variants->at(f)).toString().contains(pattern, case_sensitivity)) {
                                is_group_valid = false;
                                break;
                            }
                        }

                        if (!is_group_valid) {
                            break;
                        }
                    } else {
                        for (int f = 0; f < variants->count(); ++f) {
                            if (!currntData->data(variants->at(f)).toString().contains(pattern, case_sensitivity)) {
                                is_group_valid = false;
                                break;
                            }
                        }

                        if (!is_group_valid) {
                            break;
                        }
                    }
                }
            }
        }

        if (is_group_valid) {
            return is_group_valid;
        }
    }

    return false;
}

// Function called when the filter changes.
bool QTableViewSortFilterProxyModel::lessThan(const QModelIndex &left, const QModelIndex &right) const {

    QStandardItem *leftData = static_cast<QStandardItemModel*>(sourceModel())->itemFromIndex(left);
    QStandardItem *rightData = static_cast<QStandardItemModel*>(sourceModel())->itemFromIndex(right);

    // NOTE: isCheckable is broken if the cell is not editable.
    if (leftData->data(Qt::CheckStateRole).isValid() && rightData->data(Qt::CheckStateRole).isValid()) {
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

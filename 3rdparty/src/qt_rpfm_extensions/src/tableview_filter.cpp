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
    QList<int> show_edited_cells,
    QList<int> flagged_row_roles
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
    filter2->flagged_row_roles = flagged_row_roles;

    // Precompute the per-match caches so `filterAcceptsRow` runs in constant
    // time per row relative to the match count instead of recompiling regexes
    // and rebuilding QLists/QMaps for every source row.
    filter2->cached_regex.clear();
    filter2->cached_variants.clear();
    filter2->cached_groups.clear();

    int match_count = filter2->patterns.count();
    filter2->cached_regex.reserve(match_count);
    filter2->cached_variants.reserve(match_count);

    for (int i = 0; i < match_count; ++i) {

        // Precompile the regex once per match. If this match ends up in a
        // non-regex branch at runtime the cached entry is simply never read.
        QRegularExpression::PatternOptions opts = QRegularExpression::PatternOptions();
        Qt::CaseSensitivity cs = static_cast<Qt::CaseSensitivity>(filter2->case_sensitive.value(i, 0));
        if (cs == Qt::CaseInsensitive) {
            opts |= QRegularExpression::CaseInsensitiveOption;
        }

        QString pattern = filter2->patterns.at(i);
        bool is_regex = filter2->regex.value(i, 0) == 1;
        bool is_not = filter2->nott.value(i, 0) == 1;

        // Negated regex is implemented as a zero-width negative-lookahead
        // wrapper. Non-regex negation flips the containment test at match
        // time and must leave the pattern untouched.
        if (is_regex && is_not) {
            pattern = "^((?!" + pattern + ").)*$";
        }
        filter2->cached_regex.append(QRegularExpression(pattern, opts));

        // Variants list per match (roles 2 / 40 / both).
        QList<int> variants;
        int variant = filter2->variant_to_search.value(i, 0);
        if (variant == 0) {
            variants.append(2);
        } else if (variant == 1) {
            variants.append(40);
        } else {
            variants.append(2);
            variants.append(40);
        }
        filter2->cached_variants.append(variants);
    }

    // Group map from match indices, same priority as the old per-row build:
    // all matches in a group must pass, any single group passing accepts the
    // row.
    for (int i = 0; i < filter2->match_groups_per_column.count(); ++i) {
        int group = filter2->match_groups_per_column.at(i);
        if (!filter2->cached_groups.contains(group)) {
            filter2->cached_groups.insert(group, QList<int>{i});
        } else {
            filter2->cached_groups[group].append(i);
        }
    }

    filter2->setFilterKeyColumn(0);
}

// Constructor of QTableViewSortFilterProxyModel.
QTableViewSortFilterProxyModel::QTableViewSortFilterProxyModel(QObject *parent): QSortFilterProxyModel(parent) {}

// Function called when the filter changes.
bool QTableViewSortFilterProxyModel::filterAcceptsRow(int source_row, const QModelIndex &source_parent) const {

    // Hoisted once per row — the old code re-cast inside every match iteration.
    QStandardItemModel* model = static_cast<QStandardItemModel*>(sourceModel());

    // If flag-based row filtering is enabled, check if any cell in the row has one of the selected flags.
    if (!flagged_row_roles.isEmpty()) {
        int col_count = model->columnCount(source_parent);
        bool has_flag = false;

        for (int col = 0; col < col_count && !has_flag; ++col) {
            QModelIndex idx = model->index(source_row, col, source_parent);
            QStandardItem *item = model->itemFromIndex(idx);
            if (!item) continue;

            for (int role : flagged_row_roles) {
                QVariant v = item->data(role);
                if (!v.isNull() && v.toBool()) {
                    has_flag = true;
                    break;
                }
            }
        }

        if (!has_flag) {
            return false;
        }
    }

    // If we have no filters, show everything (unless already filtered by flags above).
    if (patterns.isEmpty()) {
        return true;
    }

    // Logic for groups:
    // - For a group to be valid, all matches on it must be valid (if one of them is not valid, the entire group is invalid).
    // - For a row to be valid, one of the group needs to be valid (we keep trying until we find a valid one).
    // The groups map itself is precomputed in `trigger_tableview_filter`.
    for (const QList<int>& group: std::as_const(cached_groups)) {
        bool is_group_valid = true;

        // For each column, check if it's on the current group.
        for (int match: group) {

            // Ignore empty matches.
            const QString& pattern = patterns.at(match);
            if (pattern.isEmpty()) {
                continue;
            }

            int column = columns.at(match);
            bool use_regex = regex.at(match) == 1;
            bool use_nott = nott.at(match) == 1;
            Qt::CaseSensitivity case_sensitivity = static_cast<Qt::CaseSensitivity>(case_sensitive.at(match));
            bool show_blank_cells_in_column = show_blank_cells.at(match) == 1;
            bool show_edited_cells_in_column = show_edited_cells.at(match) == 1;

            // Precomputed in `trigger_tableview_filter` — no per-row heap alloc
            // (the old code called `new QList<int>()` here and leaked it).
            const QList<int>& variants = cached_variants.at(match);

            QModelIndex currntIndex = model->index(source_row, column, source_parent);
            if (!currntIndex.isValid()) {
                continue;
            }

            QStandardItem *currntData = model->itemFromIndex(currntIndex);

            // Only fetch role 24 when the flag that actually consults it is on.
            // The old code fetched this for every row regardless.
            if (show_edited_cells_in_column) {
                QVariant modifiedVariant = currntData->data(24);
                if (!modifiedVariant.isNull() && modifiedVariant.toBool()) {
                    continue;
                }
            }

            // Checkbox matches.
            //
            // NOTE: isCheckable is broken if the cell is not editable.
            if (currntData->data(Qt::CheckStateRole).isValid()) {
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

            // Text matches via the cached, pre-compiled regex. The `use_nott`
            // regex-wrapping transformation was also baked in at cache time.
            else if (use_regex) {
                const QRegularExpression& re = cached_regex.at(match);
                if (re.isValid()) {
                    for (int v : variants) {
                        QRegularExpressionMatch m = re.match(currntData->data(v).toString());
                        if (!m.hasMatch()) {
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
                    for (int v : variants) {
                        if (currntData->data(v).toString().contains(pattern, case_sensitivity)) {
                            is_group_valid = false;
                            break;
                        }
                    }

                    if (!is_group_valid) {
                        break;
                    }
                } else {
                    for (int v : variants) {
                        if (!currntData->data(v).toString().contains(pattern, case_sensitivity)) {
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

        if (is_group_valid) {
            return true;
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

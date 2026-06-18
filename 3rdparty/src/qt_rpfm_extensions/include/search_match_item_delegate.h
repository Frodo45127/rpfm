#ifndef SEARCH_MATCH_ITEM_DELEGATE_H
#define SEARCH_MATCH_ITEM_DELEGATE_H

#include "qt_subclasses_global.h"
#include "extended_q_styled_item_delegate.h"
#include <QStyledItemDelegate>

extern "C" void new_search_match_item_delegate(QObject *parent = nullptr, int column = 0, bool is_dark_theme_enabled = false, bool has_filter = false);

// Delegate for the global search results tree. It renders a match row as a single line of
// context with the matched substring emphasised, reading the highlight range from the item's
// data roles. File (parent) rows fall back to the default painting so their icons are kept.
class QSearchMatchItemDelegate : public QExtendedStyledItemDelegate
{
    Q_OBJECT

public:

    explicit QSearchMatchItemDelegate(QObject *parent = nullptr, bool is_dark_theme_enabled = false, bool has_filter = false);
    void paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const override;
};
#endif // SEARCH_MATCH_ITEM_DELEGATE_H

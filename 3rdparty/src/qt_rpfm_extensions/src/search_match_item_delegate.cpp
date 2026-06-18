#include "search_match_item_delegate.h"

#include <QAbstractItemView>
#include <QAbstractTextDocumentLayout>
#include <QApplication>
#include <QFontMetrics>
#include <QPainter>
#include <QSortFilterProxyModel>
#include <QStandardItem>
#include <QStandardItemModel>
#include <QStyle>
#include <QTextDocument>
#include <QTextOption>

// Data roles carrying the highlight range, kept in sync with the Rust constants
// MATCH_HIGHLIGHT_START / MATCH_HIGHLIGHT_END in the global search UI.
static const int MATCH_HIGHLIGHT_START = 50;
static const int MATCH_HIGHLIGHT_END = 51;

// Function to be called from any other language. It assigns the delegate to the given column of the provided view.
extern "C" void new_search_match_item_delegate(QObject *parent, int column, bool is_dark_theme_enabled, bool has_filter) {
    QSearchMatchItemDelegate* delegate = new QSearchMatchItemDelegate(parent, is_dark_theme_enabled, has_filter);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

QSearchMatchItemDelegate::QSearchMatchItemDelegate(QObject *parent, bool is_dark_theme_enabled, bool has_filter): QExtendedStyledItemDelegate(parent, nullptr, is_dark_theme_enabled, has_filter, false) {
    use_filter = has_filter;
}

// Escapes the characters that would otherwise be interpreted as markup by the text document.
static QString html_escape(const QString& text) {
    QString out = text;
    out.replace('&', "&amp;");
    out.replace('<', "&lt;");
    out.replace('>', "&gt;");
    return out;
}

void QSearchMatchItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {

    // Resolve the source item behind the (possibly filtered) index so we can read the highlight roles.
    QStandardItem* item = nullptr;
    if (use_filter && index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        if (filterModel != nullptr) {
            const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());
            if (standardModel != nullptr)
                item = standardModel->itemFromIndex(filterModel->mapToSource(index));
        }
    }

    if (item == nullptr) {
        QExtendedStyledItemDelegate::paint(painter, option, index);
        return;
    }

    // File (parent) rows render as a full-width section banner: a faint accent band, the file name
    // in bold with its type icon, and the match count right-aligned. Relies on the row being
    // first-column-spanned so this cell covers the full width.
    if (item->parent() == nullptr) {
        QStyleOptionViewItem opt(option);
        initStyleOption(&opt, index);
        QStyle* style = opt.widget != nullptr ? opt.widget->style() : QApplication::style();

        bool selected = opt.state & QStyle::State_Selected;

        painter->save();
        if (selected) {
            painter->fillRect(opt.rect, opt.palette.color(QPalette::Highlight));
        } else {
            QColor band = opt.palette.color(QPalette::Highlight);
            band.setAlpha(28);
            painter->fillRect(opt.rect, band);
        }
        painter->restore();

        // Reserve room on the right for the count, then let the style draw the icon and bold name.
        QString countText = QString::number(item->rowCount());
        int countWidth = opt.fontMetrics.horizontalAdvance(countText) + 12;

        QStyleOptionViewItem textOpt(opt);
        textOpt.font.setBold(true);
        textOpt.backgroundBrush = Qt::NoBrush;
        textOpt.rect = opt.rect.adjusted(0, 0, -countWidth, 0);
        style->drawControl(QStyle::CE_ItemViewItem, &textOpt, painter, textOpt.widget);

        painter->save();
        painter->setPen(selected ? opt.palette.color(QPalette::HighlightedText)
                                  : opt.palette.color(QPalette::Disabled, QPalette::Text));
        painter->drawText(opt.rect.adjusted(0, 0, -6, 0), Qt::AlignRight | Qt::AlignVCenter, countText);
        painter->restore();
        return;
    }

    // Match rows without highlight data (e.g. unknown files) use the default painting.
    QVariant hlStartVariant = item->data(MATCH_HIGHLIGHT_START);
    QVariant hlEndVariant = item->data(MATCH_HIGHLIGHT_END);
    if (hlStartVariant.isNull() || hlEndVariant.isNull()) {
        QExtendedStyledItemDelegate::paint(painter, option, index);
        return;
    }

    QStyleOptionViewItem opt(option);
    initStyleOption(&opt, index);

    QStyle* style = opt.widget != nullptr ? opt.widget->style() : QApplication::style();

    // Let the style paint the background, selection and focus, but not the text: we draw the text
    // ourselves with the matched substring highlighted.
    QString fullText = opt.text;
    opt.text.clear();
    style->drawControl(QStyle::CE_ItemViewItem, &opt, painter, opt.widget);

    int hlStart = hlStartVariant.toInt();
    int hlEnd = hlEndVariant.toInt();
    if (hlStart < 0) hlStart = 0;
    if (hlEnd > fullText.length()) hlEnd = fullText.length();
    if (hlEnd < hlStart) hlEnd = hlStart;

    QString pre = html_escape(fullText.left(hlStart));
    QString matched = html_escape(fullText.mid(hlStart, hlEnd - hlStart));
    QString post = html_escape(fullText.mid(hlEnd));

    bool selected = opt.state & QStyle::State_Selected;
    QColor textColor = selected ? opt.palette.color(QPalette::HighlightedText) : opt.palette.color(QPalette::Text);

    QString matchSpan;
    if (selected) {

        // The row already uses the highlight colour as background, so a coloured box would vanish.
        // Emphasise the match with bold + underline instead.
        matchSpan = QString("<b><u>%1</u></b>").arg(matched);
    } else {
        QColor matchBg = opt.palette.color(QPalette::Highlight);
        QColor matchFg = opt.palette.color(QPalette::HighlightedText);
        matchSpan = QString("<span style=\"background-color:%1;color:%2;font-weight:bold;\">%3</span>")
            .arg(matchBg.name(), matchFg.name(), matched);
    }

    QString html = QString("<span style=\"color:%1;\">%2%3%4</span>")
        .arg(textColor.name(), pre, matchSpan, post);

    QTextDocument doc;
    doc.setDocumentMargin(0);
    doc.setDefaultFont(opt.font);
    QTextOption textOption(doc.defaultTextOption());
    textOption.setWrapMode(QTextOption::NoWrap);
    doc.setDefaultTextOption(textOption);
    doc.setHtml(html);

    QRect textRect = style->subElementRect(QStyle::SE_ItemViewItemText, &opt, opt.widget);

    painter->save();
    painter->setClipRect(textRect);

    // Vertically center the single line of text within the cell's text rect.
    qreal y = textRect.top() + (textRect.height() - doc.size().height()) / 2.0;
    painter->translate(textRect.left(), y);

    QAbstractTextDocumentLayout::PaintContext ctx;
    doc.documentLayout()->draw(painter, ctx);

    painter->restore();
}

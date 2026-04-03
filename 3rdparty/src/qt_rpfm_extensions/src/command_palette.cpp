#include "command_palette.h"

#include <QApplication>
#include <QKeyEvent>
#include <QVBoxLayout>
#include <QScreen>
#include <QStandardItem>
#include <QRegExp>
#include <QStyledItemDelegate>
#include <QPainter>
#include <QTextDocument>

// Custom delegate that renders items with an optional italic detail line below the main text.
class CommandPaletteDelegate : public QStyledItemDelegate {
public:
    explicit CommandPaletteDelegate(QObject* parent = nullptr) : QStyledItemDelegate(parent) {}

    void paint(QPainter* painter, const QStyleOptionViewItem& option, const QModelIndex& index) const override {
        QStyleOptionViewItem opt = option;
        initStyleOption(&opt, index);

        painter->save();

        // Draw the background (selection highlight, hover, etc).
        QStyle* style = opt.widget ? opt.widget->style() : QApplication::style();
        style->drawPrimitive(QStyle::PE_PanelItemViewItem, &opt, painter, opt.widget);

        QString displayText = index.data(Qt::DisplayRole).toString();
        QString detailText = index.data(Qt::UserRole + 1).toString();
        QIcon icon = index.data(Qt::DecorationRole).value<QIcon>();

        QRect contentRect = opt.rect.adjusted(6, 2, -6, -2);

        // Draw icon if present.
        int iconOffset = 0;
        if (!icon.isNull()) {
            int iconSize = 16;
            QRect iconRect(contentRect.x(), contentRect.y() + (contentRect.height() - iconSize) / 2, iconSize, iconSize);
            icon.paint(painter, iconRect);
            iconOffset = iconSize + 4;
        }

        QRect textRect = contentRect.adjusted(iconOffset, 0, 0, 0);

        if (detailText.isEmpty()) {
            // Single line: just the display text.
            painter->setFont(opt.font);
            painter->setPen(opt.palette.color(QPalette::Text));
            painter->drawText(textRect, Qt::AlignLeft | Qt::AlignVCenter, displayText);
        } else {
            // Two lines: display text on top, detail in italic below.
            int totalHeight = textRect.height();
            int topHeight = totalHeight * 3 / 5;
            int bottomHeight = totalHeight - topHeight;

            QRect topRect = textRect.adjusted(0, 0, 0, -bottomHeight);
            QRect bottomRect = textRect.adjusted(0, topHeight, 0, 0);

            // Main text.
            painter->setFont(opt.font);
            painter->setPen(opt.palette.color(QPalette::Text));
            painter->drawText(topRect, Qt::AlignLeft | Qt::AlignVCenter, displayText);

            // Detail text in italic, smaller, dimmed.
            QFont detailFont = opt.font;
            detailFont.setItalic(true);
            detailFont.setPointSizeF(detailFont.pointSizeF() * 0.85);
            painter->setFont(detailFont);
            painter->setPen(opt.palette.color(QPalette::Disabled, QPalette::Text));
            painter->drawText(bottomRect, Qt::AlignLeft | Qt::AlignVCenter, detailText);
        }

        painter->restore();
    }

    QSize sizeHint(const QStyleOptionViewItem& option, const QModelIndex& index) const override {
        QString detailText = index.data(Qt::UserRole + 1).toString();
        if (detailText.isEmpty()) {
            return QSize(option.rect.width(), 28);
        } else {
            return QSize(option.rect.width(), 44);
        }
    }
};

// FFI functions for Rust interop.
extern "C" QFrame* new_command_palette(QWidget* parent) {
    return new CommandPalettePopup(parent);
}

extern "C" void command_palette_show(QFrame* palette) {
    CommandPalettePopup* p = dynamic_cast<CommandPalettePopup*>(palette);
    if (p) p->showPalette();
}

extern "C" void command_palette_clear(QFrame* palette) {
    CommandPalettePopup* p = dynamic_cast<CommandPalettePopup*>(palette);
    if (p) p->clearItems();
}

extern "C" void command_palette_add_item(QFrame* palette, const QString* display_text, const QString* detail_text, const QIcon* icon) {
    CommandPalettePopup* p = dynamic_cast<CommandPalettePopup*>(palette);
    if (p && display_text) {
        p->addItem(*display_text, detail_text ? *detail_text : QString(), icon);
    }
}

extern "C" int command_palette_selected_index(QFrame* palette) {
    CommandPalettePopup* p = dynamic_cast<CommandPalettePopup*>(palette);
    return p ? p->activatedIndex() : -1;
}

extern "C" QString* command_palette_search_text(QFrame* palette) {
    CommandPalettePopup* p = dynamic_cast<CommandPalettePopup*>(palette);
    return p ? new QString(p->searchText()) : new QString();
}

// Constructor.
CommandPalettePopup::CommandPalettePopup(QWidget* parent)
    : QFrame(parent, Qt::Popup | Qt::FramelessWindowHint)
    , eventLoop(nullptr)
    , resultIndex(-1)
{
    setFixedWidth(600);
    setMaximumHeight(500);

    setStyleSheet(
        "CommandPalettePopup {"
        "  border: 1px solid palette(mid);"
        "  border-radius: 4px;"
        "}"
    );

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(4, 4, 4, 4);
    layout->setSpacing(2);

    // Search input at the top.
    searchInput = new QLineEdit(this);
    searchInput->setPlaceholderText("Search...");
    searchInput->setClearButtonEnabled(true);
    layout->addWidget(searchInput);

    // Results list below.
    resultsList = new QListView(this);
    resultsList->setEditTriggers(QAbstractItemView::NoEditTriggers);
    resultsList->setSelectionMode(QAbstractItemView::SingleSelection);
    resultsList->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    resultsList->setUniformItemSizes(false); // Items may have different heights (with/without detail).
    resultsList->setItemDelegate(new CommandPaletteDelegate(resultsList));
    layout->addWidget(resultsList);

    // Model + filter proxy.
    model = new QStandardItemModel(this);
    filterProxy = new QSortFilterProxyModel(this);
    filterProxy->setSourceModel(model);
    filterProxy->setFilterCaseSensitivity(Qt::CaseInsensitive);
    filterProxy->setFilterRole(Qt::DisplayRole);
    resultsList->setModel(filterProxy);

    // Connect text changes to filter updates.
    connect(searchInput, &QLineEdit::textChanged, this, &CommandPalettePopup::updateFilter);
    connect(searchInput, &QLineEdit::textChanged, this, &CommandPalettePopup::textChanged);

    // Double-click activates item.
    connect(resultsList, &QListView::activated, this, [this](const QModelIndex& proxyIndex) {
        QModelIndex sourceIndex = filterProxy->mapToSource(proxyIndex);
        resultIndex = sourceIndex.row();
        emit itemActivated(resultIndex);
        hide();
    });

    // Install event filter on the search input for keyboard navigation.
    searchInput->installEventFilter(this);
}

void CommandPalettePopup::showPalette() {
    resultIndex = -1;
    searchInput->clear();
    filterProxy->setFilterFixedString("");

    // Position centered at the top of the parent window.
    if (parentWidget()) {
        QPoint parentCenter = parentWidget()->mapToGlobal(
            QPoint(parentWidget()->width() / 2, 0)
        );
        int x = parentCenter.x() - width() / 2;
        int y = parentCenter.y() + 30;
        move(x, y);
    }

    // Resize height based on content, up to maximum.
    int rowCount = filterProxy->rowCount();
    int totalListHeight = 0;
    for (int i = 0; i < qMin(rowCount, 12); ++i) {
        QSize hint = resultsList->itemDelegate()->sizeHint(QStyleOptionViewItem(), filterProxy->index(i, 0));
        totalListHeight += hint.height();
    }
    totalListHeight += 4;
    int totalHeight = searchInput->sizeHint().height() + totalListHeight + 16;
    setFixedHeight(qMin(totalHeight, 500));

    show();
    searchInput->setFocus();

    // Select the first item.
    if (filterProxy->rowCount() > 0) {
        resultsList->setCurrentIndex(filterProxy->index(0, 0));
    }

    // Block until the popup is closed (by selection, Escape, or focus loss).
    eventLoop = new QEventLoop(this);
    eventLoop->exec();
    delete eventLoop;
    eventLoop = nullptr;
}

void CommandPalettePopup::hideEvent(QHideEvent* event) {
    QFrame::hideEvent(event);

    // Exit the blocking event loop when the popup is hidden for any reason.
    if (eventLoop && eventLoop->isRunning()) {
        eventLoop->quit();
    }
}

void CommandPalettePopup::clearItems() {
    model->clear();
    resultIndex = -1;
}

void CommandPalettePopup::addItem(const QString& displayText, const QString& detailText, const QIcon* icon) {
    auto* item = new QStandardItem(displayText);
    if (!detailText.isEmpty()) {
        item->setToolTip(detailText);
        item->setData(detailText, Qt::UserRole + 1);
    }
    if (icon && !icon->isNull()) {
        item->setIcon(*icon);
    }
    model->appendRow(item);
}

int CommandPalettePopup::activatedIndex() const {
    return resultIndex;
}

QString CommandPalettePopup::searchText() const {
    return searchInput->text();
}

void CommandPalettePopup::updateFilter(const QString& text) {
    // Strip the ">" command prefix for filtering purposes.
    QString filterText = text;
    if (filterText.startsWith(">")) {
        filterText = filterText.mid(1).trimmed();
    }

    // Build a fuzzy regex: split on whitespace, each word becomes ".*<escaped_word>",
    // so "db unit" matches anything containing "db" followed later by "unit".
    QStringList words = filterText.split(QRegExp("\\s+"), Qt::SkipEmptyParts);
    QString pattern;
    for (const QString& word : words) {
        if (!pattern.isEmpty()) {
            pattern += ".*";
        }
        pattern += QRegExp::escape(word);
    }
    filterProxy->setFilterRegExp(QRegExp(pattern, Qt::CaseInsensitive));

    // Resize height based on filtered content.
    int rowCount = filterProxy->rowCount();
    int totalListHeight = 0;
    for (int i = 0; i < qMin(rowCount, 12); ++i) {
        QSize hint = resultsList->itemDelegate()->sizeHint(QStyleOptionViewItem(), filterProxy->index(i, 0));
        totalListHeight += hint.height();
    }
    totalListHeight += 4;
    int totalHeight = searchInput->sizeHint().height() + totalListHeight + 16;
    setFixedHeight(qMin(totalHeight, 500));

    // Select the first visible item.
    if (filterProxy->rowCount() > 0) {
        resultsList->setCurrentIndex(filterProxy->index(0, 0));
    }
}

void CommandPalettePopup::activateSelected() {
    QModelIndex proxyIndex = resultsList->currentIndex();
    if (proxyIndex.isValid()) {
        QModelIndex sourceIndex = filterProxy->mapToSource(proxyIndex);
        resultIndex = sourceIndex.row();
        emit itemActivated(resultIndex);
        hide();
    }
}

bool CommandPalettePopup::eventFilter(QObject* obj, QEvent* event) {
    if (obj == searchInput && event->type() == QEvent::KeyPress) {
        QKeyEvent* keyEvent = static_cast<QKeyEvent*>(event);

        switch (keyEvent->key()) {
            case Qt::Key_Down: {
                int current = resultsList->currentIndex().row();
                int count = filterProxy->rowCount();
                if (current < count - 1) {
                    resultsList->setCurrentIndex(filterProxy->index(current + 1, 0));
                }
                return true;
            }
            case Qt::Key_Up: {
                int current = resultsList->currentIndex().row();
                if (current > 0) {
                    resultsList->setCurrentIndex(filterProxy->index(current - 1, 0));
                }
                return true;
            }
            case Qt::Key_Return:
            case Qt::Key_Enter:
                activateSelected();
                return true;
            case Qt::Key_Escape:
                resultIndex = -1;
                emit cancelled();
                hide();
                return true;
            default:
                break;
        }
    }

    return QFrame::eventFilter(obj, event);
}

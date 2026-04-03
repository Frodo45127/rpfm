#ifndef COMMAND_PALETTE_H
#define COMMAND_PALETTE_H

#include "qt_subclasses_global.h"
#include <QFrame>
#include <QLineEdit>
#include <QListView>
#include <QStandardItemModel>
#include <QSortFilterProxyModel>
#include <QEventLoop>

extern "C" QFrame* new_command_palette(QWidget* parent = nullptr);
extern "C" void command_palette_show(QFrame* palette);
extern "C" void command_palette_clear(QFrame* palette);
extern "C" void command_palette_add_item(QFrame* palette, const QString* display_text, const QString* detail_text, const QIcon* icon = nullptr);
extern "C" int command_palette_selected_index(QFrame* palette);
extern "C" QString* command_palette_search_text(QFrame* palette);

class CommandPalettePopup : public QFrame {
    Q_OBJECT

public:
    explicit CommandPalettePopup(QWidget* parent = nullptr);

    /// Shows the palette and blocks until the user selects an item or cancels.
    void showPalette();
    void clearItems();
    void addItem(const QString& displayText, const QString& detailText, const QIcon* icon = nullptr);

    /// Returns the source model index of the activated item, or -1 if cancelled.
    int activatedIndex() const;
    QString searchText() const;

signals:
    void itemActivated(int sourceIndex);
    void cancelled();
    void textChanged(const QString& text);

protected:
    bool eventFilter(QObject* obj, QEvent* event) override;
    void hideEvent(QHideEvent* event) override;

private:
    QLineEdit* searchInput;
    QListView* resultsList;
    QStandardItemModel* model;
    QSortFilterProxyModel* filterProxy;
    QEventLoop* eventLoop;
    int resultIndex;

    void updateFilter(const QString& text);
    void activateSelected();
};

#endif // COMMAND_PALETTE_H

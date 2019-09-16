#include "tableview_frozen.h"

#include <QScrollBar>
#include <QHeaderView>

// Fuction to be able to create a FrozenTableView from other languages, receiving it as a normal QTableView.
extern "C" QTableView* new_tableview_frozen(QAbstractItemModel* model, QTableView* frozen_table) {
    QTableViewFrozen* tableview = new QTableViewFrozen(model, frozen_table);
    return dynamic_cast<QTableView*>(tableview);
}

// Constructor of QTableViewFrozen. As we want to be able to manipulate the items inside, we provide it with
// a QAbstractItemModel and the QTableView we're going to freeze later on.
QTableViewFrozen::QTableViewFrozen(QAbstractItemModel * model, QTableView* tableview) {

    // Set the model, freeze the table and initialize it.
    setModel(model);
    frozenTableView = tableview;
    frozenTableView->setParent(this);
    init();

    // Connect the header's resize signal of both QTableViews together, so they keep the same size.
    connect(
        horizontalHeader(),
        &QHeaderView::sectionResized,
        this,
        &QTableViewFrozen::updateSectionWidth
    );

    connect(
        verticalHeader(),
        &QHeaderView::sectionResized,
        this,
        &QTableViewFrozen::updateSectionHeight
    );

    // Connect the vertical scrollbars, so both QTableViews are kept in sync.
    connect(
        frozenTableView->verticalScrollBar(),
        &QAbstractSlider::valueChanged,
        verticalScrollBar(),
        &QAbstractSlider::setValue
    );

    connect(
        verticalScrollBar(),
        &QAbstractSlider::valueChanged,
        frozenTableView->verticalScrollBar(),
        &QAbstractSlider::setValue
    );

    // Connect the sort indicators of both QTableViews, so they're kept in sync.
    connect(
        horizontalHeader(),
        &QHeaderView::sortIndicatorChanged,
        frozenTableView->horizontalHeader(),
        &QHeaderView::setSortIndicator
    );

    connect(
        frozenTableView->horizontalHeader(),
        &QHeaderView::sortIndicatorChanged,
        horizontalHeader(),
        &QHeaderView::setSortIndicator
    );
}

// Destructor. Nothing to see here, keep scrolling.
QTableViewFrozen::~QTableViewFrozen() {
    delete frozenTableView;
}

// QTableViewFrozen initializer. To prepare our frozenTableView to look and behave properly.
void QTableViewFrozen::init() {

    // Configure the Frozen QTableView to "fit" above the normal QTableView, using his same model.
    frozenTableView->setModel(model());
    frozenTableView->setFocusPolicy(Qt::NoFocus);
    frozenTableView->verticalHeader()->hide();
    frozenTableView->horizontalHeader()->setSectionResizeMode(QHeaderView::Fixed);

    // Configure (almost) the same way both tables.
    horizontalHeader()->setVisible(true);
    verticalHeader()->setVisible(true);
    horizontalHeader()->setSortIndicator(-1, Qt::SortOrder::AscendingOrder);
    setSortingEnabled(true);
    setAlternatingRowColors(true);
    horizontalHeader()->setSectionsMovable(true);
    setContextMenuPolicy(Qt::ContextMenuPolicy::CustomContextMenu);

    frozenTableView->horizontalHeader()->setSortIndicator(-1, Qt::SortOrder::AscendingOrder);
    frozenTableView->setSortingEnabled(true);
    frozenTableView->setAlternatingRowColors(true);
    frozenTableView->setContextMenuPolicy(Qt::ContextMenuPolicy::CustomContextMenu);

    // Place the Frozen QTableView above the normal one.
    viewport()->stackUnder(frozenTableView);

    frozenTableView->setSelectionModel(selectionModel());
    frozenTableView->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    frozenTableView->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    frozenTableView->show();

    updateFrozenTableGeometry();

    setHorizontalScrollMode(ScrollPerPixel);
    setVerticalScrollMode(ScrollPerPixel);
    frozenTableView->setVerticalScrollMode(ScrollPerPixel);
}

// Function to change the width columns at the same time we resize them in the main QTableView.
void QTableViewFrozen::updateSectionWidth(int logicalIndex, int /* oldSize */, int newSize) {
    frozenTableView->setColumnWidth(logicalIndex, newSize);
    updateFrozenTableGeometry();
}

// Function to change the height columns at the same time we resize them in the main QTableView.
void QTableViewFrozen::updateSectionHeight(int logicalIndex, int /* oldSize */, int newSize) {
    frozenTableView->setRowHeight(logicalIndex, newSize);
}

// Function to trigger a full geometry update of the frozen QTableView when we resize the main one.
void QTableViewFrozen::resizeEvent(QResizeEvent* event) {
    QTableView::resizeEvent(event);
    updateFrozenTableGeometry();
}

// Function to keep the cursor always visible, so it never gets hidden under the Frozen Columns.
QModelIndex QTableViewFrozen::moveCursor(
    CursorAction cursorAction,
    Qt::KeyboardModifiers modifiers
) {
    QModelIndex current = QTableView::moveCursor(cursorAction, modifiers);

    // We need to get this done dinamically, depending on the amount and size of frozen columns.
    int frozen_columns = 0;
    int frozen_width = 0;
    for (int i = 0; i < model()->columnCount(); ++i) {
        if (!frozenTableView->isColumnHidden(i)) {
            frozen_columns += 1;
            frozen_width += columnWidth(i);
        }
    }

    if (cursorAction == MoveLeft &&
        current.column() >= frozen_columns  &&
        visualRect(current).topLeft().x() < frozen_width
    ){
        const int newValue = horizontalScrollBar()->value() + visualRect(current).topLeft().x() - frozen_width;
        horizontalScrollBar()->setValue(newValue);
    }
    return current;
}

// Function to make the FrozenTableView work in consonance with the QtableView when the selection is out of view.
void QTableViewFrozen::scrollTo(const QModelIndex & index, ScrollHint hint){
    int frozen_columns = 0;
    for (int i = 0; i < model()->columnCount(); ++i) {
        if (!frozenTableView->isColumnHidden(i)) {
            frozen_columns += 1;
        }
    }

    if (index.column() >= frozen_columns) { QTableView::scrollTo(index, hint); }
}

// Function to update the geometry of the frozen QTableView when needed, to keep it at the right size.
void QTableViewFrozen::updateFrozenTableGeometry() {

    // It's simple, we get the width of every visible column, then use that as our width.
    int width = 0;
    for (int i = 0; i < model()->columnCount(); ++i) {
        if (!frozenTableView->isColumnHidden(i)) {
            width += columnWidth(i);
        }
    }
    frozenTableView->setGeometry(
        verticalHeader()->width() + frameWidth(),
        frameWidth(),
        width,
        viewport()->height()+horizontalHeader()->height()
    );
}

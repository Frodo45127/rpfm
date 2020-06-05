#include "tableview_frozen.h"

#include <QScrollBar>
#include <QHeaderView>
#include <QList>
#include <QSortFilterProxyModel>
#include <QAbstractScrollArea>

// Fuction to be able to create a QTableViewFrozen from other languages.
extern "C" QTableView* new_tableview_frozen(QWidget* parent) {
    QTableViewFrozen* tableview = new QTableViewFrozen(parent);
    return dynamic_cast<QTableView*>(tableview);
}

// Fuction to be able to get the internal frozen QTableView from other languages.
extern "C" QTableView* get_frozen_view(QTableView* tableView) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    return tableViewFrozen->tableViewFrozen;
}

// Function to set the model of the provided view and initialize it.
extern "C" void set_data_model(QTableView* tableView, QAbstractItemModel* model) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    tableViewFrozen->setDataModel(model);
}

// Function to freeze an specific column.
extern "C" void toggle_freezer(QTableView* tableView, int column) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    tableViewFrozen->toggleFreezer(column);
}

// Constructor of QTableViewFrozen.
QTableViewFrozen::QTableViewFrozen(QWidget* parent) {

    //this->setParent(parent);
    frozenColumns = QList<int>();
    tableViewFrozen = new QTableView(parent);

    // Connect the header's resize signal of both QTableViews together, so they keep the same size.
    //connect(
    //    horizontalHeader(),
    //    &QHeaderView::sectionResized,
    //    this,
    //    &QTableViewFrozen::updateSectionWidth
    //);
    //
    //connect(
    //    verticalHeader(),
    //    &QHeaderView::sectionResized,
    //    this,
    //    &QTableViewFrozen::updateSectionHeight
    //);
    //
    //// Connect the vertical scrollbars, so both QTableViews are kept in sync.
    //connect(
    //    tableViewFrozen->verticalScrollBar(),
    //    &QAbstractSlider::valueChanged,
    //    verticalScrollBar(),
    //    &QAbstractSlider::setValue
    //);
    //
    //connect(
    //    verticalScrollBar(),
    //    &QAbstractSlider::valueChanged,
    //    tableViewFrozen->verticalScrollBar(),
    //    &QAbstractSlider::setValue
    //);
    //
    //// Connect the sort indicators of both QTableViews, so they're kept in sync.
    //connect(
    //    horizontalHeader(),
    //    &QHeaderView::sortIndicatorChanged,
    //    tableViewFrozen->horizontalHeader(),
    //    &QHeaderView::setSortIndicator
    //);
    //
    //connect(
    //    tableViewFrozen->horizontalHeader(),
    //    &QHeaderView::sortIndicatorChanged,
    //    horizontalHeader(),
    //    &QHeaderView::setSortIndicator
    //);
}

// Destructor. Nothing to see here, keep scrolling.
QTableViewFrozen::~QTableViewFrozen() {
    delete tableViewFrozen;
}

void QTableViewFrozen::setDataModel(QAbstractItemModel * model) {
    setModel(model);
    init();
}

// QTableViewFrozen initializer. To prepare our frozenTableView to look and behave properly.
void QTableViewFrozen::init() {

    // Configure the QTableViews to "fit" above the normal QTableView, using his same model.
    tableViewFrozen->setModel(model());
    tableViewFrozen->setFocusPolicy(Qt::NoFocus);
    tableViewFrozen->verticalHeader()->hide();
    tableViewFrozen->horizontalHeader()->setSectionResizeMode(QHeaderView::Fixed);

    // Configure (almost) the same way both tables.
    horizontalHeader()->setVisible(true);
    verticalHeader()->setVisible(true);
    horizontalHeader()->setSortIndicator(-1, Qt::SortOrder::AscendingOrder);
    setSortingEnabled(true);
    setAlternatingRowColors(true);
    horizontalHeader()->setSectionsMovable(true);
    setContextMenuPolicy(Qt::ContextMenuPolicy::CustomContextMenu);

    tableViewFrozen->horizontalHeader()->setSortIndicator(-1, Qt::SortOrder::AscendingOrder);
    tableViewFrozen->setSortingEnabled(true);
    tableViewFrozen->setAlternatingRowColors(true);
    tableViewFrozen->setContextMenuPolicy(Qt::ContextMenuPolicy::CustomContextMenu);

    // Place the Frozen QTableView above the normal one.
    viewport()->stackUnder(tableViewFrozen);

    tableViewFrozen->setSelectionModel(selectionModel());
    tableViewFrozen->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    tableViewFrozen->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    tableViewFrozen->show();

    QSortFilterProxyModel* filterModelFrozen = dynamic_cast<QSortFilterProxyModel*>(this->model());
    for(int col = 0; col < filterModelFrozen->sourceModel()->columnCount(); col++)
        tableViewFrozen->setColumnHidden(col, true);

    setHorizontalScrollMode(ScrollPerPixel);
    setVerticalScrollMode(ScrollPerPixel);
    tableViewFrozen->setHorizontalScrollMode(ScrollPerPixel);
    tableViewFrozen->setVerticalScrollMode(ScrollPerPixel);

    tableViewFrozen->setStyleSheet("QTableView { "
        "border: none;"
        "selection-background-color: #999}"
    );

    //updateFrozenTableGeometry();
}

// Function to change the width columns at the same time we resize them in the main QTableView.
//void QTableViewFrozen::updateSectionWidth(int logicalIndex, int /* oldSize */, int newSize) {
//    tableViewFrozen->setColumnWidth(logicalIndex, newSize);
//    updateFrozenTableGeometry();
//}

// Function to change the height columns at the same time we resize them in the main QTableView.
//void QTableViewFrozen::updateSectionHeight(int logicalIndex, int /* oldSize */, int newSize) {
//    tableViewFrozen->setRowHeight(logicalIndex, newSize);
//}

// Function to trigger a full geometry update of the frozen QTableView when we resize the main one.
//void QTableViewFrozen::resizeEvent(QResizeEvent* event) {
//    QTableView::resizeEvent(event);
//    updateFrozenTableGeometry();
//}

// Function to keep the cursor always visible, so it never gets hidden under the Frozen Columns.
//QModelIndex QTableViewFrozen::moveCursor(
//    CursorAction cursorAction,
//    Qt::KeyboardModifiers modifiers
//) {
//    QModelIndex current = QTableView::moveCursor(cursorAction, modifiers);
//
//    // We need to get this done dinamically, depending on the amount and size of frozen columns.
//    int frozen_columns = 0;
//    int frozen_width = 0;
//    QSortFilterProxyModel* model = dynamic_cast<QSortFilterProxyModel*>(this->model());
//    for (int i = 0; i < model->sourceModel()->columnCount(); ++i) {
//        if (!tableViewFrozen->isColumnHidden(i)) {
//            frozen_columns += 1;
//            frozen_width += columnWidth(i);
//        }
//    }
//
//    if (cursorAction == MoveLeft &&
//        current.column() >= frozen_columns  &&
//        visualRect(current).topLeft().x() < frozen_width
//    ){
//        const int newValue = horizontalScrollBar()->value() + visualRect(current).topLeft().x() - frozen_width;
//        horizontalScrollBar()->setValue(newValue);
//    }
//    return current;
//}

// Function to make the FrozenTableView work in consonance with the QtableView when the selection is out of view.
//void QTableViewFrozen::scrollTo(const QModelIndex & index, ScrollHint hint) {
//    if (index.column() >= frozenColumns.count()) {
//        QTableView::scrollTo(index, hint);
//    }
//}

// Function to update the geometry of the frozen QTableView when needed, to keep it at the right size.
//void QTableViewFrozen::updateFrozenTableGeometry() {
//
//    // It's simple, we get the width of every visible column, then use that as our width.
//    int width = 0;
//    QSortFilterProxyModel* frozenTableModel = dynamic_cast<QSortFilterProxyModel*>(tableViewFrozen->model());
//    width += frozenColumns.count();
//    for (int i = 0; i < frozenTableModel->sourceModel()->columnCount(); ++i) {
//        if (frozenColumns.contains(i)) {
//            width += columnWidth(i);
//        }
//    }
//
//    if (frozenColumns.isEmpty()){
//        tableViewFrozen->verticalHeader()->hide();
//    }
//    else {
//        tableViewFrozen->verticalHeader()->show();
//    }
//
//    QMargins margins = viewportMargins();
//    margins.setLeft(verticalHeader()->width() + frameWidth() + width);
//    setViewportMargins(margins);
//
//    tableViewFrozen->setGeometry(
//        verticalHeader()->width() + frameWidth(),
//        frameWidth(),
//        width,
//        viewport()->height()+horizontalHeader()->height()
//    );
//}

void QTableViewFrozen::toggleFreezer(int column) {

    if (frozenColumns.contains(column)) {
        frozenColumns.removeOne(column);
        tableViewFrozen->setColumnHidden(column, true);
    }
    else {
        frozenColumns.append(column);
        tableViewFrozen->setColumnHidden(column, false);
    }
    //updateFrozenTableGeometry();
}

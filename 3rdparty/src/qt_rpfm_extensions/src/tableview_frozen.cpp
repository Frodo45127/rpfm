#include "tableview_frozen.h"
#include "extended_q_styled_item_delegate.h"

#include <QAbstractItemDelegate>
#include <QAbstractScrollArea>
#include <QHeaderView>
#include <QList>
#include <QSortFilterProxyModel>
#include <QScrollBar>
#include <QHelpEvent>
#include <QToolTip>
#include <QByteArray>
#include <QBuffer>
#include <QImage>
#include <QStandardItem>
#include <QStandardItemModel>
#include <QHelpEvent>
#include <QDebug>

//-----------------------------------------------------------
// FFI calls.
//-----------------------------------------------------------

// Fuction to be able to create a QTableViewFrozen from other languages.
extern "C" QTableView* new_tableview_frozen(QWidget* parent, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY)) {
    QTableViewFrozen* tableview = new QTableViewFrozen(parent, generate_tooltip_message);
    return dynamic_cast<QTableView*>(tableview);
}

// Function to freeze an specific column.
extern "C" void toggle_freezer(QTableView* tableView, int column) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    tableViewFrozen->toggleFreezer(column);
}

//-----------------------------------------------------------
// Private calls.
//-----------------------------------------------------------

// Constructor of QTableViewFrozen.
QTableViewFrozen::QTableViewFrozen(QWidget* parent, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY)) {

    this->setParent(parent);
    frozenColumns = QList<int>();
    tableViewFrozen = new QTableViewSubFrozen(this, generateTooltipMessage);
    generateTooltipMessage = generate_tooltip_message;

    if (generateTooltipMessage != nullptr) {
        qDebug("non-nul");
    }

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

    // Connect the header's move signal of both QTableViews together, so they keep the order.
    connect(
        horizontalHeader(),
        &QHeaderView::sectionMoved,
        this,
        &QTableViewFrozen::sectionMoved
    );

    // Connect the vertical scrollbars, so both QTableViews are kept in sync.
    connect(
        tableViewFrozen->verticalScrollBar(),
        &QAbstractSlider::valueChanged,
        verticalScrollBar(),
        &QAbstractSlider::setValue
    );

    connect(
        verticalScrollBar(),
        &QAbstractSlider::valueChanged,
        tableViewFrozen->verticalScrollBar(),
        &QAbstractSlider::setValue
    );

    // Connect the sort indicators of both QTableViews, so they're kept in sync.
    connect(
        horizontalHeader(),
        &QHeaderView::sortIndicatorChanged,
        tableViewFrozen->horizontalHeader(),
        &QHeaderView::setSortIndicator
    );

    connect(
        tableViewFrozen->horizontalHeader(),
        &QHeaderView::sortIndicatorChanged,
        horizontalHeader(),
        &QHeaderView::setSortIndicator
    );

    // Configure the QTableViews to "fit" above the normal QTableView, using his same model.
    tableViewFrozen->setFocusPolicy(Qt::NoFocus);
    tableViewFrozen->verticalHeader()->hide();
    tableViewFrozen->horizontalHeader()->setSectionResizeMode(QHeaderView::Fixed);

    // Configure (almost) the same way both tables.
    horizontalHeader()->setSectionsMovable(true);
    horizontalHeader()->setSortIndicator(-1, Qt::SortOrder::AscendingOrder);
    horizontalHeader()->setVisible(true);
    verticalHeader()->setVisible(true);

    setMouseTracking(true);
    setSortingEnabled(true);
    setAlternatingRowColors(true);
    setContextMenuPolicy(Qt::ContextMenuPolicy::CustomContextMenu);
    setHorizontalScrollMode(ScrollPerPixel);
    setVerticalScrollMode(ScrollPerPixel);

    tableViewFrozen->setMouseTracking(true);
    tableViewFrozen->setSortingEnabled(true);
    tableViewFrozen->setAlternatingRowColors(true);
    tableViewFrozen->setContextMenuPolicy(Qt::ContextMenuPolicy::CustomContextMenu);
    tableViewFrozen->setHorizontalScrollMode(ScrollPerPixel);
    tableViewFrozen->setVerticalScrollMode(ScrollPerPixel);
    tableViewFrozen->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    tableViewFrozen->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    tableViewFrozen->show();

    // Place the Frozen QTableView above the normal one.
    viewport()->stackUnder(tableViewFrozen);

    tableViewFrozen->setStyleSheet("QTableView { "
        "border: none;"
        "selection-background-color: #999}"
    );
}

//-----------------------------------------------------------
// Simple override calls & simple extension functions.
//-----------------------------------------------------------

// Destructor. Nothing to see here, keep scrolling.
QTableViewFrozen::~QTableViewFrozen() {
    delete tableViewFrozen;
}

// Override of setModel to assign the same model to both views.
void QTableViewFrozen::setModel(QAbstractItemModel* model) {
    tableViewFrozen->setModel(model);
    QTableView::setModel(model);

    // Connect the selection of both views, so they're kept in sync.
    // These are here because they need to trigger after setModel.
    connect(
        selectionModel(),
        &QItemSelectionModel::selectionChanged,
        this,
        &QTableViewFrozen::updateSelectionNormalToFrozen
    );

    connect(
        tableViewFrozen->selectionModel(),
        &QItemSelectionModel::selectionChanged,
        this,
        &QTableViewFrozen::updateSelectionFrozenToNormal
    );

    // Update the geometry, just in case we already have data in the model.
    updateFrozenTableGeometry();
}

// Function to change the width columns at the same time we resize them in the main QTableView.
void QTableViewFrozen::updateSectionWidth(int logicalIndex, int /* oldSize */, int newSize) {
    tableViewFrozen->setColumnWidth(logicalIndex, newSize);
    updateFrozenTableGeometry();
}

// Function to change the height columns at the same time we resize them in the main QTableView.
void QTableViewFrozen::updateSectionHeight(int logicalIndex, int /* oldSize */, int newSize) {
    tableViewFrozen->setRowHeight(logicalIndex, newSize);
}

// Function to trigger a full geometry update of the frozen QTableView when we resize the main one.
void QTableViewFrozen::resizeEvent(QResizeEvent* event) {
    QTableView::resizeEvent(event);
    updateFrozenTableGeometry();
}

// Function to properly keep in sync both views when enabling/disabling updates.
void QTableViewFrozen::setUpdatesEnabled(bool enable) {
    QTableView::setUpdatesEnabled(enable);
    tableViewFrozen->setUpdatesEnabled(enable);
}

// Function to apply any delegate (like the one that makes keys yellow) to both tables.
void QTableViewFrozen::setItemDelegateForColumn(int column, QAbstractItemDelegate* delegate) {
    QTableView::setItemDelegateForColumn(column, delegate);

    QExtendedStyledItemDelegate* delegateExtended = dynamic_cast<QExtendedStyledItemDelegate*>(delegate);
    QExtendedStyledItemDelegate* delegateFrozen = new QExtendedStyledItemDelegate(tableViewFrozen, nullptr, delegateExtended->dark_theme, delegateExtended->use_filter, delegateExtended->use_right_side_mark);
    tableViewFrozen->setItemDelegateForColumn(column, delegateFrozen);
}

// Function to update the geometry of the frozen QTableView when needed, to keep it at the right size.
void QTableViewFrozen::sectionMoved(int, int oldVisualIndex, int newVisualIndex) {
    tableViewFrozen->horizontalHeader()->moveSection(oldVisualIndex, newVisualIndex);
}

//-----------------------------------------------------------
// Complex override calls/extension functions.
//-----------------------------------------------------------

// Function to update the selection in the frozen table sync.
void QTableViewFrozen::updateSelectionNormalToFrozen(const QItemSelection &selected, const QItemSelection &deselected) {
    QModelIndexList items_sel = selected.indexes();
    QModelIndexList items_desel = deselected.indexes();

    QItemSelectionModel* oppositeSelectionModel = tableViewFrozen->selectionModel();
    for (const QModelIndex& index : qAsConst(items_sel)) {
        oppositeSelectionModel->select(index, QItemSelectionModel::SelectionFlag::Select);
    }

    for (const QModelIndex& index : qAsConst(items_desel)) {
        oppositeSelectionModel->select(index, QItemSelectionModel::SelectionFlag::Deselect);
    }
}

// Function to update the selection in the normal table sync.
void QTableViewFrozen::updateSelectionFrozenToNormal(const QItemSelection &selected, const QItemSelection &deselected) {
    QModelIndexList items_sel = selected.indexes();
    QModelIndexList items_desel = deselected.indexes();

    QItemSelectionModel* oppositeSelectionModel = selectionModel();
    for (const QModelIndex& index : qAsConst(items_sel)) {
        oppositeSelectionModel->select(index, QItemSelectionModel::SelectionFlag::Select);
    }

    for (const QModelIndex& index : qAsConst(items_desel)) {
        oppositeSelectionModel->select(index, QItemSelectionModel::SelectionFlag::Deselect);
    }
}

// Function so moving from a frozen column to a normal one and viceversa is seamless.
QModelIndex QTableViewFrozen::moveCursor(CursorAction cursorAction, Qt::KeyboardModifiers modifiers) {

    // Check what widget has the focus to know from what to what move.
    bool frozenFocus = tableViewFrozen->hasFocus();
    if (frozenFocus) {

        // Get the latest frozen column.
        int column = -1;

        for (int i = 0; i < frozenColumns.count(); ++i) {
            int vis = tableViewFrozen->horizontalHeader()->visualIndex(frozenColumns[i]);
            if (vis > column) {
                column = vis;
            }
        }

        if (cursorAction == MoveRight && tableViewFrozen->horizontalHeader()->visualIndex(tableViewFrozen->currentIndex().column()) == column) {
            QModelIndex precurrent = tableViewFrozen->currentIndex();
            QModelIndex current = tableViewFrozen->moveCursor2(cursorAction, modifiers);

            int row = precurrent.row();
            int column = 9999;
            int columnLogic = 9999;

            // We need to get this done dinamically, depending on the amount and size of frozen columns.
            for (int i = 0; i < horizontalHeader()->count(); ++i) {
                int vis = horizontalHeader()->visualIndex(i);
                if (vis < column) {
                    column = vis;
                    columnLogic = i;
                }
            }

            current = tableViewFrozen->model()->index(row, columnLogic);
            //tableViewFrozen->setCurrentIndex(current);
            setFocus();
            setCurrentIndex(current);

            return current;
        } else {
            QModelIndex current = tableViewFrozen->moveCursor2(cursorAction, modifiers);
            return current;
        }
    } else if (cursorAction == MoveLeft && !frozenColumns.isEmpty() && horizontalHeader()->visualIndex(currentIndex().column()) == 0) {
        QModelIndex precurrent = currentIndex();
        QModelIndex current = QTableView::moveCursor(cursorAction, modifiers);

        int row = precurrent.row();
        int column = -1;
        int columnLogic = -1;

        // We need to get this done dinamically, depending on the amount and size of frozen columns.
        for (int i = 0; i < frozenColumns.count(); ++i) {
            int vis = tableViewFrozen->horizontalHeader()->visualIndex(frozenColumns[i]);
            if (vis > column) {
                column = vis;
                columnLogic = frozenColumns[i];
            }
        }

        current = model()->index(row, columnLogic);
        //setCurrentIndex(current);
        tableViewFrozen->setFocus();
        tableViewFrozen->setCurrentIndex(current);

        return current;
    } else {
        QModelIndex current = QTableView::moveCursor(cursorAction, modifiers);
        return current;
    }
}

// Function to make the FrozenTableView work in consonance with the QtableView when the selection is out of view.
void QTableViewFrozen::scrollTo(const QModelIndex & index, ScrollHint hint) {
    if (index.column() >= frozenColumns.count()) {
        QTableView::scrollTo(index, hint);
    }
}

// Function to update the geometry of the frozen QTableView when needed, to keep it at the right size.
void QTableViewFrozen::updateFrozenTableGeometry() {

    // It's simple, we get the width of every frozen column, then use that as our width.
    int width = 0;
    QSortFilterProxyModel* frozenTableModel = dynamic_cast<QSortFilterProxyModel*>(tableViewFrozen->model());

    for (int i = 0; i < frozenTableModel->sourceModel()->columnCount(); ++i) {
        bool is_frozen = frozenColumns.contains(i);

        tableViewFrozen->setColumnHidden(i, !is_frozen);

        if (is_frozen) {
            width += columnWidth(i);
        }
    }

    // Fixes misaligned headers due to icons.
    tableViewFrozen->horizontalHeader()->setFixedSize(horizontalHeader()->size());

    if (baseLeftMargin != -1) {
        setViewportMargins(baseLeftMargin + width, viewportMargins().top(), viewportMargins().right(), viewportMargins().bottom());
    }

    if (frozenColumns.isEmpty()) {
        tableViewFrozen->setGeometry(
            frameWidth(),
            frameWidth(),
            width,
            viewport()->height()+horizontalHeader()->height()
        );
    } else {
        tableViewFrozen->setGeometry(
            frameWidth(),
            frameWidth(),
            verticalHeader()->width() + width,
            viewport()->height()+horizontalHeader()->height()
            );
    }
}

// Function to freeze/unfreeze the provided column.
void QTableViewFrozen::toggleFreezer(int column) {
    if (baseLeftMargin == -1) {
        baseLeftMargin = viewportMargins().left();
    }

    if (frozenColumns.contains(column)) {
        frozenColumns.removeOne(column);
    }
    else {
        frozenColumns.append(column);;
    }

    // Show/hide the row count in the frozen table.
    tableViewFrozen->verticalHeader()->setVisible(!frozenColumns.isEmpty());
    //verticalHeader()->setVisible(frozenColumns.isEmpty());

    // Only allow manually moving columns if we don't have frozen columns. Otherwise the order gets wonky as fuck.
    //if (frozenColumns.isEmpty()) {
    //    horizontalHeader()->setSectionsMovable(true);
    //} else {
    //    horizontalHeader()->setSectionsMovable(false);
    //}

    updateGeometry();
    updateFrozenTableGeometry();
}

bool QTableViewFrozen::viewportEvent(QEvent *event) {
    if (event->type() == QEvent::ToolTip) {
        _lastPosition = static_cast<QHelpEvent*>(event)->globalPos();
        QTableView* view = static_cast<QTableView*>(this);

        if (generateTooltipMessage != nullptr) {
            generateTooltipMessage(view, _lastPosition.x(), _lastPosition.y());
        }
    }
    return QTableView::viewportEvent(event);
}




QTableViewSubFrozen::QTableViewSubFrozen(QWidget* parent, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY)) {
    this->setParent(parent);
    generateTooltipMessage = generate_tooltip_message;

    if (generateTooltipMessage != nullptr) {
        qDebug("non-nul");
    }
}

// Destructor. Nothing to see here, keep scrolling.
QTableViewSubFrozen::~QTableViewSubFrozen() {
    delete this;
}

bool QTableViewSubFrozen::viewportEvent(QEvent *event) {
    if (event->type() == QEvent::ToolTip) {
        _lastPosition = static_cast<QHelpEvent*>(event)->globalPos();
        QTableView* view = static_cast<QTableView*>(this);

        if (generateTooltipMessage != nullptr) {
            generateTooltipMessage(view, _lastPosition.x(), _lastPosition.y());
        }
    }
    return QTableView::viewportEvent(event);
}

QModelIndex QTableViewSubFrozen::moveCursor2(CursorAction cursorAction, Qt::KeyboardModifiers modifiers) {
    QModelIndex current = QTableView::moveCursor(cursorAction, modifiers);
    return current;
}

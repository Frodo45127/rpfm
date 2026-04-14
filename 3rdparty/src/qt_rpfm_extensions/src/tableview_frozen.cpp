#include "tableview_frozen.h"

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

// Fuction to be able to create a QTableViewFrozen from other languages.
extern "C" QTableView* new_tableview_frozen(QWidget* parent, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY)) {
    QTableViewFrozen* tableview = new QTableViewFrozen(parent, generate_tooltip_message);
    return dynamic_cast<QTableView*>(tableview);
}

// Function to freeze an specific column.
extern "C" void toggle_freezer(QTableView* tableView, int column) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    if (tableViewFrozen) {
        tableViewFrozen->toggleFreezer(column);
    }
}

// Function to get the inner frozen QTableView from a QTableViewFrozen.
extern "C" QTableView* get_frozen_view(QTableView* tableView) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    if (tableViewFrozen) {
        return tableViewFrozen->tableViewFrozen;
    }
    return nullptr;
}

// Function to force a geometry update on the frozen view overlay.
extern "C" void update_frozen_view_geometry(QTableView* tableView) {
    QTableViewFrozen* tableViewFrozen = dynamic_cast<QTableViewFrozen*>(tableView);
    if (tableViewFrozen) {
        tableViewFrozen->updateFrozenTableGeometry();
    }
}

//-----------------------------------------------------------
// Private calls.
//-----------------------------------------------------------

// Constructor of QTableViewFrozen.
QTableViewFrozen::QTableViewFrozen(QWidget* parent, void (*generate_tooltip_message)(QTableView* view, int globalPosX, int globalPosY)) {

    this->setParent(parent);
    frozenColumns = QList<int>();
    sortSyncInProgress = false;
    tableViewFrozen = new QTableView(this);
    generateTooltipMessage = generate_tooltip_message;

    // Connect the header's resize signal of both QTableViews together, so they keep the same size.
    // Use blockSignals to prevent feedback loops between main↔frozen resize syncs.
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

    // Sync column resizes from the frozen view back to the main view.
    connect(
        tableViewFrozen->horizontalHeader(),
        &QHeaderView::sectionResized,
        this,
        [this](int logicalIndex, int /*oldSize*/, int newSize) {
            horizontalHeader()->blockSignals(true);
            setColumnWidth(logicalIndex, newSize);
            horizontalHeader()->blockSignals(false);
            updateFrozenTableGeometry();
        }
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
    // Use a guard flag to prevent infinite recursion from bidirectional connections.
    connect(
        horizontalHeader(),
        &QHeaderView::sortIndicatorChanged,
        this,
        [this](int logicalIndex, Qt::SortOrder order) {
            if (!sortSyncInProgress) {
                sortSyncInProgress = true;
                tableViewFrozen->horizontalHeader()->setSortIndicator(logicalIndex, order);
                sortSyncInProgress = false;
            }
        }
    );

    connect(
        tableViewFrozen->horizontalHeader(),
        &QHeaderView::sortIndicatorChanged,
        this,
        [this](int logicalIndex, Qt::SortOrder order) {
            if (!sortSyncInProgress) {
                sortSyncInProgress = true;
                horizontalHeader()->setSortIndicator(logicalIndex, order);
                sortSyncInProgress = false;
            }
        }
    );

    // Configure the QTableViews to "fit" above the normal QTableView, using his same model.
    tableViewFrozen->setFocusPolicy(Qt::NoFocus);
    tableViewFrozen->verticalHeader()->hide();
    tableViewFrozen->horizontalHeader()->setSectionResizeMode(QHeaderView::Interactive);
    tableViewFrozen->horizontalHeader()->setContextMenuPolicy(Qt::CustomContextMenu);

    // Forward the frozen header's context menu signal to the main header, so a single
    // Rust slot can handle both. The position is mapped to the main header's coordinate space.
    connect(
        tableViewFrozen->horizontalHeader(),
        &QWidget::customContextMenuRequested,
        this,
        [this](const QPoint& pos) {
            // Map the frozen header position to a logical index, then to the main header's coordinate.
            int logicalIndex = tableViewFrozen->horizontalHeader()->logicalIndexAt(pos.x());
            if (logicalIndex >= 0) {
                // Emit the main header's signal with a position that resolves to the same logical index.
                int sectionPos = horizontalHeader()->sectionViewportPosition(logicalIndex);
                QPoint mainPos(sectionPos + 1, pos.y());
                emit horizontalHeader()->customContextMenuRequested(mainPos);
            }
        }
    );

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
    // Start hidden — it will be shown by updateFrozenTableGeometry when columns are frozen.
    tableViewFrozen->hide();

    // Place the Frozen QTableView above the normal one.
    viewport()->stackUnder(tableViewFrozen);

    tableViewFrozen->setStyleSheet("QTableView { "
        "border: none;"
        "selection-background-color: #999}"
    );
}

// Destructor.
QTableViewFrozen::~QTableViewFrozen() {
    // tableViewFrozen is a child of this, Qt handles its deletion automatically.
}

// Override of setModel to assign the same model to both views.
void QTableViewFrozen::setModel(QAbstractItemModel* model) {
    tableViewFrozen->setModel(model);
    QTableView::setModel(model);

    QSortFilterProxyModel* filterModelFrozen = dynamic_cast<QSortFilterProxyModel*>(this->model());
    if (filterModelFrozen && filterModelFrozen->sourceModel()) {
        for(int col = 0; col < filterModelFrozen->sourceModel()->columnCount(); col++) {
            tableViewFrozen->setColumnHidden(col, true);
        }
    }

    updateFrozenTableGeometry();
}

// Function to change the width columns at the same time we resize them in the main QTableView.
void QTableViewFrozen::updateSectionWidth(int logicalIndex, int /* oldSize */, int newSize) {
    tableViewFrozen->horizontalHeader()->blockSignals(true);
    tableViewFrozen->setColumnWidth(logicalIndex, newSize);
    tableViewFrozen->horizontalHeader()->blockSignals(false);
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

void QTableViewFrozen::setUpdatesEnabled(bool enable) {
    QTableView::setUpdatesEnabled(enable);
    tableViewFrozen->setUpdatesEnabled(enable);
}

// Function to keep the cursor always visible, so it never gets hidden under the Frozen Columns.
QModelIndex QTableViewFrozen::moveCursor(
    CursorAction cursorAction,
    Qt::KeyboardModifiers modifiers
) {
    QModelIndex current = QTableView::moveCursor(cursorAction, modifiers);

    QSortFilterProxyModel* model = dynamic_cast<QSortFilterProxyModel*>(this->model());
    if (!model || !model->sourceModel()) {
        return current;
    }

    // We need to get this done dinamically, depending on the amount and size of frozen columns.
    int frozen_columns = 0;
    int frozen_width = 0;
    for (int i = 0; i < model->sourceModel()->columnCount(); ++i) {
        if (!tableViewFrozen->isColumnHidden(i)) {
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
void QTableViewFrozen::scrollTo(const QModelIndex & index, ScrollHint hint) {
    if (index.column() >= frozenColumns.count()) {
        QTableView::scrollTo(index, hint);
    }
}

// Function to update the geometry of the frozen QTableView when needed, to keep it at the right size.
void QTableViewFrozen::updateFrozenTableGeometry() {

    QSortFilterProxyModel* frozenTableModel = dynamic_cast<QSortFilterProxyModel*>(tableViewFrozen->model());
    if (!frozenTableModel || !frozenTableModel->sourceModel()) {
        return;
    }

    // Calculate the total width of all frozen columns.
    int frozenWidth = 0;
    for (int i = 0; i < frozenTableModel->sourceModel()->columnCount(); ++i) {
        if (frozenColumns.contains(i)) {
            frozenWidth += columnWidth(i);
        }
    }

    // Always get the vertical header width, even when hidden (for frozen mode).
    int vHeaderWidth = verticalHeader()->width();
    if (vHeaderWidth <= 0) {
        vHeaderWidth = verticalHeader()->sizeHint().width();
    }
    int fw = frameWidth();

    if (frozenColumns.isEmpty()) {
        verticalHeader()->show();
        tableViewFrozen->verticalHeader()->hide();
        tableViewFrozen->hide();

        QMargins margins = viewportMargins();
        margins.setLeft(vHeaderWidth + fw);
        setViewportMargins(margins);
    } else {
        // Hide the main view's vertical header and show the frozen view's own instead,
        // to prevent the main header from rendering behind the frozen overlay.
        verticalHeader()->hide();
        tableViewFrozen->verticalHeader()->show();
        tableViewFrozen->verticalHeader()->setDefaultSectionSize(verticalHeader()->defaultSectionSize());

        QMargins margins = viewportMargins();
        margins.setLeft(fw + vHeaderWidth + frozenWidth);
        setViewportMargins(margins);

        // The frozen view covers the frozen columns area on the left.
        tableViewFrozen->setGeometry(
            fw,
            fw,
            vHeaderWidth + frozenWidth,
            viewport()->height() + horizontalHeader()->height()
        );
        tableViewFrozen->raise();
        tableViewFrozen->show();

        // Ensure the frozen view never scrolls horizontally — it must stay pinned at 0.
        tableViewFrozen->horizontalScrollBar()->setValue(0);
    }
}

void QTableViewFrozen::toggleFreezer(int column) {

    if (frozenColumns.contains(column)) {
        frozenColumns.removeOne(column);
    }
    else {
        frozenColumns.append(column);
    }

    // Ensure every column in the frozen view matches the expected state. We can't rely
    // on setModel's hide loop because the source model has 0 columns at that point
    // (data is loaded afterwards), so columns added later are never hidden.
    if (model()) {
        QHeaderView* frozenHeader = tableViewFrozen->horizontalHeader();
        frozenHeader->blockSignals(true);
        for (int i = 0; i < model()->columnCount(); ++i) {
            bool shouldBeVisible = frozenColumns.contains(i);
            tableViewFrozen->setColumnHidden(i, !shouldBeVisible);
            if (shouldBeVisible) {
                tableViewFrozen->setColumnWidth(i, columnWidth(i));
                frozenHeader->setSectionResizeMode(i, QHeaderView::Interactive);
            }
        }
        frozenHeader->blockSignals(false);

        // Sync row heights from the main view.
        for (int row = 0; row < model()->rowCount(); ++row) {
            tableViewFrozen->setRowHeight(row, rowHeight(row));
        }
    }

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

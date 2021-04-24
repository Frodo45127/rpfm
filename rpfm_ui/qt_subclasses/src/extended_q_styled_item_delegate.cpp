#include "extended_q_styled_item_delegate.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QSortFilterProxyModel>
#include <QPen>
#include <QColor>
#include <QPainter>
#include <QStandardItem>
#include <QStyle>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QExtendedStyledItemDelegate.
extern "C" void new_generic_item_delegate(QObject *parent, const int column, QTimer* timer, bool is_dark_theme_enabled, bool has_filter) {
    QExtendedStyledItemDelegate* delegate = new QExtendedStyledItemDelegate(parent, timer, is_dark_theme_enabled, has_filter);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of QExtendedStyledItemDelegate. We use it to store the integer type of the value in the delegate.
QExtendedStyledItemDelegate::QExtendedStyledItemDelegate(QObject *parent, QTimer* timer, bool is_dark_theme_enabled, bool has_filter): QStyledItemDelegate(parent)
{
    diag_timer = timer;
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;
}

// Function called when the editor for the cell it's created.
QWidget* QExtendedStyledItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &option, const QModelIndex &index) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    return QStyledItemDelegate::createEditor(parent, option, index);
}

// Function for the delegate to showup properly.
void QExtendedStyledItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    QStyledItemDelegate::paint( painter, option, index );

    if (use_filter && index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());
        QStandardItem* item = standardModel->itemFromIndex(filterModel->mapToSource(index));

        if (item != nullptr) {
            QVariant isKeyVariant = item->data(20);
            QVariant isAddedVariant = item->data(21);
            QVariant isModifiedVariant = item->data(22);
            QVariant isErrorVariant = item->data(25);
            QVariant isWarningVariant = item->data(26);
            QVariant isInfoVariant = item->data(27);

            bool isKey = !isKeyVariant.isNull() ? isKeyVariant.toBool(): false;
            bool isAdded = !isAddedVariant.isNull() ? isAddedVariant.toBool(): false;
            bool isModified = !isModifiedVariant.isNull() ? isModifiedVariant.toBool(): false;

            bool isError = !isErrorVariant.isNull() ? isErrorVariant.toBool(): false;
            bool isWarning = !isWarningVariant.isNull() ? isWarningVariant.toBool(): false;
            bool isInfo = !isInfoVariant.isNull() ? isInfoVariant.toBool(): false;

            // Fun fact about the painter. It's the same it was used in the cell before,
            // with the same config as the cell before.
            //
            // This means if the cell before was a key, this one will have the key background.
            // This and the restore at the end fixes it.
            painter->save();

            // Paint the background of keys, to identify them.
            if (isKey) {
                QColor colorBrush;

                if (dark_theme) {
                    colorBrush.setRgbF(82, 82, 0, 0.1);
                } else {
                    colorBrush.setRgbF(255, 255, 0, 0.1);
                }

                QBrush qBrush(colorBrush);
                qBrush.setStyle(Qt::BrushStyle::SolidPattern);

                auto pen = QPen();
                pen.setWidth(0);
                pen.setColor(colorBrush);

                painter->setBrush(qBrush);
                painter->setPen(pen);
                painter->drawRect(option.rect);
            }

            // Modified takes priority over added.
            if (isModified) {
                auto pen = QPen();

                if (dark_theme) {
                    QColor colorPen = Qt::GlobalColor::yellow;
                    pen.setColor(colorPen);
                } else {
                    QColor colorPen;
                    colorPen.setRgb(230, 126, 34);
                    pen.setColor(colorPen);
                }

                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(2);

                painter->setPen(pen);
                painter->drawRect(option.rect.x() + 1, option.rect.y() + 1, option.rect.width() - 2, option.rect.height() - 2);
            }

            else if (!isModified && isAdded) {
                auto pen = QPen();

                if (dark_theme) {
                    QColor colorPen = Qt::GlobalColor::green;
                    pen.setColor(colorPen);
                } else {
                    QColor colorPen = Qt::GlobalColor::green;
                    pen.setColor(colorPen);
                }

                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(2);

                painter->setPen(pen);
                painter->drawRect(option.rect.x() + 1, option.rect.y() + 1, option.rect.width() - 2, option.rect.height() - 2);
            }

            // By priority, info goes first.
            if (isInfo) {
                auto pen = QPen();

                if (dark_theme) {
                    QColor colorPen = Qt::GlobalColor::blue;
                    pen.setColor(colorPen);
                } else {
                    QColor colorPen = Qt::GlobalColor::blue;
                    pen.setColor(colorPen);
                }

                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(4);

                painter->setPen(pen);
                painter->drawRect(option.rect.x() + 1, option.rect.y() + 1, option.rect.width() - 2, option.rect.height() - 2);
            }

            // Warning goes second, overwriting info.
            if (isWarning) {
                auto pen = QPen();

                if (dark_theme) {
                    QColor colorPen = Qt::GlobalColor::yellow;
                    pen.setColor(colorPen);
                } else {
                    QColor colorPen;
                    colorPen.setRgb(190, 190, 0);
                    pen.setColor(colorPen);
                }

                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(4);

                painter->setPen(pen);
                painter->drawRect(option.rect.x() + 1, option.rect.y() + 1, option.rect.width() - 2, option.rect.height() - 2);
            }

            // Error goes last, overwriting everything.
            if (isError) {
                auto pen = QPen();

                if (dark_theme) {
                    QColor colorPen = Qt::GlobalColor::red;
                    pen.setColor(colorPen);
                } else {
                    QColor colorPen = Qt::GlobalColor::red;
                    pen.setColor(colorPen);
                }

                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(4);

                painter->setPen(pen);
                painter->drawRect(option.rect.x() + 1, option.rect.y() + 1, option.rect.width() - 2, option.rect.height() - 2);
            }

            // Remember to restore the painter so we can reuse it for other cells.
            painter->restore();
        }
    }
}

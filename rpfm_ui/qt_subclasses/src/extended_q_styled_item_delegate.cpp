#include "extended_q_styled_item_delegate.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QSortFilterProxyModel>
#include <QPen>
#include <QColor>
#include <QPainter>
#include <QStandardItem>
#include <QStyle>
#include <QSettings>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QExtendedStyledItemDelegate.
extern "C" void new_generic_item_delegate(QObject *parent, const int column, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark) {
    QExtendedStyledItemDelegate* delegate = new QExtendedStyledItemDelegate(parent, timer, is_dark_theme_enabled, has_filter, right_side_mark);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of QExtendedStyledItemDelegate. We use it to store the integer type of the value in the delegate.
QExtendedStyledItemDelegate::QExtendedStyledItemDelegate(QObject *parent, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark): QStyledItemDelegate(parent)
{
    diag_timer = timer;
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;
    use_right_side_mark = right_side_mark;

    QSettings* q_settings = new QSettings("FrodoWazEre", "rpfm");

    if (dark_theme) {
        colour_table_added = QColor(q_settings->value("colour_dark_table_added").toString());
        colour_table_modified = QColor(q_settings->value("colour_dark_table_modified").toString());
        colour_diagnostic_error = QColor(q_settings->value("colour_dark_diagnostic_error").toString());
        colour_diagnostic_warning = QColor(q_settings->value("colour_dark_diagnostic_warning").toString());
        colour_diagnostic_info = QColor(q_settings->value("colour_dark_diagnostic_info").toString());
    } else {
        colour_table_added = QColor(q_settings->value("colour_light_table_added").toString());
        colour_table_modified = QColor(q_settings->value("colour_light_table_modified").toString());
        colour_diagnostic_error = QColor(q_settings->value("colour_light_diagnostic_error").toString());
        colour_diagnostic_warning = QColor(q_settings->value("colour_light_diagnostic_warning").toString());
        colour_diagnostic_info = QColor(q_settings->value("colour_light_diagnostic_info").toString());
    }
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
                pen.setColor(colour_table_modified);

                int lineWidth = 2;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                if (use_right_side_mark) {
                    painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
                } else {
                    painter->drawLine(QLineF(option.rect.x() + 1, option.rect.y() + (lineWidth / 2), option.rect.x() + 1, option.rect.y() + option.rect.height() - (lineWidth / 4)));
                }
            }

            else if (!isModified && isAdded) {
                auto pen = QPen();
                pen.setColor(colour_table_added);

                int lineWidth = 2;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                if (use_right_side_mark) {
                    painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
                } else {
                    painter->drawLine(QLineF(option.rect.x() + 1, option.rect.y() + (lineWidth / 2), option.rect.x() + 1, option.rect.y() + option.rect.height() - (lineWidth / 4)));
                }
            }

            // By priority, info goes first.
            if (isInfo) {
                auto pen = QPen();
                pen.setColor(colour_diagnostic_info);

                int lineWidth = 4;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                if (use_right_side_mark) {
                    painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
                } else {
                    painter->drawLine(QLineF(option.rect.x() + 1, option.rect.y() + (lineWidth / 2), option.rect.x() + 1, option.rect.y() + option.rect.height() - (lineWidth / 4)));
                }
            }

            // Warning goes second, overwriting info.
            if (isWarning) {
                auto pen = QPen();
                pen.setColor(colour_diagnostic_warning);

                int lineWidth = 4;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                if (use_right_side_mark) {
                    painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
                } else {
                    painter->drawLine(QLineF(option.rect.x() + 1, option.rect.y() + (lineWidth / 2), option.rect.x() + 1, option.rect.y() + option.rect.height() - (lineWidth / 4)));
                }
            }

            // Error goes last, overwriting everything.
            if (isError) {
                auto pen = QPen();
                pen.setColor(colour_diagnostic_error);

                int lineWidth = 4;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                if (use_right_side_mark) {
                    painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
                } else {
                    painter->drawLine(QLineF(option.rect.x() + 1, option.rect.y() + (lineWidth / 2), option.rect.x() + 1, option.rect.y() + option.rect.height() - (lineWidth / 4)));
                }
            }

            // Remember to restore the painter so we can reuse it for other cells.
            painter->restore();
        }
    }
}

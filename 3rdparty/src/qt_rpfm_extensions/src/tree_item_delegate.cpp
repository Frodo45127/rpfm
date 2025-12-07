#include "tree_item_delegate.h"
#include <QAbstractItemView>
#include <QColor>
#include <QDebug>
#include <QDoubleSpinBox>
#include <QPainter>
#include <QPen>
#include <QSettings>
#include <QSortFilterProxyModel>
#include <QStandardItem>
#include <QStyle>


// Function to be called from any other language. This assing to the provided column of the provided TableView a QTreeItemDelegate.
extern "C" void new_tree_item_delegate(QObject *parent, bool is_dark_theme_enabled, bool has_filter) {
    QTreeItemDelegate* delegate = new QTreeItemDelegate(parent, is_dark_theme_enabled, has_filter);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(0, delegate);
}

// Constructor of the QTreeItemDelegate.
QTreeItemDelegate::QTreeItemDelegate(QObject *parent, bool is_dark_theme_enabled, bool has_filter): QExtendedStyledItemDelegate(parent, nullptr, is_dark_theme_enabled, has_filter, true) {
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;
    use_right_side_mark = true;

    QSettings* q_settings = new QSettings("FrodoWazEre", "rpfm");

    if (dark_theme) {
        colour_tree_added = QColor(q_settings->value("colour_dark_table_added").toString());
        colour_tree_modified = QColor(q_settings->value("colour_dark_table_modified").toString());

    } else {
        colour_tree_added = QColor(q_settings->value("colour_light_table_added").toString());
        colour_tree_modified = QColor(q_settings->value("colour_light_table_modified").toString());
    }
}


// Function for the delegate to showup properly.
void QTreeItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    QStyledItemDelegate::paint( painter, option, index );

    if (use_filter && index.isValid()) {
        const QSortFilterProxyModel* filterModel = dynamic_cast<const QSortFilterProxyModel*>(index.model());
        const QStandardItemModel* standardModel = dynamic_cast<const QStandardItemModel*>(filterModel->sourceModel());
        QStandardItem* item = standardModel->itemFromIndex(filterModel->mapToSource(index));

        if (item != nullptr) {
            QVariant statusVariant = item->data(21);
            QVariant isForeverModifiedVariant = item->data(22);

            int status = !statusVariant.isNull() ? statusVariant.toInt(): 0;
            bool isForeverModified = !isForeverModifiedVariant.isNull() ? isForeverModifiedVariant.toBool(): false;

            // Fun fact about the painter. It's the same it was used in the cell before,
            // with the same config as the cell before.
            //
            // This means if the cell before was a key, this one will have the key background.
            // This and the restore at the end fixes it.
            painter->save();

            // Modified takes priority over added.
            if (status == 2 || isForeverModified) {
                auto pen = QPen();
                pen.setColor(colour_tree_modified);

                int lineWidth = 2;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
            }

            else if (status == 1) {
                auto pen = QPen();
                pen.setColor(colour_tree_added);

                int lineWidth = 2;
                pen.setStyle(Qt::PenStyle::SolidLine);
                pen.setWidth(lineWidth);

                painter->setPen(pen);
                painter->drawLine(QLineF(option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + (lineWidth / 2), option.rect.x() + option.rect.width() - (lineWidth / 2), option.rect.y() + option.rect.height() - (lineWidth / 4)));
            }

            // Remember to restore the painter so we can reuse it for other cells.
            painter->restore();
        }
    }
}

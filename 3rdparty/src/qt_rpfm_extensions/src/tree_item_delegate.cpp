#include "tree_item_delegate.h"
#include <QAbstractItemView>
#include <QColor>
#include <QDebug>
#include <QDoubleSpinBox>
#include <QGuiApplication>
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
    use_filter = has_filter;
    use_right_side_mark = true;
}


// Function for the delegate to showup properly.
void QTreeItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    refreshThemeColorsIfNeeded();

    // Refresh tree-specific colors when the palette changes.
    qint64 current_key = QGuiApplication::palette().cacheKey();
    if (current_key != cached_tree_palette_key) {
        cached_tree_palette_key = current_key;

        QSettings q_settings("FrodoWazEre", "rpfm");
        if (dark_theme) {
            colour_tree_added = QColor(q_settings.value("colour_dark_table_added").toString());
            colour_tree_modified = QColor(q_settings.value("colour_dark_table_modified").toString());
        } else {
            colour_tree_added = QColor(q_settings.value("colour_light_table_added").toString());
            colour_tree_modified = QColor(q_settings.value("colour_light_table_modified").toString());
        }
    }

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

            // Draw "MyMod" label in italic on the right side for MyMod pack root nodes.
            // Role 23 = ROOT_NODE_TYPE, value 5 = ROOT_NODE_TYPE_MYMOD_PACKFILE.
            QVariant rootTypeVariant = item->data(23);
            if (!rootTypeVariant.isNull() && rootTypeVariant.toInt() == 5) {
                QFont italicFont = option.font;
                italicFont.setItalic(true);
                italicFont.setPointSizeF(italicFont.pointSizeF() * 0.85);
                painter->setFont(italicFont);

                QColor labelColor = option.palette.color(QPalette::Disabled, QPalette::Text);
                painter->setPen(labelColor);

                int rightMargin = 8;
                QRect textRect = option.rect.adjusted(0, 0, -rightMargin, 0);
                painter->drawText(textRect, Qt::AlignRight | Qt::AlignVCenter, "MyMod");
            }

            // Remember to restore the painter so we can reuse it for other cells.
            painter->restore();
        }
    }
}

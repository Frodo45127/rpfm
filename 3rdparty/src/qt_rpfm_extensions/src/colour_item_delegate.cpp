#include "colour_item_delegate.h"
#include "float.h"
#include <QDebug>
#include <QAbstractItemView>
#include <QColorDialog>
#include <QPainter>
#include <QSettings>
#include <QApplication>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QColourItemDelegate.
extern "C" void new_colour_item_delegate(QObject *parent, const int column, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark, bool enable_diff_markers) {
    QColourPickerItemDelegate* delegate = new QColourPickerItemDelegate(parent, timer, is_dark_theme_enabled, has_filter, right_side_mark, enable_diff_markers);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of the QDoubleSpinBoxItemDelegate. Empty, as we don't need to do anything special with it.
QColourPickerItemDelegate::QColourPickerItemDelegate(QObject *parent, QTimer* timer, bool is_dark_theme_enabled, bool has_filter, bool right_side_mark, bool enable_diff_markers):
    QExtendedStyledItemDelegate(parent, timer, is_dark_theme_enabled, has_filter, right_side_mark, enable_diff_markers) {
    skipTextPainting = true;
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

// Function called when the spinbox it's created. Here we configure the limits and decimals of the spinbox.
QWidget* QColourPickerItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    QColorDialog* dialog = new QColorDialog(parent);
    return dialog;
}

// Function called after the spinbox it's created. It just gives it his initial value (the one currently in the model).
void QColourPickerItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QColorDialog* dialog = static_cast<QColorDialog*>(editor);
    QString colorName = index.model()->data(index, Qt::EditRole).toString();
    colorName.insert(0, '#');
    QColor* value = new QColor(colorName);
    dialog->setCurrentColor(*value);
}

// Function to be called when we're done. It just takes the value in the spinbox and saves it in the Table Model.
void QColourPickerItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QColorDialog* dialog = static_cast<QColorDialog*>(editor);
    QColor color = dialog->currentColor();
    QString colorName = color.name(QColor::HexRgb);

    // Remove #, so parsing as radix doesn't fail.
    colorName.remove(0, 1);
    model->setData(index, colorName.toUpper(), Qt::EditRole);
}

// Function for the delegate to showup properly.
void QColourPickerItemDelegate::paint(QPainter *painter, const QStyleOptionViewItem &option, const QModelIndex &index) const {

    // Manually initialize part of the QStyledItemDelegate painter code.
    QStyleOptionViewItem opt = option;
    //initStyleOption(&opt, index);

    const QWidget *widget = option.widget;
    QStyle *style = widget ? widget->style() : QApplication::style();
    style->drawControl(QStyle::CE_ItemViewItem, &opt, painter, widget);

    QExtendedStyledItemDelegate::paint( painter, option, index);
    if (index.isValid()) {

        // Fun fact about the painter. It's the same it was used in the cell before,
        // with the same config as the cell before.
        //
        // This means if the cell before was a key, this one will have the key background.
        // This and the restore at the end fixes it.
        painter->save();

        // Paint a small square on the left with the colour of the cell.
        QString colorName = index.model()->data(index, Qt::EditRole).toString();
        colorName.insert(0, '#');

        QColor* color = new QColor(colorName);

        QBrush qBrush(*color);
        qBrush.setStyle(Qt::BrushStyle::SolidPattern);

        auto pen = QPen();
        pen.setWidth(1);
        pen.setColor(*color);

        painter->setBrush(qBrush);
        painter->setPen(pen);

        int squareHeight = (float)option.rect.height() / (float)100 * 60;
        int squareMargin = (option.rect.height() - squareHeight) / 2;
        int posX = option.rect.x() + squareMargin;
        int posY = option.rect.y() + squareMargin;

        painter->drawRect(posX, posY, squareHeight, squareHeight);

        // Remember to restore the painter so we can reuse it for other cells.
        painter->restore();

        // Repaint the text to move it to the right.
        painter->save();
        QRect* textRect = new QRect(option.rect.x() + (squareMargin * 2) + squareHeight, option.rect.y(), option.rect.width() - squareHeight - (squareMargin * 2), option.rect.height());
        painter->drawText(*textRect, option.displayAlignment, index.data().toString());
        painter->restore();
    }
}

void QColourPickerItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &index) const {
    editor->setGeometry(option.rect);
    editor->move(QCursor::pos());
}

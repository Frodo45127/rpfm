#include "qstring_item_delegate.h"
#include <QAbstractItemView>
#include <QLineEdit>

// Function to be called from any other language. This assing to the provided column of the provided TableView a QStringItemDelegate.
extern "C" void new_qstring_item_delegate(QObject *parent, const int column, const int max_lenght, QTimer* timer, bool is_dark_theme_enabled, bool has_filter) {
    QStringItemDelegate* delegate = new QStringItemDelegate(parent, max_lenght, timer, is_dark_theme_enabled, has_filter);
    dynamic_cast<QAbstractItemView*>(parent)->setItemDelegateForColumn(column, delegate);
}

// Constructor of the QStringItemDelegate. We use it to store the max lenght allowed for the delegate.
QStringItemDelegate::QStringItemDelegate(QObject *parent, const int lenght, QTimer* timer, bool is_dark_theme_enabled, bool has_filter): QExtendedStyledItemDelegate(parent) {
    max_lenght = lenght;
    diag_timer = timer;
    dark_theme = is_dark_theme_enabled;
    use_filter = has_filter;
}

// Function called when the widget it's created. Here we configure the QLinEdit.
QWidget* QStringItemDelegate::createEditor(QWidget *parent, const QStyleOptionViewItem &, const QModelIndex &) const {

    // Stop the diagnostics timer, so it doesn't steal the focus of the editor.
    if (diag_timer) {
        diag_timer->stop();
    }

    QLineEdit *editor = new QLineEdit(parent);
    //if (this->max_lenght > 0) {
    //    editor->setMaxLength(max_lenght);
    //}
    return editor;
}

// Function called after the QLinEdit it's created. It just gives it his initial value (the one currently in the model).
void QStringItemDelegate::setEditorData(QWidget *editor, const QModelIndex &index) const {
    QLineEdit *line = static_cast<QLineEdit*>(editor);
    QString value = index.model()->data(index, Qt::EditRole).toString();
    line->setText(value);
}

// Function to be called when we're done. It just takes the value in the QLineEdit and saves it in the Table Model.
void QStringItemDelegate::setModelData(QWidget *editor, QAbstractItemModel *model, const QModelIndex &index) const {
    QLineEdit *line = static_cast<QLineEdit*>(editor);
    QString value = line->text();
    model->setData(index, value);
}

// Function for the QLineEdit to show up properly.
void QStringItemDelegate::updateEditorGeometry(QWidget *editor, const QStyleOptionViewItem &option, const QModelIndex &) const {
    editor->setGeometry(option.rect);
}

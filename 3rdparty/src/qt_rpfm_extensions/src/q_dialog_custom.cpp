#include "q_dialog_custom.h"
#include <QApplication>
#include <QDebug>
#include <QFileInfo>
#include <QIcon>
#include <QResource>
#include <QMimeData>

// Fuction to be able to create a custom QMainWindow.
extern "C" QDialog* new_q_dialog_custom(QWidget *parent, bool (*are_you_sure) (QDialog* dialog)) {
    return dynamic_cast<QDialog*>(new QDialogCustom(parent, are_you_sure));
}

QDialogCustom::QDialogCustom(QWidget *parent, bool (*are_you_sure_fn) (QDialog* dialog)) : QDialog(parent) {
    are_you_sure = are_you_sure_fn;
}

// Overload of the close event so we can put a dialog there.
void QDialogCustom::closeEvent(QCloseEvent *event) {
    event->ignore();

    if (are_you_sure(this)) {
        event->accept();
    }
}

#include "q_main_window_custom.h"

// Fuction to be able to create a custom QMainWindow.
extern "C" QMainWindow* new_q_main_window_custom(bool (*are_you_sure) (QMainWindow* main_window, bool is_delete_my_mod)) {
    return dynamic_cast<QMainWindow*>(new QMainWindowCustom(nullptr, are_you_sure));
}

QMainWindowCustom::QMainWindowCustom(QWidget *parent, bool (*are_you_sure_fn) (QMainWindow* main_window, bool is_delete_my_mod)) : QMainWindow(parent) {
    are_you_sure = are_you_sure_fn;
}

// Overload of the close event so we can put a dialog there.
void QMainWindowCustom::closeEvent(QCloseEvent *event) {
    event->ignore();

    // Save the state of the window before closing it.
    QSettings settings("FrodoWazEre", "rpfm");
    settings.setValue("geometry", saveGeometry());
    settings.setValue("windowState", saveState());

    // Make sure the settings are saved before closing.
    settings.sync();

    if (are_you_sure(this, false)) {
        event->accept();
    }
}

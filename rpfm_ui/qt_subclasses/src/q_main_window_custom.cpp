#include "q_main_window_custom.h"
#include <QApplication>
#include <QIcon>

// Fuction to be able to create a custom QMainWindow.
extern "C" QMainWindow* new_q_main_window_custom(bool (*are_you_sure) (QMainWindow* main_window, bool is_delete_my_mod)) {
    return dynamic_cast<QMainWindow*>(new QMainWindowCustom(nullptr, are_you_sure));
}

QMainWindowCustom::QMainWindowCustom(QWidget *parent, bool (*are_you_sure_fn) (QMainWindow* main_window, bool is_delete_my_mod)) : QMainWindow(parent) {
    are_you_sure = are_you_sure_fn;
    busyIndicator = new KBusyIndicatorWidget(this);

    // Initialize the global icon loader. We don't use it, just initialize it.
    QStringList iconThemes;
    iconThemes << QApplication::applicationDirPath()+"/icons";
    QIcon::setThemeSearchPaths(iconThemes);
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

void QMainWindowCustom::moveEvent(QMoveEvent* event) {
    Q_UNUSED(event);

    const QPoint center = rect().center();
    busyIndicator->move(center.x() - busyIndicator->width() / 2, center.y() - busyIndicator->height() / 2);
}

void QMainWindowCustom::changeEvent(QEvent* event) {
    bool enabled = isEnabled();
    if (event->type() == QEvent::EnabledChange) {
        if (enabled) {
            busyIndicator->hide();
        } else {
            busyIndicator->show();
        }
    }
}

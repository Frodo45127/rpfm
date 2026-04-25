#include "kicontheme.h"
#include <breezeicons.h>

#include "q_main_window_custom.h"
#include <QApplication>
#include <QDebug>
#include <QFileInfo>
#include <QIcon>
#include <QMimeData>
#include <QResource>
#include <QStatusBar>

// Must be called before QApplication is created. Sets up KIconTheme so that the
// KIconEnginePlugin is discovered and icons are palette-recolored on dark themes.
extern "C" void init_icon_theme() {
    KIconTheme::initTheme();
}

// Fuction to be able to create a custom QMainWindow.
extern "C" QMainWindow* new_q_main_window_custom(bool (*are_you_sure) (QMainWindow* main_window, bool is_delete_my_mod, bool is_full_close), bool is_dark_theme_enabled) {
    return dynamic_cast<QMainWindow*>(new QMainWindowCustom(nullptr, are_you_sure, is_dark_theme_enabled));
}

QMainWindowCustom::QMainWindowCustom(QWidget *parent, bool (*are_you_sure_fn) (QMainWindow* main_window, bool is_delete_my_mod, bool is_full_close), bool is_dark_theme_enabled) : QMainWindow(parent) {
    are_you_sure = are_you_sure_fn;
    dark_theme_enabled = is_dark_theme_enabled;

    busyIndicator = new KBusyIndicatorWidget();
    busyIndicator->setFixedSize(16, 16);
    statusBar()->addPermanentWidget(busyIndicator);
    busyIndicator->hide();

    setAcceptDrops(true);

    #ifdef _WIN32

        // Initialize the Breeze icon theme from the KF6BreezeIcons library.
        // This registers both breeze and breeze-dark themes from the compiled-in
        // resources, and Qt's icon engine handles dark/light switching automatically
        // based on the current palette.
        BreezeIcons::initIcons();
        QIcon::setThemeName(QStringLiteral("breeze"));
    #endif
}

// Overload of the close event so we can put a dialog there.
void QMainWindowCustom::closeEvent(QCloseEvent *event) {
    event->ignore();

    if (are_you_sure(this, false, true)) {
        event->accept();
    }
}

void QMainWindowCustom::changeEvent(QEvent* event) {
    if (event->type() == QEvent::EnabledChange) {
        if (isEnabled()) {
            busyIndicator->hide();
        } else {
            busyIndicator->show();
        }
    }

    // Notify Rust side so it can update theme-dependent widgets
    if (event->type() == QEvent::PaletteChange) {
        emit themeChanged();
    }

    QMainWindow::changeEvent(event);
}

void QMainWindowCustom::dragEnterEvent(QDragEnterEvent *event) {
    event->accept();
}

void QMainWindowCustom::dragMoveEvent(QDragMoveEvent *event) {
    QMainWindow::dragMoveEvent(event);
}

void QMainWindowCustom::dragLeaveEvent(QDragLeaveEvent *event) {
    QMainWindow::dragLeaveEvent(event);
}

void QMainWindowCustom::dropEvent(QDropEvent *event) {

    const QMimeData* mimeData = event->mimeData();
    if (mimeData->hasUrls()) {
        QStringList pathList;
        QList<QUrl> urlList = mimeData->urls();

        for (int i = 0; i < urlList.size(); ++i) {
            pathList.append(urlList.at(i).toLocalFile());
        }

        emit openPack(pathList);
    }
}

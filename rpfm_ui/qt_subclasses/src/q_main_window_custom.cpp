#include "kicontheme.h"

#include "q_main_window_custom.h"
#include <QApplication>
#include <QDebug>
#include <QFileInfo>
#include <QIcon>
#include <QResource>

// Fuction to be able to create a custom QMainWindow.
extern "C" QMainWindow* new_q_main_window_custom(bool (*are_you_sure) (QMainWindow* main_window, bool is_delete_my_mod), bool is_dark_theme_enabled) {
    return dynamic_cast<QMainWindow*>(new QMainWindowCustom(nullptr, are_you_sure, is_dark_theme_enabled));
}

QMainWindowCustom::QMainWindowCustom(QWidget *parent, bool (*are_you_sure_fn) (QMainWindow* main_window, bool is_delete_my_mod), bool is_dark_theme_enabled) : QMainWindow(parent) {
    are_you_sure = are_you_sure_fn;
    dark_theme_enabled = is_dark_theme_enabled;
    busyIndicator = new KBusyIndicatorWidget(this);

    // Initialize the icon theme. Holy shit this took way too much research to find how it works.
    const QString iconThemeName = QStringLiteral("breeze");

    const QString iconThemeRccFallback = qApp->applicationDirPath() + QStringLiteral("/data/icons/breeze/breeze-icons.rcc");
    const QString iconThemeRccDark = qApp->applicationDirPath() + QStringLiteral("/data/icons/breeze-dark/breeze-icons-dark.rcc");

    qWarning() << "Rcc file for Dark theme" << iconThemeRccDark;
    qWarning() << "Rcc file for Light theme" << iconThemeRccFallback;

    if (!iconThemeRccDark.isEmpty() && !iconThemeRccFallback.isEmpty()) {
        const QString iconSubdir = QStringLiteral("/icons/") + iconThemeName;
        bool load_fallback = QResource::registerResource(iconThemeRccFallback, iconSubdir);

        // Only load the dark theme resources if needed.
        bool load_dark = false;
        if (dark_theme_enabled) {
            load_dark = QResource::registerResource(iconThemeRccDark, iconSubdir);
        }

        // If nothing failed, set the themes.
        if (load_fallback && (load_dark || !dark_theme_enabled)) {
            if (QFileInfo::exists(QLatin1Char(':') + iconSubdir + QStringLiteral("/index.theme"))) {
                QIcon::setThemeName(iconThemeName);
                QIcon::setFallbackThemeName(QStringLiteral("breeze"));
            } else {
                qWarning() << "No index.theme found in" << iconThemeRccDark;
                qWarning() << "No index.theme found in" << iconThemeRccFallback;
                QResource::unregisterResource(iconThemeRccDark, iconSubdir);
                QResource::unregisterResource(iconThemeRccFallback, iconSubdir);
            }
        } else {
            qWarning() << "Invalid rcc file" << iconThemeRccFallback;
        }
    } else {
        qWarning() << "Empty rcc file" << iconThemeRccDark;
        qWarning() << "Empty rcc file" << iconThemeRccFallback;
    }
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

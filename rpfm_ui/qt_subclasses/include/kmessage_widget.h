#ifndef KMESSAGE_WIDGET_H
#define KMESSAGE_WIDGET_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF5/KWidgetsAddons/KMessageWidget>
#else
#include <KMessageWidget>
#endif
#include <QWidget>
#include <QString>

extern "C" void kmessage_widget_close(QWidget* widget = nullptr);
extern "C" void kmessage_widget_set_error(QWidget* widget = nullptr, QString const text = "");
extern "C" void kmessage_widget_set_warning(QWidget* widget = nullptr, QString const text = "");
extern "C" void kmessage_widget_set_info(QWidget* widget = nullptr, QString const text = "");
#endif // KMESSAGE_WIDGET_H

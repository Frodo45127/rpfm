#ifndef KLINE_EDIT_CUSTOM_H
#define KLINE_EDIT_CUSTOM_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF5/KCompletion/KLineEdit>
#else
#include <KLineEdit>
#endif
#include <QWidget>
#include <QColor>

extern "C" void kline_edit_configure(QWidget* view = nullptr);

#endif // KLINE_EDIT_CUSTOM_H

#ifndef KCOLOR_COMBO_H
#define KCOLOR_COMBO_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF5/KWidgetsAddons/KColorCombo>
#else
#include <KColorCombo>
#endif
#include <QWidget>
#include <QColor>

extern "C" int get_color(QWidget* view = nullptr);

extern "C" void set_color(QWidget* view = nullptr, QColor* color = nullptr);

#endif // KCOLOR_COMBO_H

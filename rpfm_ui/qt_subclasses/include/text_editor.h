#ifndef TEXT_EDITOR_H
#define TEXT_EDITOR_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF5/KTextEditor/ktexteditor/Document>
#include <KF5/KTextEditor/ktexteditor/Editor>
#include <KF5/KTextEditor/ktexteditor/View>
#else
#include <KTextEditor/Document>
#include <KTextEditor/Editor>
#include <KTextEditor/View>
#endif
#include <QWidget>

extern "C" QWidget* new_text_editor(QWidget* parent = nullptr);

extern "C" QString* get_text(QWidget* parent = nullptr);

extern "C" void set_text(QWidget* view = nullptr, QString* text = nullptr, QString* highlighting_mode = nullptr);

extern "C" void open_text_editor_config(QWidget* parent);

#endif // TEXT_EDITOR_H

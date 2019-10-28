#ifndef TEXT_EDITOR_H
#define TEXT_EDITOR_H

#include "qt_subclasses_global.h"
#include <KTextEditor/Document>
#include <KTextEditor/Editor>
#include <KTextEditor/View>

extern "C" QWidget* new_text_editor(QWidget* parent = nullptr);

extern "C" QString* get_text(QWidget* parent = nullptr);

extern "C" void set_text(QWidget* view = nullptr, QString* text = nullptr);

extern "C" void config(QWidget* parent);

#endif // TEXT_EDITOR_H

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
#include <QLineEdit>

// This one is needed for the save fix.
#include <KActionCollection>

extern "C" QWidget* new_text_editor(QWidget* parent = nullptr);

extern "C" QString* get_text(QWidget* parent = nullptr);

extern "C" void set_text(QWidget* view = nullptr, QString* text = nullptr, QString* highlighting_mode = nullptr);

extern "C" void open_text_editor_config(QWidget* parent);

extern "C" QLineEdit* get_text_changed_dummy_widget(QWidget* view = nullptr);

extern "C" void scroll_to_row(QWidget* view = nullptr, int row_number = 0);

#endif // TEXT_EDITOR_H

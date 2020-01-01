#include "text_editor.h"

// Function to create the filter in a way that we don't need to bother Rust with new types.
extern "C" QWidget* new_text_editor(QWidget* parent) {
    KTextEditor::Editor *editor = KTextEditor::Editor::instance();
    KTextEditor::Document *doc = editor->createDocument(parent);
    KTextEditor::View *view = doc->createView(parent);
    return dynamic_cast<QWidget*>(view);
}

// Function to return the current text of the Text Editor.
extern "C" QString* get_text(QWidget* view) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    KTextEditor::Document* doc = doc_view->document();
    QString text_object = doc->text();
    QString* text = new QString(text_object);

    return text;
}

// Function to set the current text of the text editor.
extern "C" void set_text(QWidget* view, QString* text, QString* highlighting_mode) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    KTextEditor::Document* doc = doc_view->document();
    QString text_object = *text;
    doc->setText(text_object);

    QString highlight_mode = *highlighting_mode;
    doc->setHighlightingMode(highlight_mode);
}

// Function to trigger the config dialog of the text editor.
extern "C" void open_text_editor_config(QWidget* parent) {

    KTextEditor::Editor* editor = KTextEditor::Editor::instance();
    editor->configDialog(parent);
}


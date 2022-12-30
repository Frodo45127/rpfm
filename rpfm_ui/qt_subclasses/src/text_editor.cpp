#include "text_editor.h"

// Function to create the filter in a way that we don't need to bother Rust with new types.
extern "C" QWidget* new_text_editor(QWidget* parent) {
    KTextEditor::Editor *editor = KTextEditor::Editor::instance();
    KTextEditor::Document *doc = editor->createDocument(parent);
    KTextEditor::View *view = doc->createView(parent);

    // Disable the status bar.
    view->setStatusBarEnabled(false);

    // Remove the save and saveAs actions, as we don't support saving to disk and interfere with RPFM.
    KActionCollection* actions = view->actionCollection();
    actions->removeAction(actions->action("file_save"));
    actions->removeAction(actions->action("file_save_as"));

    QLineEdit* dummy = new QLineEdit(view);
    dummy->setObjectName("Dummy");
    dummy->setVisible(false);

    // Return the view widget, so we can access it later.
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

    // This fixes the "modified" state due to setting the text for the first time.
    // IF you hit Ctrl+Z it still removes the text, but at least we can now keep track of when a file has been modified.
    doc->setModified(false);
    doc_view->setCursorPosition(KTextEditor::Cursor::start());

    QLineEdit* dummy = doc_view->findChild<QLineEdit*>("Dummy");
    QObject::connect(
        doc,
        &KTextEditor::Document::textChanged,
        dummy,
        [dummy] {
            emit dummy->textChanged(nullptr);
        }
    );

    QString highlight_mode = *highlighting_mode;
    doc->setHighlightingMode(highlight_mode);
}

// Function to trigger the config dialog of the text editor.
extern "C" void open_text_editor_config(QWidget* parent) {

    KTextEditor::Editor* editor = KTextEditor::Editor::instance();
    editor->configDialog(parent);
}

// Function to return the dummy widget of the Text Editor, for notifications.
extern "C" QLineEdit* get_text_changed_dummy_widget(QWidget* view) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    QLineEdit* dummy = doc_view->findChild<QLineEdit*>("Dummy");
    return dummy;
}

// Function to scroll to a specific row in a text file.
extern "C" void scroll_to_row(QWidget* view, int row_number) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    KTextEditor::Cursor* cursor = new KTextEditor::Cursor(row_number, 0);
    doc_view->setCursorPosition(*cursor);
}

// Function to get the current row of the cursor in a text file.
extern "C" int cursor_row(QWidget* view) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    return doc_view->cursorPosition().line();
}

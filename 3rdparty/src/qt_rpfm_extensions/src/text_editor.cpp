#include "text_editor.h"

// Levels of the diagnostics, matching the order of the variants of DiagnosticLevel in Rust.
static const int DIAGNOSTIC_LEVEL_INFO = 0;
static const int DIAGNOSTIC_LEVEL_WARNING = 1;

static const char* TEXT_ANNOTATIONS_NAME = "TextAnnotations";

// Function to get the annotations helper of a view.
static TextAnnotations* text_annotations(KTextEditor::View* view) {
    return dynamic_cast<TextAnnotations*>(view->findChild<QObject*>(TEXT_ANNOTATIONS_NAME, Qt::FindDirectChildrenOnly));
}

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

    // The icon border is where diagnostic marks are shown.
    view->setConfigValue(QStringLiteral("icon-bar"), true);
    new TextAnnotations(view);

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

    // Annotations belong to the previous text, so they're gone until they're requested again for the new one.
    if (TextAnnotations* annotations = text_annotations(doc_view)) {
        annotations->clear();
    }
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

// Function to scroll to a specific position, and select a range.
extern "C" void scroll_to_pos_and_select(QWidget* view, int start_row, int start_column, int end_row, int end_column) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    KTextEditor::Cursor* start_cursor = new KTextEditor::Cursor(start_row, start_column);
    KTextEditor::Cursor* end_cursor = new KTextEditor::Cursor(end_row, end_column);
    KTextEditor::Range* range = new KTextEditor::Range(*start_cursor, *end_cursor);

    doc_view->setSelection(*range);
    doc_view->setCursorPosition(*start_cursor);
    doc_view->setScrollPosition(*start_cursor);
}

// Function to add a diagnostic to a view. Levels are the ones of DiagnosticLevel: 0 info, 1 warning, 2 error.
extern "C" void add_text_diagnostic(QWidget* view, int start_row, int start_column, int end_row, int end_column, int level, QString* message) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    if (TextAnnotations* annotations = text_annotations(doc_view)) {
        annotations->add_diagnostic(KTextEditor::Range(start_row, start_column, end_row, end_column), level, *message);
    }
}

// Function to remove all diagnostics from a view.
extern "C" void clear_text_diagnostics(QWidget* view) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    if (TextAnnotations* annotations = text_annotations(doc_view)) {
        annotations->clear_diagnostics();
    }
}

// Function to add the docs of a range to a view, shown as rich text when hovering the range.
extern "C" void add_text_hover(QWidget* view, int start_row, int start_column, int end_row, int end_column, QString* html) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    if (TextAnnotations* annotations = text_annotations(doc_view)) {
        annotations->add_hover(KTextEditor::Range(start_row, start_column, end_row, end_column), *html);
    }
}

// Function to remove all docs from a view.
extern "C" void clear_text_hovers(QWidget* view) {

    KTextEditor::View* doc_view = dynamic_cast<KTextEditor::View*>(view);
    if (TextAnnotations* annotations = text_annotations(doc_view)) {
        annotations->clear_hovers();
    }
}

TextAnnotations::TextAnnotations(KTextEditor::View* view) : QObject(view), document(view->document()) {
    setObjectName(TEXT_ANNOTATIONS_NAME);
    view->registerTextHintProvider(this);

    document->setMarkIcon(KTextEditor::Document::Error, QIcon::fromTheme(QStringLiteral("data-error")));
    document->setMarkIcon(KTextEditor::Document::Warning, QIcon::fromTheme(QStringLiteral("data-warning")));

    // The document resets its moving ranges on reload, so drop ours before that happens.
    connect(document, &KTextEditor::Document::aboutToInvalidateMovingInterfaceContent, this, &TextAnnotations::clear);
}

// The view is not unregistered from here because this object is a child of the view, so the view is always destroyed first.
TextAnnotations::~TextAnnotations() {

    // The document is created before the view with the same parent, so it's usually destroyed first. Its ranges can't
    // be safely deleted after that, so they're let go instead.
    if (document.isNull()) {
        for (Annotation& diagnostic : diagnostics) {
            diagnostic.range.release();
        }

        for (Annotation& hover : hovers) {
            hover.range.release();
        }
    }
}

void TextAnnotations::add_diagnostic(KTextEditor::Range range, int level, const QString& message) {
    if (document.isNull()) {
        return;
    }

    // Zero-width ranges, like syntax errors at the end of a line, would get no visible underline.
    if (range.isEmpty()) {
        int line_length = document->lineLength(range.start().line());
        if (range.start().column() < line_length) {
            range.setEnd(KTextEditor::Cursor(range.start().line(), range.start().column() + 1));
        } else if (range.start().column() > 0) {
            range.setStart(KTextEditor::Cursor(range.start().line(), range.start().column() - 1));
        }
    }

    KTextEditor::Attribute::Ptr attribute(new KTextEditor::Attribute());
    attribute->setUnderlineStyle(QTextCharFormat::SpellCheckUnderline);

    if (level == DIAGNOSTIC_LEVEL_INFO) {
        attribute->setUnderlineColor(QColor(Qt::blue));
    } else if (level == DIAGNOSTIC_LEVEL_WARNING) {
        attribute->setUnderlineColor(QColor(255, 165, 0));
        document->addMark(range.start().line(), KTextEditor::Document::Warning);
    } else {
        attribute->setUnderlineColor(QColor(Qt::red));
        document->addMark(range.start().line(), KTextEditor::Document::Error);
    }

    std::unique_ptr<KTextEditor::MovingRange> moving_range(document->newMovingRange(range));
    moving_range->setAttribute(attribute);
    moving_range->setAttributeOnlyForViews(true);
    diagnostics.push_back({std::move(moving_range), message});
}

void TextAnnotations::add_hover(KTextEditor::Range range, const QString& html) {
    if (document.isNull()) {
        return;
    }

    std::unique_ptr<KTextEditor::MovingRange> moving_range(document->newMovingRange(range));
    hovers.push_back({std::move(moving_range), html});
}

void TextAnnotations::clear_diagnostics() {
    diagnostics.clear();
    if (document.isNull()) {
        return;
    }

    // Marks move with their lines on edit, so look for them where they are now instead of where they were added.
    const QList<int> lines = document->marks().keys();
    for (int line : lines) {
        document->removeMark(line, KTextEditor::Document::Error | KTextEditor::Document::Warning);
    }
}

void TextAnnotations::clear_hovers() {
    hovers.clear();
}

void TextAnnotations::clear() {
    clear_diagnostics();
    clear_hovers();
}

QString TextAnnotations::textHint(KTextEditor::View*, const KTextEditor::Cursor& position) {
    auto is_hovered = [&position](const Annotation& annotation) {
        KTextEditor::Range range = annotation.range->toRange();
        return range.contains(position) || range.end() == position;
    };

    // Everything is joined as rich text, so the plain text messages of the diagnostics need escaping.
    QStringList sections;
    for (const Annotation& diagnostic : diagnostics) {
        if (is_hovered(diagnostic)) {
            sections.append(QStringLiteral("<p>%1</p>").arg(diagnostic.text.toHtmlEscaped()));
        }
    }

    for (const Annotation& hover : hovers) {
        if (is_hovered(hover)) {
            sections.append(hover.text);
        }
    }

    return sections.join(QStringLiteral("<hr/>"));
}

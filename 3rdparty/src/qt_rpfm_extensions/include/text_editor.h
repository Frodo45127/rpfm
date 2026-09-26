#ifndef TEXT_EDITOR_H
#define TEXT_EDITOR_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF6/KTextEditor/KTextEditor/Attribute>
#include <KF6/KTextEditor/KTextEditor/Document>
#include <KF6/KTextEditor/KTextEditor/Editor>
#include <KF6/KTextEditor/KTextEditor/MovingRange>
#include <KF6/KTextEditor/KTextEditor/TextHintInterface>
#include <KF6/KTextEditor/KTextEditor/View>
#else
#include <KTextEditor/Attribute>
#include <KTextEditor/Document>
#include <KTextEditor/Editor>
#include <KTextEditor/MovingRange>
#include <KTextEditor/TextHintInterface>
#include <KTextEditor/View>
#endif
#include <QLineEdit>
#include <QPointer>

#include <memory>
#include <vector>

// This one is needed for the save fix.
#include <KActionCollection>

extern "C" QWidget* new_text_editor(QWidget* parent = nullptr);

extern "C" QString* get_text(QWidget* parent = nullptr);

extern "C" void set_text(QWidget* view = nullptr, QString* text = nullptr, QString* highlighting_mode = nullptr);

extern "C" void open_text_editor_config(QWidget* parent);

extern "C" QLineEdit* get_text_changed_dummy_widget(QWidget* view = nullptr);

extern "C" void scroll_to_row(QWidget* view = nullptr, int row_number = 0);

extern "C" void scroll_to_pos_and_select(QWidget* view, int start_row = 0, int start_column = 0, int end_row = 0, int end_column = 0);

extern "C" void add_text_diagnostic(QWidget* view, int start_row, int start_column, int end_row, int end_column, int level, QString* message);

extern "C" void clear_text_diagnostics(QWidget* view);

extern "C" void add_text_hover(QWidget* view, int start_row, int start_column, int end_row, int end_column, QString* html);

extern "C" void clear_text_hovers(QWidget* view);

// Annotations shown in a text editor view, the way an LSP client shows them. Diagnostics are underlined ranges
// that follow the edits, with a mark in the icon border of their lines. Hovering them shows their messages,
// followed by the docs of what's under the cursor, if any.
class TextAnnotations : public QObject, public KTextEditor::TextHintProvider {
public:
    explicit TextAnnotations(KTextEditor::View* view);
    ~TextAnnotations() override;

    void add_diagnostic(KTextEditor::Range range, int level, const QString& message);
    void add_hover(KTextEditor::Range range, const QString& html);
    void clear_diagnostics();
    void clear_hovers();
    void clear();
    QString textHint(KTextEditor::View* view, const KTextEditor::Cursor& position) override;

private:
    struct Annotation {
        std::unique_ptr<KTextEditor::MovingRange> range;
        QString text;
    };

    // Guarded because the document and the view are siblings, so the document may be destroyed first.
    QPointer<KTextEditor::Document> document;
    std::vector<Annotation> diagnostics;
    std::vector<Annotation> hovers;
};
#endif // TEXT_EDITOR_H

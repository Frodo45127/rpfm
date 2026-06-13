#ifndef ELIDING_CHECK_BOX_H
#define ELIDING_CHECK_BOX_H

#include "qt_subclasses_global.h"
#include <QCheckBox>
#include <QString>

// Function to create an eliding checkbox from Rust.
extern "C" QCheckBox* new_eliding_check_box(const QString* text = nullptr, QWidget* parent = nullptr);

// QCheckBox subclass whose label is elided (xxxx...) when there isn't enough horizontal
// space to draw it in full, instead of forcing the layout to keep its full width.
class ElidingCheckBox : public QCheckBox {
    Q_OBJECT
public:
    explicit ElidingCheckBox(const QString& text = "", QWidget* parent = nullptr);

    // Reduced minimum width so the layout is allowed to shrink the checkbox and elide the label.
    QSize minimumSizeHint() const override;

protected:
    void paintEvent(QPaintEvent* event) override;
};

#endif // ELIDING_CHECK_BOX_H

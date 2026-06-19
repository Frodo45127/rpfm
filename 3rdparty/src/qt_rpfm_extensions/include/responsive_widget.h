#ifndef RESPONSIVE_WIDGET_H
#define RESPONSIVE_WIDGET_H

#include "qt_subclasses_global.h"
#include <QWidget>

// A plain QWidget that emits resized() whenever its geometry changes. Used as a container
// whose owner needs to react to width changes (e.g. the table filter bar reflowing its chips
// onto a separate row when it gets too narrow), since QWidget has no built-in resize signal.
extern "C" QWidget* new_responsive_widget(QWidget* parent = nullptr);

class ResponsiveWidget : public QWidget {
    Q_OBJECT

public:
    explicit ResponsiveWidget(QWidget* parent = nullptr);

signals:
    void resized(int width);

protected:
    void resizeEvent(QResizeEvent* event) override;
};

#endif // RESPONSIVE_WIDGET_H

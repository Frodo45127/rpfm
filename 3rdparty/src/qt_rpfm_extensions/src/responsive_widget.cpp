#include "responsive_widget.h"
#include <QResizeEvent>

// Function to create the responsive widget from Rust.
extern "C" QWidget* new_responsive_widget(QWidget* parent) {
    return new ResponsiveWidget(parent);
}

ResponsiveWidget::ResponsiveWidget(QWidget* parent) : QWidget(parent) {}

void ResponsiveWidget::resizeEvent(QResizeEvent* event) {
    QWidget::resizeEvent(event);
    emit resized(event->size().width());
}

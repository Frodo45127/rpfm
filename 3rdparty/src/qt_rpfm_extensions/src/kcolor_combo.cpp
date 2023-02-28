#include "kcolor_combo.h"

extern "C" int get_color(QWidget* view) {
    KColorCombo* combo = dynamic_cast<KColorCombo*>(view);
    return combo->color().rgba();
}

extern "C" void set_color(QWidget* view, QColor* color) {
    KColorCombo* combo = dynamic_cast<KColorCombo*>(view);
    combo->setColor(*color);
    combo->update();
}

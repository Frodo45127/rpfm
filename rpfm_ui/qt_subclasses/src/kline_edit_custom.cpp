#include "kline_edit_custom.h"

extern "C" void kline_edit_configure(QWidget* view) {
    KLineEdit* line_edit = dynamic_cast<KLineEdit*>(view);
    KCompletion *comp = line_edit->completionObject();
    KCompletion::connect(line_edit, SIGNAL(returnPressed(const QString&)), comp, SLOT(addItem(const QString&)));
}

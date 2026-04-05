#include "kline_edit_custom.h"

extern "C" void kline_edit_configure(QWidget* view) {
    KLineEdit* line_edit = dynamic_cast<KLineEdit*>(view);
    KCompletion *comp = line_edit->completionObject();
    QObject::connect(line_edit, &KLineEdit::returnKeyPressed, comp, qOverload<const QString &>(&KCompletion::addItem));
}

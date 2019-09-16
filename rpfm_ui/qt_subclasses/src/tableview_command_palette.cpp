#include "tableview_command_palette.h"
#include "QTableView"

extern "C" QTableView* new_tableview_command_palette() {
    QTableViewCommandPalette* tableview = new QTableViewCommandPalette();
    return dynamic_cast<QTableView*>(tableview);
}

QTableViewCommandPalette::QTableViewCommandPalette(): QTableView() {}

int QTableViewCommandPalette::sizeHintForRow(int row) const {
    return 36;
}

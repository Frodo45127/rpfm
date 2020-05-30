#ifndef COMMAND_PALLETE_H
#define COMMAND_PALLETE_H

#include "qt_subclasses_global.h"
#include "QTableView"

extern "C" QTableView* new_tableview_command_palette();

class QTableViewCommandPalette : public QTableView {
    Q_OBJECT

    public:
        explicit QTableViewCommandPalette();

        int sizeHintForRow(int row) const override;


    signals:

    public slots:

};

#endif // COMMAND_PALLETE_H

#ifndef QDIALOGCUSTOM_H
#define QDIALOGCUSTOM_H

#include <QDialog>
#include <QCloseEvent>
#include <QMoveEvent>
#include <QEvent>
#include <QMessageBox>
#include <QSettings>
#include <KBusyIndicatorWidget>

extern "C" QDialog* new_q_dialog_custom(QWidget *parent = nullptr, bool (*are_you_sure)(QDialog* dialog) = nullptr);

class QDialogCustom : public QDialog
{
    Q_OBJECT
public:
    explicit QDialogCustom(QWidget *parent = nullptr, bool (*are_you_sure)(QDialog* dialog) = nullptr);
    void closeEvent(QCloseEvent *event) override;

private:
    bool (*are_you_sure)(QDialog* dialog);

};

#endif // QDIALOGCUSTOM_H

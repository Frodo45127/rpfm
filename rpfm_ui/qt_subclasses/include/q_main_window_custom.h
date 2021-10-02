#ifndef QMAINWINDOWCUSTOM_H
#define QMAINWINDOWCUSTOM_H

#include <QMainWindow>
#include <QCloseEvent>
#include <QMoveEvent>
#include <QEvent>
#include <QMessageBox>
#include <QSettings>
#include <KBusyIndicatorWidget>

extern "C" QMainWindow* new_q_main_window_custom(bool (*are_you_sure)(QMainWindow* main_window, bool is_delete_my_mod) = nullptr);

class QMainWindowCustom : public QMainWindow
{
    Q_OBJECT

public:
    explicit QMainWindowCustom(QWidget *parent = nullptr, bool (*are_you_sure)(QMainWindow* main_window, bool is_delete_my_mod) = nullptr);
    void closeEvent(QCloseEvent *event);
    void moveEvent(QMoveEvent *event);
    void changeEvent(QEvent *event);

private:
    bool (*are_you_sure)(QMainWindow* main_window, bool is_delete_my_mod);
    KBusyIndicatorWidget* busyIndicator;

signals:

};

#endif // QMAINWINDOWCUSTOM_H

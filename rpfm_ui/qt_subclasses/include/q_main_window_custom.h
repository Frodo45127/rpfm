#ifndef QMAINWINDOWCUSTOM_H
#define QMAINWINDOWCUSTOM_H

#include <QMainWindow>
#include <QCloseEvent>
#include <QMessageBox>
#include <QSettings>

extern "C" QMainWindow* new_q_main_window_custom(bool (*are_you_sure)(QMainWindow* main_window, bool is_delete_my_mod) = nullptr);

class QMainWindowCustom : public QMainWindow
{
    Q_OBJECT

public:
    explicit QMainWindowCustom(QWidget *parent = nullptr, bool (*are_you_sure)(QMainWindow* main_window, bool is_delete_my_mod) = nullptr);
    void closeEvent(QCloseEvent *event);

private:
    bool (*are_you_sure)(QMainWindow* main_window, bool is_delete_my_mod);

signals:

};

#endif // QMAINWINDOWCUSTOM_H

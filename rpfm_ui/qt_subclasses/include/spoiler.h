#include <QFrame>
#include <QGridLayout>
#include <QParallelAnimationGroup>
#include <QScrollArea>
#include <QToolButton>
#include <QWidget>

extern "C" QWidget* new_spoiler(const QString & title = nullptr, const int animationDuration = 50, QWidget *parent = nullptr);
extern "C" void set_spoiler_layout(QWidget *spoiler, QLayout & layout);
extern "C" void toggle_animated(QWidget *spoiler);

class Spoiler : public QWidget {
    Q_OBJECT
private:
    QGridLayout mainLayout;
    QToolButton toggleButton;
    QFrame headerLine;
    QParallelAnimationGroup toggleAnimation;
    QScrollArea contentArea;
    int animationDuration{300};
public:
    explicit Spoiler(const QString & title = "", const int animationDuration = 300, QWidget *parent = 0);
    void setContentLayout(QLayout & contentLayout);
    void toggleAnimated();
};

#ifndef RESIZABLE_LABEL_H
#define RESIZABLE_LABEL_H

#include "qt_subclasses_global.h"
#include <QLabel>

extern "C" QLabel* new_resizable_label(QWidget *parent = nullptr, QPixmap *pixmap = nullptr);
extern "C" void set_pixmap_on_resizable_label(QLabel *label = nullptr, QPixmap *pixmap = nullptr);

class ResizableLabel : public QLabel {

public:
    explicit ResizableLabel(QWidget *parent = 0, QPixmap *pixmap = nullptr);
    virtual int heightForWidth( int width ) const;
    virtual QSize sizeHint() const;
    QPixmap scaledPixmap() const;
    QPixmap pix;
public slots:
    void resizeEvent(QResizeEvent *);
};

#endif // RESIZABLE_LABEL_H

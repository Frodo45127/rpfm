#include "resizable_label.h"
#include <QDebug>

// Function to create the resizable label from Rust.
extern "C" QLabel* new_resizable_label(QWidget *parent, QPixmap *pixmap) {
    ResizableLabel* label = new ResizableLabel(parent, pixmap);
    return dynamic_cast<QLabel*>(label);
}

extern "C" void set_pixmap_on_resizable_label(QLabel *label, QPixmap *pixmap) {
    ResizableLabel* resizable_label = dynamic_cast<ResizableLabel*>(label);
    resizable_label->pix = *pixmap;
    resizable_label->setPixmap(resizable_label->scaledPixmap());
}


ResizableLabel::ResizableLabel(QWidget *parent, QPixmap *pixmap): QLabel(parent) {
    this->setMinimumSize(1,1);
    setScaledContents(false);
    pix = *pixmap;
    setPixmap(scaledPixmap());
}

int ResizableLabel::heightForWidth( int width ) const {
    return pix.isNull() ? this->height() : ((qreal)pix.height()*width)/pix.width();
}

QSize ResizableLabel::sizeHint() const {
    int w = this->width();
    return QSize( w, heightForWidth(w) );
}

QPixmap ResizableLabel::scaledPixmap() const {
    if (pix.height() > this->height()) {
        return pix.scaled(this->size(), Qt::KeepAspectRatio);
    }
    else {
        return pix;
    }
}

void ResizableLabel::resizeEvent(QResizeEvent*) {
    if(!pix.isNull()) {
        QLabel::setPixmap(scaledPixmap());
    }
}

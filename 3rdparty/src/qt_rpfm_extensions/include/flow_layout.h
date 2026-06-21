#ifndef FLOW_LAYOUT_H
#define FLOW_LAYOUT_H

#include "qt_subclasses_global.h"
#include <QLayout>
#include <QRect>
#include <QStyle>
#include <QList>

// Creates a FlowLayout and installs it on `parent` (so `parent->layout()` returns it).
// Used by the table filter bar so its chips wrap onto a new line instead of overflowing
// to the right once they no longer fit on the current line.
extern "C" QLayout* new_flow_layout(QWidget* parent);

// A layout that arranges its items left-to-right and wraps to a new line when the next
// item would not fit in the remaining width. Adapted from Qt's official FlowLayout example.
class FlowLayout : public QLayout {
    Q_OBJECT

public:
    explicit FlowLayout(QWidget* parent, int margin = -1, int hSpacing = -1, int vSpacing = -1);
    explicit FlowLayout(int margin = -1, int hSpacing = -1, int vSpacing = -1);
    ~FlowLayout();

    void addItem(QLayoutItem* item) override;
    int horizontalSpacing() const;
    int verticalSpacing() const;
    Qt::Orientations expandingDirections() const override;
    bool hasHeightForWidth() const override;
    int heightForWidth(int width) const override;
    int count() const override;
    QLayoutItem* itemAt(int index) const override;
    QSize minimumSize() const override;
    void setGeometry(const QRect& rect) override;
    QSize sizeHint() const override;
    QLayoutItem* takeAt(int index) override;

private:
    int doLayout(const QRect& rect, bool testOnly) const;
    int smartSpacing(QStyle::PixelMetric pm) const;

    QList<QLayoutItem*> itemList;
    int m_hSpace;
    int m_vSpace;
};

#endif // FLOW_LAYOUT_H

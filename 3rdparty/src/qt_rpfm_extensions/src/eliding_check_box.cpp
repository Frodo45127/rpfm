#include "eliding_check_box.h"

#include <QStyle>
#include <QStylePainter>
#include <QStyleOptionButton>
#include <QFontMetrics>

// Function to create an eliding checkbox from Rust.
extern "C" QCheckBox* new_eliding_check_box(const QString* text, QWidget* parent) {
    ElidingCheckBox* checkBox = new ElidingCheckBox(text == nullptr ? QString() : *text, parent);
    return dynamic_cast<QCheckBox*>(checkBox);
}

ElidingCheckBox::ElidingCheckBox(const QString& text, QWidget* parent) : QCheckBox(text, parent) {

    // Let the layout shrink us below the full label width (down to minimumSizeHint), while still
    // preferring the full width when there's room for it.
    QSizePolicy policy = sizePolicy();
    policy.setHorizontalPolicy(QSizePolicy::Preferred);
    setSizePolicy(policy);

    // Keep the full text reachable via tooltip, as the label may end up elided.
    setToolTip(text);
}

QSize ElidingCheckBox::minimumSizeHint() const {
    QStyleOptionButton opt;
    initStyleOption(&opt);

    // Reserve room for the indicator, its spacing and a few characters worth of label, so the
    // checkbox can collapse down to "xxx..." instead of forcing its whole label into the layout.
    const int indicator = style()->pixelMetric(QStyle::PM_IndicatorWidth, &opt, this);
    const int spacing = style()->pixelMetric(QStyle::PM_CheckBoxLabelSpacing, &opt, this);
    const int minTextWidth = fontMetrics().horizontalAdvance(QStringLiteral("xxx..."));

    return QSize(indicator + spacing + minTextWidth, QCheckBox::minimumSizeHint().height());
}

void ElidingCheckBox::paintEvent(QPaintEvent*) {
    QStylePainter painter(this);

    QStyleOptionButton opt;
    initStyleOption(&opt);

    // Elide the label to whatever width the style leaves for the checkbox contents.
    const QRect textRect = style()->subElementRect(QStyle::SE_CheckBoxContents, &opt, this);
    opt.text = fontMetrics().elidedText(text(), Qt::ElideRight, textRect.width());

    painter.drawControl(QStyle::CE_CheckBox, opt);
}

// Copyright 2014-2016, Durachenko Aleksey V. <durachenko.aleksey@gmail.com>
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA
#include "qt_long_long_spinbox.h"
#include <QLineEdit>
#include <limits>
#include <QEvent>
#include <QKeyEvent>

// Function to create the resizable label from Rust.
extern "C" QAbstractSpinBox* new_q_spinbox_i64(QWidget* parent) {
    QtLongLongSpinBox* spinbox = new QtLongLongSpinBox(parent);
    return dynamic_cast<QAbstractSpinBox*>(spinbox);
}

extern "C" qlonglong value_q_spinbox_i64(QAbstractSpinBox* widget) {
    QtLongLongSpinBox* spinbox = dynamic_cast<QtLongLongSpinBox*>(widget);
    return spinbox->value();
}

extern "C" void set_value_q_spinbox_i64(QAbstractSpinBox* widget, qlonglong value) {
    QtLongLongSpinBox* spinbox = dynamic_cast<QtLongLongSpinBox*>(widget);
    return spinbox->setValue(value);
}

extern "C" void set_min_q_spinbox_i64(QAbstractSpinBox* widget, qlonglong value) {
    QtLongLongSpinBox* spinbox = dynamic_cast<QtLongLongSpinBox*>(widget);
    return spinbox->setMinimum(value);
}

extern "C" void set_max_q_spinbox_i64(QAbstractSpinBox* widget, qlonglong value) {
    QtLongLongSpinBox* spinbox = dynamic_cast<QtLongLongSpinBox*>(widget);
    return spinbox->setMaximum(value);
}

QtLongLongSpinBox::QtLongLongSpinBox(QWidget *parent) :
    QAbstractSpinBox(parent)
{
    m_minimum = std::numeric_limits<qlonglong>::min();
    m_maximum = std::numeric_limits<qlonglong>::max();
    m_value = 0;
    m_singleStep = 1;

    setValue(m_value);
}

qlonglong QtLongLongSpinBox::value() const
{
    return m_value;
}

void QtLongLongSpinBox::setValue(qlonglong expectedNewValue)
{
    const qlonglong newValue = qBound(m_minimum, expectedNewValue, m_maximum);
    const QString newValueString = QString::number(newValue);
    lineEdit()->setText(m_prefix + newValueString + m_suffix);
    if (m_value != newValue) {
        m_value = newValue;

        emit valueChanged(newValue);
        emit valueChanged(newValueString);
    }
}

QString QtLongLongSpinBox::prefix() const
{
    return m_prefix;
}

void QtLongLongSpinBox::setPrefix(const QString &prefix)
{
    m_prefix = prefix;

    setValue(m_value);
}

QString QtLongLongSpinBox::suffix() const
{
    return m_suffix;
}

void QtLongLongSpinBox::setSuffix(const QString &suffix)
{
    m_suffix = suffix;

    setValue(m_value);
}

QString QtLongLongSpinBox::cleanText() const
{
    return QString::number(m_value);
}

qlonglong QtLongLongSpinBox::singleStep() const
{
    return m_singleStep;
}

void QtLongLongSpinBox::setSingleStep(qlonglong step)
{
    m_singleStep = step;
}

qlonglong QtLongLongSpinBox::minimum() const
{
    return m_minimum;
}

void QtLongLongSpinBox::setMinimum(qlonglong min)
{
    m_minimum = min;
    if (m_maximum < m_minimum) {
        m_maximum = m_minimum;
    }

    setValue(m_value);
}

qlonglong QtLongLongSpinBox::maximum() const
{
    return m_maximum;
}

void QtLongLongSpinBox::setMaximum(qlonglong max)
{
    m_maximum = max;
    if (m_maximum < m_minimum) {
        m_maximum = m_minimum;
    }

    setValue(m_value);
}

void QtLongLongSpinBox::setRange(qlonglong min, qlonglong max)
{
    if (min < max) {
        m_minimum = min;
        m_maximum = max;
    }
    else {
        m_minimum = max;
        m_maximum = min;
    }

    setValue(m_value);
}

void QtLongLongSpinBox::keyPressEvent(QKeyEvent *event)
{
    switch (event->key()) {
    case Qt::Key_Enter:
    case Qt::Key_Return:
        selectCleanText();
        lineEditEditingFinalize();
    }

    QAbstractSpinBox::keyPressEvent(event);
}

void QtLongLongSpinBox::focusOutEvent(QFocusEvent *event)
{
    lineEditEditingFinalize();

    QAbstractSpinBox::focusOutEvent(event);
}

QAbstractSpinBox::StepEnabled QtLongLongSpinBox::stepEnabled() const
{
    if (isReadOnly()) {
        return StepNone;
    }

    StepEnabled se = StepNone;
    if (wrapping() || m_value < m_maximum) {
        se |= StepUpEnabled;
    }
    if (wrapping() || m_value > m_minimum) {
        se |= StepDownEnabled;
    }

    return se;
}

void QtLongLongSpinBox::stepBy(int steps)
{
    if (isReadOnly()) {
        return;
    }

    if (m_prefix + QString::number(m_value) + m_suffix != lineEdit()->text()) {
        lineEditEditingFinalize();
    }

    qlonglong newValue = m_value + (steps * m_singleStep);
    QString newString = QString::number(newValue);
    int pos = 0;
    if (validate(newString, pos) == QValidator::Invalid) {
        return;
    }
    if (wrapping()) {
        // emulating the behavior of QSpinBox
        if (newValue > m_maximum) {
            if (m_value == m_maximum) {
                newValue = m_minimum;
            }
            else {
                newValue = m_maximum;
            }
        }
        else if (newValue < m_minimum) {
            if (m_value == m_minimum) {
                newValue = m_maximum;
            }
            else {
                newValue = m_minimum;
            }
        }
    }
    else {
        newValue = qBound(m_minimum, newValue, m_maximum);
    }

    setValue(newValue);
    selectCleanText();
}

QValidator::State QtLongLongSpinBox::validate(QString &input, int &pos) const
{
    // first, we try to interpret as a number without prefixes
    bool ok;
    const qlonglong value = input.toLongLong(&ok);
    if (ok && m_invalidValues.contains(value)) {
        return QValidator::Invalid;
    }

    if (input.isEmpty() || (ok && value <= m_maximum)) {
        input = m_prefix + input + m_suffix;
        pos += m_prefix.length();
        return QValidator::Acceptable;
    }

    // if string of text editor aren't simple number, try to interpret it
    // as a number with prefix and suffix
    bool valid = true;
    if (!m_prefix.isEmpty() && !input.startsWith(m_prefix)) {
        valid = false;
    }
    if (!m_suffix.isEmpty() && !input.endsWith(m_suffix)) {
        valid = false;
    }

    if (valid) {
        const int start = m_prefix.length();
        const int length = input.length() - start - m_suffix.length();

        bool ok;
        const QString number = input.mid(start, length);
        const qlonglong value = number.toLongLong(&ok);
        if (number.isEmpty() || (ok && value <= m_maximum)) {
            return QValidator::Acceptable;
        }
    }

    // otherwise not acceptable
    return QValidator::Invalid;
}

void QtLongLongSpinBox::lineEditEditingFinalize()
{
    const QString text = lineEdit()->text();

    // first, we try to read as a number without prefixes
    bool ok;
    qlonglong value = text.toLongLong(&ok);
    if (ok) {
        setValue(value);
        return;
    }

    // if string of text editor aren't simple number, try to interpret it
    // as a number with prefix and suffix
    bool valid = true;
    if (!m_prefix.isEmpty() && !text.startsWith(m_prefix)) {
        valid = false;
    }
    else if (!m_suffix.isEmpty() && !text.endsWith(m_suffix)) {
        valid = false;
    }

    if (valid) {
        const int start = m_prefix.length();
        const int length = text.length() - start - m_suffix.length();

        bool ok;
        const qlonglong value = text.mid(start, length).toLongLong(&ok);
        if (ok) {
            setValue(value);
            return;
        }
    }

    // otherwise set old value
    setValue(m_value);
}

void QtLongLongSpinBox::selectCleanText()
{
    lineEdit()->setSelection(m_prefix.length(),
                             lineEdit()->text().length()
                                 - m_prefix.length()
                                 - m_suffix.length());
}

void QtLongLongSpinBox::setInvalidValues(QList<qlonglong> values)
{
    m_invalidValues = values;
}


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
#ifndef QT_LONG_LONG_SPINBOX_H
#define QT_LONG_LONG_SPINBOX_H

#include "qt_subclasses_global.h"
#include <QAbstractSpinBox>
#include <QtGlobal>

extern "C" QAbstractSpinBox* new_q_spinbox_i64(QWidget* view = nullptr);

class QtLongLongSpinBox : public QAbstractSpinBox
{
    Q_OBJECT
public:
    explicit QtLongLongSpinBox(QWidget *parent = 0);

    qlonglong value() const;

    QString prefix() const;
    void setPrefix(const QString &prefix);

    QString suffix() const;
    void setSuffix(const QString &suffix);

    QString cleanText() const;

    qlonglong singleStep() const;
    void setSingleStep(qlonglong val);

    qlonglong minimum() const;
    void setMinimum(qlonglong min);

    qlonglong maximum() const;
    void setMaximum(qlonglong max);

    void setRange(qlonglong min, qlonglong max);

public slots:
    void setValue(qlonglong value);

signals:
    void valueChanged(qlonglong i);
    void valueChanged(const QString &text);

protected:
    virtual void keyPressEvent(QKeyEvent *event);
    virtual void focusOutEvent(QFocusEvent *event);
    virtual void stepBy(int steps);
    virtual StepEnabled stepEnabled() const;
    virtual QValidator::State validate(QString & input, int &pos) const;

private:
    void lineEditEditingFinalize();
    void selectCleanText();

private:
    QString m_prefix;
    QString m_suffix;
    qlonglong m_singleStep;
    qlonglong m_minimum;
    qlonglong m_maximum;
    qlonglong m_value;

private:
    Q_DISABLE_COPY(QtLongLongSpinBox)
};


#endif // QT_LONG_LONG_SPINBOX_H

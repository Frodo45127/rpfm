#ifndef KSHORTCUTS_DIALOG_H
#define KSHORTCUTS_DIALOG_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF5/KXmlGui/KShortcutsDialog>
#include <KF5/KTextEditor/ktexteditor/Document>
#include <KF5/KTextEditor/ktexteditor/Editor>
#include <KF5/KTextEditor/ktexteditor/View>
#else
#include <KShortcutsDialog>
#include <KTextEditor/Document>
#include <KTextEditor/Editor>
#include <KTextEditor/View>
#endif
#include <QWidget>
#include <QString>
#include <QAction>

#endif // KSHORTCUTS_DIALOG_H

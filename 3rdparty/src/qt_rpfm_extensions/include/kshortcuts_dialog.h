#ifndef KSHORTCUTS_DIALOG_H
#define KSHORTCUTS_DIALOG_H

#include "qt_subclasses_global.h"
#ifdef _WIN32
#include <KF6/KXmlGui/KShortcutsDialog>
#include <KF6/KTextEditor/KTextEditor/Document>
#include <KF6/KTextEditor/KTextEditor/Editor>
#include <KF6/KTextEditor/KTextEditor/View>
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

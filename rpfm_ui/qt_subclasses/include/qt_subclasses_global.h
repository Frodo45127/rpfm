#ifndef QT_SUBCLASSES_GLOBAL_H

#define QT_SUBCLASSES_GLOBAL_H

#include <QtCore/qglobal.h>
#ifdef _WIN32
#include <KF5/KTextEditor/ktexteditor_export.h>
#endif
#if defined(QT_SUBCLASSES__LIBRARY)
#  define QT_CUSTOM_RPFMSHARED_EXPORT Q_DECL_EXPORT
#else
#  define QT_SUBCLASSES_SHARED_EXPORT Q_DECL_IMPORT
#endif

#endif // QT_SUBCLASSES__GLOBAL_H

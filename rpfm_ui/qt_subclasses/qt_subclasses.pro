#-------------------------------------------------
#
# Project created by QtCreator 2018-09-28T23:28:57
#
#-------------------------------------------------

QT       += widgets
QT       += KTextEditor
QT       += KWidgetsAddons

TARGET = qt_subclasses
TEMPLATE = lib

# We only want the release version, as this lib is not going to get "advanced" stuff.
# In case you want to build the debug version, change the following line, removing the "release".
CONFIG += staticlib release

DEFINES += QT_SUBCLASSES_LIBRARY

# The following define makes your compiler emit warnings if you use
# any feature of Qt which has been marked as deprecated (the exact warnings
# depend on your compiler). Please consult the documentation of the
# deprecated API in order to know how to port your code away from it.
DEFINES += QT_DEPRECATED_WARNINGS

# You can also make your code fail to compile if you use deprecated APIs.
# In order to do so, uncomment the following line.
# You can also select to disable deprecated APIs only up to a certain version of Qt.
DEFINES += QT_DISABLE_DEPRECATED_BEFORE=0x060000    # disables all the APIs deprecated before Qt 6.0.0

SOURCES += \
    src/colour_item_delegate.cpp \
    src/extended_q_styled_item_delegate.cpp \
    src/kcolor_combo.cpp \
    src/kmessage_widget.cpp \
    src/q_main_window_custom.cpp \
    src/packed_file_model.cpp \
    src/qstring_item_delegate.cpp \
    src/combobox_item_delegate.cpp \
    src/resizable_label.cpp \
    src/spinbox_item_delegate.cpp \
    src/doublespinbox_item_delegate.cpp \
    src/spoiler.cpp \
    src/tableview_command_palette.cpp \
    src/tableview_filter.cpp \
    src/tableview_frozen.cpp \
    src/text_editor.cpp \
    src/tips_item_delegate.cpp \
    src/tree_item_delegate.cpp \
    src/treeview_filter.cpp

INCLUDEPATH += include
INCLUDEPATH += C:\CraftRoot\include

HEADERS += \
    include/colour_item_delegate.h \
    include/extended_q_styled_item_delegate.h \
    include/kmessage_widget.h \
    include/qt_subclasses_global.h \
    include/tableview_command_palette.h \
    include/tableview_filter.h \
    include/tableview_frozen.h \
    include/combobox_item_delegate.h \
    include/spinbox_item_delegate.h \
    include/doublespinbox_item_delegate.h \
    include/text_editor.h \
    include/tips_item_delegate.h \
    include/tree_item_delegate.h \
    include/treeview_filter.h \
    include/qstring_item_delegate.h \
    include/packed_file_model.h \
    include/resizable_label.h \
    include/q_main_window_custom.h \
    include/spoiler.h \
    include/kcolor_combo.h

release:DESTDIR = release
release:OBJECTS_DIR = release/.obj
release:MOC_DIR = release/.moc
release:RCC_DIR = release/.rcc
release:UI_DIR = release/.ui

debug:DESTDIR = debug
debug:OBJECTS_DIR = debug/.obj
debug:MOC_DIR = debug/.moc
debug:RCC_DIR = debug/.rcc
debug:UI_DIR = debug/.ui

unix {

    # For some reason, these flags fuck up compilation on windows, so we leave them linux only.
    QMAKE_CXXFLAGS = -Wl,-rpath='${ORIGIN}'
}

# This means we generate all the artifacts in target and drop the final lib in libs.
DESTDIR         = ../../libs
BASEDIR         = ../../target/qt_subclasses
MOC_DIR         = ../../target/qt_subclasses/moc
OBJECTS_DIR     = ../../target/qt_subclasses/obj

# Fix for make failing due to missing folders.
commands = ; $(MKDIR -p) BASEDIR; $(MKDIR -p) $MOC_DIR; $(MKDIR -p) $OBJECTS_DIR

#include "kshortcuts_dialog.h"

#include "kactioncollection.h"

void new_action(KActionCollection* actions, QString const action_name, Qt::ShortcutContext context, QList<QKeySequence> shortcut) {
    QAction* action = actions->addAction(action_name);
    action->setText(action_name);
    action->setShortcutContext(context);
    actions->setDefaultShortcuts(action, shortcut);
}

extern "C" void shortcut_collection_init(QWidget* parent, QList<QObject*>* shortcuts) {

    // Pack Menu actions.
    KActionCollection* pack_menu_actions = new KActionCollection(parent, "pack_menu");
    pack_menu_actions->setComponentDisplayName("Pack Menu");
    new_action(pack_menu_actions, "New Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+N"));
    new_action(pack_menu_actions, "Open Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+O"));
    new_action(pack_menu_actions, "Save Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+S"));
    new_action(pack_menu_actions, "Save Pack As", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+S"));
    new_action(pack_menu_actions, "Install Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+I"));
    new_action(pack_menu_actions, "Uninstall Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+U"));
    new_action(pack_menu_actions, "Load All CA Packs", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+G"));
    new_action(pack_menu_actions, "Settings", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+P"));
    new_action(pack_menu_actions, "Quit", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    pack_menu_actions->readSettings();

    // MyMod Menu actions.
    KActionCollection* mymod_menu_actions = new KActionCollection(parent, "mymod_menu");
    mymod_menu_actions->setComponentDisplayName("MyMod Menu");
    new_action(mymod_menu_actions, "Open MyMod Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "New MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "Delete Open MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "Import MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "Export MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    mymod_menu_actions->readSettings();

    // View Menu actions.
    KActionCollection* view_menu_actions = new KActionCollection(parent, "view_menu");
    view_menu_actions->setComponentDisplayName("View Menu");
    new_action(view_menu_actions, "Pack Contents Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(view_menu_actions, "Global Search Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+F"));
    new_action(view_menu_actions, "Diagnostics Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(view_menu_actions, "Dependencies Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(view_menu_actions, "References Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    view_menu_actions->readSettings();

    // Game Selected Menu actions.
    KActionCollection* game_selected_menu_actions = new KActionCollection(parent, "game_selected_menu");
    game_selected_menu_actions->setComponentDisplayName("Game Selected Menu");
    new_action(game_selected_menu_actions, "Launch Game", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(game_selected_menu_actions, "Open Game Data Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(game_selected_menu_actions, "Open Game Assembly Kit Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(game_selected_menu_actions, "Open RPFM Config Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    game_selected_menu_actions->readSettings();

    // Special Stuff Menu actions.
    KActionCollection* special_stuff_menu_actions = new KActionCollection(parent, "special_stuff_menu");
    special_stuff_menu_actions->setComponentDisplayName("Special Stuff Menu");
    new_action(special_stuff_menu_actions, "Generate Dependencies Cache", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(special_stuff_menu_actions, "Optimize Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(special_stuff_menu_actions, "Patch SiegeAI", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    special_stuff_menu_actions->readSettings();

    // About Menu actions.
    KActionCollection* about_menu_actions = new KActionCollection(parent, "about_menu");
    about_menu_actions->setComponentDisplayName("About Menu");
    new_action(about_menu_actions, "About Qt", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "About RPFM", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "Open Manual", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+H"));
    new_action(about_menu_actions, "Support Me On Patreon", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "Check Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+U"));
    new_action(about_menu_actions, "Check Schema Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+U"));
    new_action(about_menu_actions, "Check Message Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "Check TW Autogen Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    about_menu_actions->readSettings();

    // File Tab actions.
    KActionCollection* file_tab_actions = new KActionCollection(parent, "file_tab");
    file_tab_actions->setComponentDisplayName("File Tabs");
    new_action(file_tab_actions, "Close Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+W"));
    new_action(file_tab_actions, "Close All Tabs", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "Close All Tabs to the Left", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "Close All Tabs to the Right", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "Previous Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+Tab"));
    new_action(file_tab_actions, "Next Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Tab"));
    new_action(file_tab_actions, "Import From Dependencies", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "Toggle Tips", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    file_tab_actions->readSettings();

    // Pack Tree Context actions.
    KActionCollection* pack_tree_actions = new KActionCollection(parent, "pack_tree_context_menu");
    pack_tree_actions->setComponentDisplayName("Pack Tree Context Menu");

    new_action(pack_tree_actions, "Add File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+A"));
    new_action(pack_tree_actions, "Add Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+A"));
    new_action(pack_tree_actions, "Add From Pack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Alt+A"));
    new_action(pack_tree_actions, "New Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+F"));
    new_action(pack_tree_actions, "New AnimPack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "New DB", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"));
    new_action(pack_tree_actions, "New Loc", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+L"));
    new_action(pack_tree_actions, "New Text", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+T"));
    new_action(pack_tree_actions, "New Quick File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Q"));
    new_action(pack_tree_actions, "Merge Tables", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+M"));
    new_action(pack_tree_actions, "Update Tables", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "Generate Missing Loc Data", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"));
    new_action(pack_tree_actions, "Extract", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+E"));
    new_action(pack_tree_actions, "Rename", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+R"));
    new_action(pack_tree_actions, "Copy Path", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "Open In Decoder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+J"));
    new_action(pack_tree_actions, "Open Dependency Manager", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "Open In External Program", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+K"));
    new_action(pack_tree_actions, "Open Containing Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "Open Pack Settings", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "Open Pack Notes", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Y"));
    new_action(pack_tree_actions, "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"));
    new_action(pack_tree_actions, "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    pack_tree_actions->readSettings();

    // Dependencies Tree Context actions.
    KActionCollection* dependencies_tree_actions = new KActionCollection(parent, "dependencies_context_menu");
    dependencies_tree_actions->setComponentDisplayName("Dependencies Tree Context Menu");
    new_action(dependencies_tree_actions, "Copy Path", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(dependencies_tree_actions, "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"));
    new_action(dependencies_tree_actions, "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    new_action(dependencies_tree_actions, "Import From Dependencies", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    dependencies_tree_actions->readSettings();

    // Table Editor actions.
    KActionCollection* table_editor_actions = new KActionCollection(parent, "table_editor");
    table_editor_actions->setComponentDisplayName("Table Editor");
    new_action(table_editor_actions, "Add Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+A"));
    new_action(table_editor_actions, "Insert Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+I"));
    new_action(table_editor_actions, "Delete Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"));
    new_action(table_editor_actions, "Delete Filtered Out Rows", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+Del"));
    new_action(table_editor_actions, "Clone And Insert Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"));
    new_action(table_editor_actions, "Clone And Append Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+D"));
    new_action(table_editor_actions, "Copy", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+C"));
    new_action(table_editor_actions, "Copy as LUA Table", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+C"));
    new_action(table_editor_actions, "Copy to Filter Value", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Paste", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+V"));
    new_action(table_editor_actions, "Paste as New Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+V"));
    new_action(table_editor_actions, "Rewrite Selection", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Y"));
    new_action(table_editor_actions, "Invert Selection", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    new_action(table_editor_actions, "Generate IDs", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Reset Selected Values", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Import TSV", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Export TSV", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Search", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+F"));
    new_action(table_editor_actions, "Sidebar", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Undo", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Z"));
    new_action(table_editor_actions, "Redo", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+Z"));
    new_action(table_editor_actions, "Smart Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"));
    new_action(table_editor_actions, "Resize Columns", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Rename References", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "Go To Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    table_editor_actions->readSettings();

    // Decoder actions.
    KActionCollection* decoder_actions = new KActionCollection(parent, "decoder");
    decoder_actions->setComponentDisplayName("Decoder");
    new_action(decoder_actions, "Move Field Up", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Up"));
    new_action(decoder_actions, "Move Field Down", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Down"));
    new_action(decoder_actions, "Move Field Left", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Left"));
    new_action(decoder_actions, "Move Field Right", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Right"));
    new_action(decoder_actions, "Delete Field", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"));
    new_action(decoder_actions, "Delete Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"));
    new_action(decoder_actions, "Load Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+L"));
    decoder_actions->readSettings();

    // Text Editor actions.
    KTextEditor::Editor *editor = KTextEditor::Editor::instance();
    KTextEditor::Document *doc = editor->createDocument(nullptr);
    KTextEditor::View *view = doc->createView(nullptr);
    KActionCollection* text_editor_actions = view->actionCollection();
    text_editor_actions->readSettings();

    // Add all the actions to our list.
    shortcuts->append(dynamic_cast<QObject*>(pack_menu_actions));
    shortcuts->append(dynamic_cast<QObject*>(mymod_menu_actions));
    shortcuts->append(dynamic_cast<QObject*>(view_menu_actions));
    shortcuts->append(dynamic_cast<QObject*>(game_selected_menu_actions));
    shortcuts->append(dynamic_cast<QObject*>(special_stuff_menu_actions));
    shortcuts->append(dynamic_cast<QObject*>(about_menu_actions));
    shortcuts->append(dynamic_cast<QObject*>(file_tab_actions));
    shortcuts->append(dynamic_cast<QObject*>(pack_tree_actions));
    shortcuts->append(dynamic_cast<QObject*>(dependencies_tree_actions));
    shortcuts->append(dynamic_cast<QObject*>(table_editor_actions));
    shortcuts->append(dynamic_cast<QObject*>(decoder_actions));
    shortcuts->append(dynamic_cast<QObject*>(text_editor_actions));
}

extern "C" QAction* shortcut_action(QList<QObject*> const &shortcuts, QString const action_group, QString const action_name) {
    foreach (QObject* collection, shortcuts) {
        KActionCollection* actions = dynamic_cast<KActionCollection*>(collection);

        if (actions->componentName() == action_group) {
            QAction* action = actions->action(action_name);
            if (action != nullptr) {
                return action;
            }
        }
    }
    return nullptr;
}

extern "C" void shortcut_associate_action_group_to_widget(QList<QObject*>* shortcuts, QString const action_group, QWidget* widget) {
    QList<QObject *>::iterator i;
    for (i = shortcuts->begin(); i != shortcuts->end(); ++i) {
        KActionCollection* actions = dynamic_cast<KActionCollection*>(*i);
        if (actions->componentName() == action_group) {
            actions->associateWidget(widget);
            break;
        }
    }
}


extern "C" void kshortcut_dialog_init(QWidget* widget, QList<QObject*>* shortcuts) {
    KShortcutsDialog* dialog = new KShortcutsDialog(widget);

    QList<QObject *>::iterator i;
    for (i = shortcuts->begin(); i != shortcuts->end(); ++i) {
        KActionCollection* actions = dynamic_cast<KActionCollection*>(*i);
        dialog->addCollection(actions);
    }

    dialog->setAttribute(Qt::WA_DeleteOnClose);
    dialog->configure(true);
}

#include "kshortcuts_dialog.h"

#include "kactioncollection.h"

void new_action(KActionCollection* actions, QString const action_name, QString const display_name, Qt::ShortcutContext context, QList<QKeySequence> shortcut) {
    QAction* action = actions->addAction(action_name);
    action->setText(display_name);
    action->setShortcutContext(context);
    actions->setDefaultShortcuts(action, shortcut);
}

extern "C" void shortcut_collection_init(QWidget* parent, QList<QObject*>* shortcuts) {

    // Pack Menu actions.
    KActionCollection* pack_menu_actions = new KActionCollection(parent, "pack_menu");
    pack_menu_actions->setComponentDisplayName("Pack Menu");
    new_action(pack_menu_actions, "new_pack", "New Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+N"));
    new_action(pack_menu_actions, "open_pack", "Open Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+O"));
    new_action(pack_menu_actions, "save_pack", "Save Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+S"));
    new_action(pack_menu_actions, "save_pack_as", "Save Pack As", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+S"));
    new_action(pack_menu_actions, "install_pack", "Install Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+I"));
    new_action(pack_menu_actions, "uninstall_pack", "Uninstall Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+U"));
    new_action(pack_menu_actions, "load_all_ca_packs", "Load All CA Packs", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+G"));
    new_action(pack_menu_actions, "settings", "Settings", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+P"));
    new_action(pack_menu_actions, "quit", "Quit", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    pack_menu_actions->readSettings();

    // MyMod Menu actions.
    KActionCollection* mymod_menu_actions = new KActionCollection(parent, "mymod_menu");
    mymod_menu_actions->setComponentDisplayName("MyMod Menu");
    new_action(mymod_menu_actions, "open_mymod_folder", "Open MyMod Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "new_mymod", "New MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "delete_mymod", "Delete Open MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "import_mymod", "Import MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(mymod_menu_actions, "export_mymod", "Export MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    mymod_menu_actions->readSettings();

    // View Menu actions.
    KActionCollection* view_menu_actions = new KActionCollection(parent, "view_menu");
    view_menu_actions->setComponentDisplayName("View Menu");
    new_action(view_menu_actions, "pack_contents_panel", "Pack Contents Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(view_menu_actions, "global_search_panel", "Global Search Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+F"));
    new_action(view_menu_actions, "diagnostics_panel", "Diagnostics Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(view_menu_actions, "dependencies_panel", "Dependencies Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(view_menu_actions, "references_panel", "References Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    view_menu_actions->readSettings();

    // Game Selected Menu actions.
    KActionCollection* game_selected_menu_actions = new KActionCollection(parent, "game_selected_menu");
    game_selected_menu_actions->setComponentDisplayName("Game Selected Menu");
    new_action(game_selected_menu_actions, "launch_game", "Launch Game", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(game_selected_menu_actions, "open_game_data_folder", "Open Game Data Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(game_selected_menu_actions, "open_game_ak_folder", "Open Game Assembly Kit Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(game_selected_menu_actions, "open_rpfm_config_folder", "Open RPFM Config Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    game_selected_menu_actions->readSettings();

    // Special Stuff Menu actions.
    KActionCollection* special_stuff_menu_actions = new KActionCollection(parent, "special_stuff_menu");
    special_stuff_menu_actions->setComponentDisplayName("Special Stuff Menu");
    new_action(special_stuff_menu_actions, "generate_dependencies_cache", "Generate Dependencies Cache", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(special_stuff_menu_actions, "optimize_pack", "Optimize Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(special_stuff_menu_actions, "patch_siege_ai", "Patch SiegeAI", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    special_stuff_menu_actions->readSettings();

    // About Menu actions.
    KActionCollection* about_menu_actions = new KActionCollection(parent, "about_menu");
    about_menu_actions->setComponentDisplayName("About Menu");
    new_action(about_menu_actions, "about_qt", "About Qt", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "about_rpfm", "About RPFM", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "open_manual", "Open Manual", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+H"));
    new_action(about_menu_actions, "support_me_on_patreon", "Support Me On Patreon", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "check_updates", "Check Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+U"));
    new_action(about_menu_actions, "check_schema_updates", "Check Schema Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+U"));
    new_action(about_menu_actions, "check_message_updates", "Check Message Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(about_menu_actions, "check_tw_autogen_updates", "Check TW Autogen Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    about_menu_actions->readSettings();

    // File Tab actions.
    KActionCollection* file_tab_actions = new KActionCollection(parent, "file_tab");
    file_tab_actions->setComponentDisplayName("File Tabs");
    new_action(file_tab_actions, "close_tab", "Close Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+W"));
    new_action(file_tab_actions, "close_other_tabs", "Close All Tabs", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "close_other_tabs_left", "Close All Tabs to the Left", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "close_other_tabs_right", "Close All Tabs to the Right", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "previus_tab", "Previous Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+Tab"));
    new_action(file_tab_actions, "next_tab", "Next Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Tab"));
    new_action(file_tab_actions, "import_from_dependencies", "Import From Dependencies", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    new_action(file_tab_actions, "toggle_tips", "Toggle Tips", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""));
    file_tab_actions->readSettings();

    // Pack Tree Context actions.
    KActionCollection* pack_tree_actions = new KActionCollection(parent, "pack_tree_context_menu");
    pack_tree_actions->setComponentDisplayName("Pack Tree Context Menu");

    new_action(pack_tree_actions, "add_file", "Add File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+A"));
    new_action(pack_tree_actions, "add_folder", "Add Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+A"));
    new_action(pack_tree_actions, "add_from_pack", "Add From Pack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Alt+A"));
    new_action(pack_tree_actions, "new_folder", "New Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+F"));
    new_action(pack_tree_actions, "new_animpack", "New AnimPack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "new_db", "New DB", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"));
    new_action(pack_tree_actions, "new_loc", "New Loc", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+L"));
    new_action(pack_tree_actions, "new_text", "New Text", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+T"));
    new_action(pack_tree_actions, "new_quick_file", "New Quick File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Q"));
    new_action(pack_tree_actions, "merge_files", "Merge Files", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+M"));
    new_action(pack_tree_actions, "update_files", "Update Tables", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "generate_missing_loc_data", "Generate Missing Loc Data", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "delete", "Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"));
    new_action(pack_tree_actions, "extract", "Extract", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+E"));
    new_action(pack_tree_actions, "rename", "Rename", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+R"));
    new_action(pack_tree_actions, "copy_path", "Copy Path", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "open_in_decoder", "Open In Decoder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+J"));
    new_action(pack_tree_actions, "open_dependency_manager", "Open Dependency Manager", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "open_in_external_program", "Open In External Program", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+K"));
    new_action(pack_tree_actions, "open_containing_folder", "Open Containing Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "open_pack_settings", "Open Pack Settings", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(pack_tree_actions, "open_pack_notes", "Open Pack Notes", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Y"));
    new_action(pack_tree_actions, "expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"));
    new_action(pack_tree_actions, "collapse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    pack_tree_actions->readSettings();

    // Dependencies Tree Context actions.
    KActionCollection* dependencies_tree_actions = new KActionCollection(parent, "dependencies_context_menu");
    dependencies_tree_actions->setComponentDisplayName("Dependencies Tree Context Menu");
    new_action(dependencies_tree_actions, "copy_path", "Copy Path", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(dependencies_tree_actions, "expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"));
    new_action(dependencies_tree_actions, "collapsse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    new_action(dependencies_tree_actions, "import_from_dependencies", "Import From Dependencies", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    dependencies_tree_actions->readSettings();

    // AnimPack Tree Context actions.
    KActionCollection* anim_pack_tree_actions = new KActionCollection(parent, "anim_pack_tree_context_menu");
    anim_pack_tree_actions->setComponentDisplayName("AnimPack Tree Context Menu");
    new_action(anim_pack_tree_actions, "delete", "Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"));
    new_action(anim_pack_tree_actions, "expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"));
    new_action(anim_pack_tree_actions, "pack_expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"));
    new_action(anim_pack_tree_actions, "collapse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    new_action(anim_pack_tree_actions, "pack_collapse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    anim_pack_tree_actions->readSettings();

    // Table Editor actions.
    KActionCollection* table_editor_actions = new KActionCollection(parent, "table_editor");
    table_editor_actions->setComponentDisplayName("Table Editor");
    new_action(table_editor_actions, "add_row", "Add Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+A"));
    new_action(table_editor_actions, "insert_row", "Insert Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+I"));
    new_action(table_editor_actions, "delete_row", "Delete Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"));
    new_action(table_editor_actions, "delete_filtered_out_row", "Delete Filtered Out Rows", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+Del"));
    new_action(table_editor_actions, "clone_and_insert_row", "Clone And Insert Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"));
    new_action(table_editor_actions, "clone_and_append_row", "Clone And Append Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+D"));
    new_action(table_editor_actions, "copy", "Copy", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+C"));
    new_action(table_editor_actions, "copy_as_lua_table", "Copy as LUA Table", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+C"));
    new_action(table_editor_actions, "copy_as_filter_value", "Copy to Filter Value", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "paste", "Paste", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+V"));
    new_action(table_editor_actions, "paste_as_new_row", "Paste as New Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+V"));
    new_action(table_editor_actions, "rewrite_selection", "Rewrite Selection", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Y"));
    new_action(table_editor_actions, "invert_selection", "Invert Selection", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"));
    new_action(table_editor_actions, "generate_ids", "Generate IDs", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "reset_selected_values", "Reset Selected Values", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "import_tsv", "Import TSV", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "export_tsv", "Export TSV", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "search", "Search", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+F"));
    new_action(table_editor_actions, "sidebar", "Sidebar", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "undo", "Undo", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Z"));
    new_action(table_editor_actions, "redo", "Redo", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+Z"));
    new_action(table_editor_actions, "smart_delete", "Smart Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"));
    new_action(table_editor_actions, "resize_columns", "Resize Columns", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "rename_references", "Rename References", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "patch_columns", "Patch Columns", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "find_references", "Find References", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    new_action(table_editor_actions, "go_to_definition", "Go To Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""));
    table_editor_actions->readSettings();

    // Decoder actions.
    KActionCollection* decoder_actions = new KActionCollection(parent, "decoder");
    decoder_actions->setComponentDisplayName("Decoder");
    new_action(decoder_actions, "move_field_up", "Move Field Up", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Up"));
    new_action(decoder_actions, "move_field_down", "Move Field Down", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Down"));
    new_action(decoder_actions, "move_field_left", "Move Field Left", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Left"));
    new_action(decoder_actions, "move_field_right", "Move Field Right", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Right"));
    new_action(decoder_actions, "delete_field", "Delete Field", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"));
    new_action(decoder_actions, "delete_definition", "Delete Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"));
    new_action(decoder_actions, "load_definition", "Load Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+L"));
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
    shortcuts->append(dynamic_cast<QObject*>(anim_pack_tree_actions));
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

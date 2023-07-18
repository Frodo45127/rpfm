#include "kshortcuts_dialog.h"

#include "kactioncollection.h"

void new_action(KActionCollection* actions, QString const action_name, QString const display_name, Qt::ShortcutContext context, QList<QKeySequence> shortcut, QString const icon_name = "") {
    QAction* action = actions->addAction(action_name);
    action->setText(display_name);
    action->setIcon(QIcon::fromTheme(icon_name));
    action->setShortcutContext(context);
    actions->setDefaultShortcuts(action, shortcut);
}

extern "C" void shortcut_collection_init(QWidget* parent, QList<QObject*>* shortcuts) {

    // Pack Menu actions.
    KActionCollection* pack_menu_actions = new KActionCollection(parent, "pack_menu");
    pack_menu_actions->setComponentDisplayName("Pack Menu");
    new_action(pack_menu_actions, "new_pack", "New Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+N"), "project-development-new-template");
    new_action(pack_menu_actions, "open_pack", "Open Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+O"), "project-open");
    new_action(pack_menu_actions, "save_pack", "Save Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+S"), "document-save");
    new_action(pack_menu_actions, "save_pack_as", "Save Pack As", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+S"), "document-save-as");
    new_action(pack_menu_actions, "install_pack", "Install Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+I"), "format-align-vertical-top");
    new_action(pack_menu_actions, "uninstall_pack", "Uninstall Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+U"), "format-align-vertical-bottom");
    new_action(pack_menu_actions, "load_all_ca_packs", "Load All CA Packs", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+G"), "dialog-object-properties");
    new_action(pack_menu_actions, "settings", "Settings", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+P"), "settings-configure");
    new_action(pack_menu_actions, "quit", "Quit", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "gtk-quit");
    pack_menu_actions->readSettings();

    // MyMod Menu actions.
    KActionCollection* mymod_menu_actions = new KActionCollection(parent, "mymod_menu");
    mymod_menu_actions->setComponentDisplayName("MyMod Menu");
    new_action(mymod_menu_actions, "open_mymod_folder", "Open MyMod Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "document-open-folder");
    new_action(mymod_menu_actions, "new_mymod", "New MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "project-development-new-template");
    new_action(mymod_menu_actions, "delete_mymod", "Delete Open MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "draw-eraser-delete-objects");
    new_action(mymod_menu_actions, "import_mymod", "Import MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Alt+I"), "document-import");
    new_action(mymod_menu_actions, "export_mymod", "Export MyMod", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Alt+E"), "document-export");
    mymod_menu_actions->readSettings();

    // View Menu actions.
    KActionCollection* view_menu_actions = new KActionCollection(parent, "view_menu");
    view_menu_actions->setComponentDisplayName("View Menu");
    new_action(view_menu_actions, "pack_contents_panel", "Pack Contents Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "builder-view-left-pane-symbolic");
    new_action(view_menu_actions, "global_search_panel", "Global Search Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+F"), "builder-view-left-pane-symbolic");
    new_action(view_menu_actions, "diagnostics_panel", "Diagnostics Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "builder-view-left-pane-symbolic");
    new_action(view_menu_actions, "dependencies_panel", "Dependencies Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "builder-view-left-pane-symbolic");
    new_action(view_menu_actions, "references_panel", "References Panel", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "builder-view-left-pane-symbolic");
    view_menu_actions->readSettings();

    // Game Selected Menu actions.
    KActionCollection* game_selected_menu_actions = new KActionCollection(parent, "game_selected_menu");
    game_selected_menu_actions->setComponentDisplayName("Game Selected Menu");
    new_action(game_selected_menu_actions, "launch_game", "Launch Game", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "media-playback-start-symbolic");
    new_action(game_selected_menu_actions, "open_game_data_folder", "Open Game Data Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "document-open-folder");
    new_action(game_selected_menu_actions, "open_game_ak_folder", "Open Game Assembly Kit Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "document-open-folder");
    new_action(game_selected_menu_actions, "open_rpfm_config_folder", "Open RPFM Config Folder", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "document-open-folder");
    game_selected_menu_actions->readSettings();

    // Special Stuff Menu actions.
    KActionCollection* special_stuff_menu_actions = new KActionCollection(parent, "special_stuff_menu");
    special_stuff_menu_actions->setComponentDisplayName("Special Stuff Menu");
    new_action(special_stuff_menu_actions, "generate_dependencies_cache", "Generate Dependencies Cache", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "database-index");
    new_action(special_stuff_menu_actions, "optimize_pack", "Optimize Pack", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "games-highscores");
    new_action(special_stuff_menu_actions, "patch_siege_ai", "Patch SiegeAI", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "selection-move-to-layer-below");
    new_action(special_stuff_menu_actions, "live_export", "Live Export", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "format-align-vertical-top");
    new_action(special_stuff_menu_actions, "pack_map", "Pack Map", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "folder-add");
    special_stuff_menu_actions->readSettings();

    // About Menu actions.
    KActionCollection* about_menu_actions = new KActionCollection(parent, "about_menu");
    about_menu_actions->setComponentDisplayName("About Menu");
    new_action(about_menu_actions, "about_qt", "About Qt", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "help-about-symbolic");
    new_action(about_menu_actions, "about_rpfm", "About RPFM", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "help-about-symbolic");
    new_action(about_menu_actions, "open_manual", "Open Manual", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+H"), "help-about-symbolic");
    new_action(about_menu_actions, "support_me_on_patreon", "Support Me On Patreon", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "help-donate-eur");
    new_action(about_menu_actions, "check_updates", "Check Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+U"), "svn-update");
    new_action(about_menu_actions, "check_schema_updates", "Check Schema Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+U"), "svn-update");
    new_action(about_menu_actions, "check_message_updates", "Check Message Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "svn-update");
    new_action(about_menu_actions, "check_tw_autogen_updates", "Check TW Autogen Updates", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "svn-update");
    about_menu_actions->readSettings();

    // File Tab actions.
    KActionCollection* file_tab_actions = new KActionCollection(parent, "file_tab");
    file_tab_actions->setComponentDisplayName("File Tabs");
    new_action(file_tab_actions, "close_tab", "Close Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+W"), "project-development-close");
    new_action(file_tab_actions, "close_other_tabs", "Close All Tabs", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "view-close");
    new_action(file_tab_actions, "close_other_tabs_left", "Close All Tabs to the Left", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "view-left-close");
    new_action(file_tab_actions, "close_other_tabs_right", "Close All Tabs to the Right", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "view-right-close");
    new_action(file_tab_actions, "previus_tab", "Previous Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Shift+Tab"), "go-previous-symbolic");
    new_action(file_tab_actions, "next_tab", "Next Tab", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString("Ctrl+Tab"), "go-previous-symbolic-rtl");
    new_action(file_tab_actions, "import_from_dependencies", "Import From Dependencies", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "document-import-ocal");
    new_action(file_tab_actions, "toggle_quick_notes", "Toggle Quick Notes", Qt::ShortcutContext::ApplicationShortcut, QKeySequence::listFromString(""), "visibility");
    file_tab_actions->readSettings();

    // Pack Tree Context actions.
    KActionCollection* pack_tree_actions = new KActionCollection(parent, "pack_tree_context_menu");
    pack_tree_actions->setComponentDisplayName("Pack Tree Context Menu");

    new_action(pack_tree_actions, "add_file", "Add File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+A"), "document-new-symbolic");
    new_action(pack_tree_actions, "add_folder", "Add Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+A"), "tab-new-symbolic");
    new_action(pack_tree_actions, "add_from_pack", "Add From Pack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Alt+A"), "labplot-workbook-new");
    new_action(pack_tree_actions, "new_folder", "New Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+F"), "albumfolder-new");
    new_action(pack_tree_actions, "new_animpack", "New AnimPack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "document-new");
    new_action(pack_tree_actions, "new_db", "New DB", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"), "document-new");
    new_action(pack_tree_actions, "new_loc", "New Loc", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+L"), "document-new");
    new_action(pack_tree_actions, "new_portrait_settings", "New PortraitSettings", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "document-new");
    new_action(pack_tree_actions, "new_text", "New Text", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+T"), "document-new");
    new_action(pack_tree_actions, "new_quick_file", "New Quick File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Q"), "document-new");
    new_action(pack_tree_actions, "merge_files", "Merge Files", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+M"), "merge");
    new_action(pack_tree_actions, "update_files", "Update Tables", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "edit-text-frame-update");
    new_action(pack_tree_actions, "generate_missing_loc_data", "Generate Missing Loc Data", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "languages");
    new_action(pack_tree_actions, "delete", "Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"), "entry-delete");
    new_action(pack_tree_actions, "extract", "Extract", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+E"), "archive-extract");
    new_action(pack_tree_actions, "rename", "Rename", Qt::ShortcutContext::WidgetShortcut, {QKeySequence("Ctrl+R"), QKeySequence("F2")}, "edit-move");
    new_action(pack_tree_actions, "copy_path", "Copy Path", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "edit-copy-path");
    new_action(pack_tree_actions, "open_in_decoder", "Open In Decoder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+J"), "document-edit-decrypt");
    new_action(pack_tree_actions, "open_dependency_manager", "Open Dependency Manager", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "dblatex");
    new_action(pack_tree_actions, "open_in_external_program", "Open In External Program", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+K"), "quickopen-function");
    new_action(pack_tree_actions, "open_containing_folder", "Open Containing Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "document-open");
    new_action(pack_tree_actions, "open_pack_settings", "Open Pack Settings", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "settings-configure");
    new_action(pack_tree_actions, "open_pack_notes", "Open Pack Notes", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Y"), "view-pim-notes");
    new_action(pack_tree_actions, "expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"), "expand-all-symbolic");
    new_action(pack_tree_actions, "collapse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"), "collapse-all-symbolic");
    pack_tree_actions->readSettings();

    // Dependencies Tree Context actions.
    KActionCollection* dependencies_tree_actions = new KActionCollection(parent, "dependencies_context_menu");
    dependencies_tree_actions->setComponentDisplayName("Dependencies Tree Context Menu");
    new_action(dependencies_tree_actions, "copy_path", "Copy Path", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "edit-copy-path");
    new_action(dependencies_tree_actions, "expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"), "expand-all-symbolic");
    new_action(dependencies_tree_actions, "collapsse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"), "collapse-all-symbolic");
    new_action(dependencies_tree_actions, "import_from_dependencies", "Import From Dependencies", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "document-import-ocal");
    new_action(dependencies_tree_actions, "extract_from_dependencies", "Extract From Dependencies", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "archive-extract");
    dependencies_tree_actions->readSettings();

    // Diagnostics Table Context actions.
    KActionCollection* diagnostics_table_actions = new KActionCollection(parent, "diagnostics_context_menu");
    diagnostics_table_actions->setComponentDisplayName("Diagnostics Table Context Menu");
    new_action(diagnostics_table_actions, "ignore_parent_folder", "Ignore Parent Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_parent_folder_field", "Ignore Field for Parent Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_file", "Ignore File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_file_field", "Ignore Field for File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_diagnostic_for_parent_folder", "Ignore Diagnostic for Parent Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_diagnostic_for_parent_folder_field", "Ignore Diagnostic in Field for Parent Folder", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_diagnostic_for_file", "Ignore Diagnostic for File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_diagnostic_for_file_field", "Ignore Diagnostic in Field for File", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    new_action(diagnostics_table_actions, "ignore_diagnostic_for_pack", "Ignore Diagnostic for Pack", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "mail-thread-ignored");
    diagnostics_table_actions->readSettings();

    // AnimPack Tree Context actions.
    KActionCollection* anim_pack_tree_actions = new KActionCollection(parent, "anim_pack_tree_context_menu");
    anim_pack_tree_actions->setComponentDisplayName("AnimPack Tree Context Menu");
    new_action(anim_pack_tree_actions, "delete", "Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"), "entry-delete");
    new_action(anim_pack_tree_actions, "expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"), "expand-all-symbolic");
    new_action(anim_pack_tree_actions, "pack_expand_all", "Expand All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"), "expand-all-symbolic");
    new_action(anim_pack_tree_actions, "collapse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"), "collapse-all-symbolic");
    new_action(anim_pack_tree_actions, "pack_collapse_all", "Collapse All", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"), "collapse-all-symbolic");
    anim_pack_tree_actions->readSettings();

    // AnimPack Pack Tree Context actions.
    KActionCollection* secondary_pack_tree_actions = new KActionCollection(parent, "secondary_pack_tree_context_menu");
    secondary_pack_tree_actions->setComponentDisplayName("Pack Tree Context Menu");
    new_action(secondary_pack_tree_actions, "expand", "Expand", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl++"), "expand-all-symbolic");
    new_action(secondary_pack_tree_actions, "collapse", "Collapse", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"), "collapse-all-symbolic");
    secondary_pack_tree_actions->readSettings();

    // Table Editor actions.
    KActionCollection* table_editor_actions = new KActionCollection(parent, "table_editor");
    table_editor_actions->setComponentDisplayName("Table Editor");
    new_action(table_editor_actions, "add_row", "Add Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+A"), "edit-table-insert-row-below");
    new_action(table_editor_actions, "insert_row", "Insert Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+I"), "insert-table-row");
    new_action(table_editor_actions, "delete_row", "Delete Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"), "edit-table-delete-row");
    new_action(table_editor_actions, "delete_filtered_out_row", "Delete Filtered Out Rows", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+Del"), "edit-table-delete-row");
    new_action(table_editor_actions, "clone_and_insert_row", "Clone And Insert Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"), "insert-table-row");
    new_action(table_editor_actions, "clone_and_append_row", "Clone And Append Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+D"), "edit-table-insert-row-below");
    new_action(table_editor_actions, "copy", "Copy", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+C"), "edit-copy-symbolic");
    new_action(table_editor_actions, "copy_as_lua_table", "Copy as LUA Table", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+C"), "edit-copy-symbolic");
    new_action(table_editor_actions, "copy_as_filter_value", "Copy to Filter Value", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "edit-copy-symbolic");
    new_action(table_editor_actions, "paste", "Paste", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+V"), "edit-paste-symbolic");
    new_action(table_editor_actions, "paste_as_new_row", "Paste as New Row", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+V"), "edit-paste-symbolic");
    new_action(table_editor_actions, "rewrite_selection", "Rewrite Selection", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Y"), "layer-rename");
    new_action(table_editor_actions, "invert_selection", "Invert Selection", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+-"), "edit-select-invert");
    new_action(table_editor_actions, "generate_ids", "Generate IDs", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "gtk-index");
    new_action(table_editor_actions, "reset_selected_values", "Reset Selected Values", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "edit-select-original");
    new_action(table_editor_actions, "import_tsv", "Import TSV", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "albumfolder-importimages");
    new_action(table_editor_actions, "export_tsv", "Export TSV", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "export-symbolic");
    new_action(table_editor_actions, "search", "Search", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+F"), "folder-saved-search-symbolic");
    new_action(table_editor_actions, "sidebar", "Sidebar", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "view-right-new");
    new_action(table_editor_actions, "create_profile", "New Profile", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "go-jump-definition");
    new_action(table_editor_actions, "undo", "Undo", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Z"), "edit-undo-symbolic");
    new_action(table_editor_actions, "redo", "Redo", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Shift+Z"), "edit-redo-symbolic");
    new_action(table_editor_actions, "smart_delete", "Smart Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Del"), "edit-delete-shred");
    new_action(table_editor_actions, "resize_columns", "Resize Columns", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "resizecol");
    new_action(table_editor_actions, "rename_references", "Rename References", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "tool_references");
    new_action(table_editor_actions, "patch_columns", "Patch Columns", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "edit-table-insert-column-right");
    new_action(table_editor_actions, "find_references", "Find References", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "folder-saved-search-symbolic");
    new_action(table_editor_actions, "go_to_definition", "Go To Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString(""), "go-jump-definition");
    table_editor_actions->readSettings();

    // Decoder actions.
    KActionCollection* decoder_actions = new KActionCollection(parent, "decoder");
    decoder_actions->setComponentDisplayName("Decoder");
    new_action(decoder_actions, "move_field_up", "Move Field Up", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Up"), "go-up");
    new_action(decoder_actions, "move_field_down", "Move Field Down", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Down"), "go-down");
    new_action(decoder_actions, "move_field_left", "Move Field Left", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Left"), "arrow-left");
    new_action(decoder_actions, "move_field_right", "Move Field Right", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Right"), "arrow-right");
    new_action(decoder_actions, "delete_field", "Delete Field", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"), "entry-delete");
    new_action(decoder_actions, "delete_definition", "Delete Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"), "entry-delete");
    new_action(decoder_actions, "load_definition", "Load Definition", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+L"), "kt-set-max-upload-speed");
    decoder_actions->readSettings();

    KActionCollection* portrait_settings_actions = new KActionCollection(parent, "portrait_settings");
    portrait_settings_actions->setComponentDisplayName("Portrait Settings");
    new_action(portrait_settings_actions, "add", "Add", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+A"), "edit-table-insert-row-below");
    new_action(portrait_settings_actions, "clone", "Clone", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+D"), "insert-table-row");
    new_action(portrait_settings_actions, "delete", "Delete", Qt::ShortcutContext::WidgetShortcut, QKeySequence::listFromString("Ctrl+Del"), "edit-table-delete-row");
    portrait_settings_actions->readSettings();

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
    shortcuts->append(dynamic_cast<QObject*>(diagnostics_table_actions));
    shortcuts->append(dynamic_cast<QObject*>(anim_pack_tree_actions));
    shortcuts->append(dynamic_cast<QObject*>(secondary_pack_tree_actions));
    shortcuts->append(dynamic_cast<QObject*>(table_editor_actions));
    shortcuts->append(dynamic_cast<QObject*>(decoder_actions));
    shortcuts->append(dynamic_cast<QObject*>(portrait_settings_actions));
    shortcuts->append(dynamic_cast<QObject*>(text_editor_actions));
}

extern "C" QAction* shortcut_action(QList<QObject*> const &shortcuts, QString const action_group, QString const action_name) {
    foreach (QObject* collection, shortcuts) {
        KActionCollection* actions = dynamic_cast<KActionCollection*>(collection);

        if (actions->componentName() == action_group) {
            QAction* action = actions->action(action_name);
            if (action != nullptr) {

                // Create a new action as a copy instead of returning it to avoid issues with duplicated triggers.
                QAction* new_action = new QAction();
                new_action->setText(action->text());
                new_action->setIcon(action->icon());
                new_action->setShortcuts(action->shortcuts());
                new_action->setShortcutContext(action->shortcutContext());
                return new_action;
            }
        }
    }
    return nullptr;
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

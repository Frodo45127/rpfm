### Localization for RPFM-UI - Chinese
## Translator : 黄俊浩
## General Localization

gen_loc_accept = 接受
gen_loc_create = 创建
gen_loc_packedfile = Packed 文件
gen_loc_packfile = Pack 文件
gen_loc_packfile_contents = Pack 文件内容

gen_loc_column = 列
gen_loc_row = 行
gen_loc_match = 匹配
gen_loc_length = 长度

trololol = queek_headtaker_yes_yes

### mod.rs localization

## Menu Bar

menu_bar_packfile = &Pack文件
menu_bar_view = &查看
menu_bar_mymod = &我的MOD
menu_bar_game_selected = &游戏选择
menu_bar_special_stuff = &特殊选项
menu_bar_about = &关于
menu_bar_debug = &调试

## PackFile Menu

new_packfile = &新 Pack 文件
open_packfile = &打开 Pack 文件
save_packfile = &保存 Pack 文件
save_packfile_as = 保存 Pack 文件&为...
load_all_ca_packfiles = &载入所有 CA Pack 文件
preferences = &偏好设置
quit = &退出
open_from_content = 从MOD目录打开
open_from_data = 从Data目录打开
change_packfile_type = &更改 Pack 文件类型

## Change Packfile Type Menu

packfile_type_boot = &Boot
packfile_type_release = &Release
packfile_type_patch = &Patch
packfile_type_mod = &Mod
packfile_type_movie = Mo&vie
packfile_type_other = &Other

change_packfile_type_header_is_extended = &Header Is Extended
change_packfile_type_index_includes_timestamp = &Index Includes Timestamp
change_packfile_type_index_is_encrypted = Index Is &Encrypted
change_packfile_type_data_is_encrypted = &Data Is Encrypted
change_packfile_type_data_is_compressed = Data Is &Compressed

## MyMod Menu

mymod_new = &创建新的Mod
mymod_delete_selected = &删除选定的Mod
mymod_install = &安装
mymod_uninstall = &卸载

mymod_name = Mod名称:
mymod_name_default = 举例: one_ring_for_me
mymod_game = Game of the Mod:

## View Menu

view_toggle_packfile_contents = 开关 &PackFile 显示框
view_toggle_global_search_panel = 开关全局搜索框

## Game Selected Menu

game_selected_launch_game = 打开选定的游戏
game_selected_open_game_data_folder = 打开游戏Data目录
game_selected_open_game_assembly_kit_folder = 打开游戏 Assembly Kit 目录
game_selected_open_config_folder = 打开 RPFM 配置目录

## Special Stuff

special_stuff_optimize_packfile = &优化 Pack 文件
special_stuff_generate_pak_file = &生成 PAK 文件
special_stuff_patch_siege_ai = &修复 Siege AI
special_stuff_select_ak_folder = 选择 Assembly Kit 目录
special_stuff_select_raw_db_folder = 选择 Raw DB 目录

## About Menu

about_about_qt = 关于 &Qt
about_about_rpfm = 关于 RPFM
about_open_manual = &打开手册
about_patreon_link = &在 Patreon 上支持我
about_check_updates = &检查更新
about_check_schema_updates = 检查 Schema &更新

## Debug Menu

update_current_schema_from_asskit = Update currently loaded Schema with Assembly Kit
generate_schema_diff = Generate Schema Diff

### app_ui_extra.rs localisation

## Update Stuff

update_checker = 更新检查
update_schema_checker = Schema更新检查
update_searching = 检查更新中...
update_button = &更新
update_in_prog = <p>正在下载更新,请不要关闭此窗口...</p> <p>可能还要再等一下.</p>
update_no_local_schema = <p>没有找到本地语言文件,您是否要下载最新的语言文件?</p><p><b>注意:</b> 打开表需要Schemas文件, locs 和其他 PackedFiles. 没有 schemas 意味着你无法编辑任何表.</p>

## Folder Dialogues

new_folder_default = new_folder
new_folder = 新建文件夹

## PackedFile Dialogues

new_file_default = new_file
new_db_file = 新 DB 文件
new_loc_file = 新 Loc 文件
new_txt_file = 新 Text 文件
new_packedfile_name = 新 PackedFile 的名称

packedfile_filter = 在此输入内容来筛选列表中的表格. 可以使用正则表达式!

merge_tables = 合并表
merge_tables_new_name = 在此输入新文件的名称.
merge_tables_delete_option = 删除原始表

## External FileDialog

open_packfiles = 打开 Pack 文件

### tips.rs

## PackFile menu tips

tt_packfile_new_packfile = 创建一个新的 Pack 文件并打开. 如果要保留文件,记得退出前保存!
tt_packfile_open_packfile = 打开一个现有的 Pack 文件, 或将多个 Pack 文件合并为一个.
tt_packfile_save_packfile = 保存对当前打开的 Pack 文件的所有更改.
tt_packfile_save_packfile_as = 将当前打开的 Pack 文件保存为一个新文件,而不是覆盖原文件.
tt_packfile_load_all_ca_packfiles = 尝试使用延迟加载打开所有的CA原始Pack文件,并同时加载到RPFM.请注意,如果尝试保存,则你的电脑可能会死机.
tt_packfile_preferences = 打开设置界面.
tt_packfile_quit = 退出程序.

tt_change_packfile_type_boot = 更改 Pack 文件类型为 Boot. 永远不要使用该功能.
tt_change_packfile_type_release = 更改 Pack 文件类型为 Release. 永远不要使用该功能.
tt_change_packfile_type_patch = 更改 Pack 文件类型为 Patch. 永远不要使用该功能.
tt_change_packfile_type_mod = 更改 Pack 文件类型为 Mod. 适用于更改为会显示在Mod管理器中的Mod.
tt_change_packfile_type_movie = 更改 Pack 文件类型为 Movie. 适用于更改为会持续激活,且不会显示在Mod管理器中的Mod.
tt_change_packfile_type_other = 更改 Pack 文件类型为 Other. 针对不具有写功能的 Pack 文件, 永远不要使用该功能.

tt_change_packfile_type_data_is_encrypted = 如果选中,则此 Pack 文件将被加密.启用该功能的 Pack 文件将不能保存.
tt_change_packfile_type_index_includes_timestamp = 如果选中,则此 Pack 文件的索引将包含每个 Packed 文件的'最后修改日期'.请注意,启用该功能的 Pack 文件将不会在官方启动器中显示为Mod.
tt_change_packfile_type_index_is_encrypted = 如果选中,则此 Pack 文件的索引将被加密.启用该功能的 Pack 文件将不能保存.
tt_change_packfile_type_header_is_extended = 如果选中,则此 Pack 文件的文件头将拓展20个字节.启用该功能的 Pack 文件将不能保存.
tt_change_packfile_type_data_is_compressed = 如果选中,则此 Pack 文件的每个 Packed 文件将在保存时被压缩.如果要解压缩 Pack 文件,请禁用该功能然后保存它.

## MyMod menu tips

tt_mymod_new = 打开一个创建新 Mod 的对话框.
tt_mymod_delete_selected = 删除当前选择的 Mod.
tt_mymod_install = 复制当前 Mod 到所选游戏的 Data 目录.
tt_mymod_uninstall = 从所选游戏的 Data 目录删除所选的 Mod.

## GameSelected menu tips

tt_game_selected_launch_game = 尝试从Steam启动当前选中的游戏.
tt_game_selected_open_game_data_folder = 尝试在文件管理器中打开所选游戏的Data目录(如果存在).
tt_game_selected_open_game_assembly_kit_folder = 尝试在文件管理器中打开所选游戏的Assembly Kit目录(如果存在).
tt_game_selected_open_config_folder = 打开RPFM配置目录, 即 config/schemas/ctd 所在目录.

tt_game_selected_three_kingdoms = 设置 'TW:Three Kingdoms' 为'选中游戏'.
tt_game_selected_warhammer_2 = 设置 'TW:Warhammer 2' 为'选中游戏'.
tt_game_selected_warhammer = 设置 'TW:Warhammer' 为'选中游戏'.
tt_game_selected_thrones_of_britannia = 设置 'TW: Thrones of Britannia' 为'选中游戏'.
tt_game_selected_attila = 设置 'TW:Attila' 为'选中游戏'.
tt_game_selected_rome_2 = 设置 'TW:Rome 2' 为'选中游戏'.
tt_game_selected_shogun_2 = 设置 'TW:Shogun 2' 为'选中游戏'.
tt_game_selected_napoleon = 设置 'TW:Napoleon' 为'选中游戏'.
tt_game_selected_empire = 设置 'TW:Empire' 为'选中游戏'.
tt_game_selected_arena = 设置 'TW:Arena' 为'选中游戏'.

## Special Stuff menu tips

tt_generate_pak_file = 为所选游戏创建一个PAK文件 (Processed Assembly Kit File), 以帮助进行依赖性检查.
tt_optimize_packfile = 检查并删除所有Db,Loc文件中与基本游戏中相同的内容 (Loc 文件仅适用于英语). 避免与其他Mod的兼容性问题.
tt_patch_siege_ai = 修复并清理Pack文件中导出的地图数据.修复了Siege AI (如果有的话) 并删除了使Pack文件大小膨胀的无用Xml文件,从而减小了文件体积.

## About menu tips

tt_about_about_qt = 有关 Qt 的信息, 制作此程序的Ui程序包.
tt_about_about_rpfm = 有关RPFM的信息.
tt_about_open_manual = 在PDF阅读器中打开RPFM手册.
tt_about_patreon_link = 打开RPFM的Patreon页面.即使您不愿成为赞助人,也请查看.我会不时发布有关下一次更新和开发中功能的信息.
tt_about_check_updates = 检查是否有RPFM的可用更新.
tt_about_check_schema_updates = 检查是否有Schemas的可用更新.这是在游戏发布更新补丁后必须更新的内容.

### global_search_ui/mod.rs

global_search = 全局搜索
global_search_info = 搜索信息
global_search_search = 搜索
global_search_replace = 替换
global_search_replace_all = 替换全部
global_search_clear = 清除
global_search_case_sensitive = 区分大小写
global_search_use_regex = 使用正则表达式
global_search_search_on = 在其中搜索

global_search_all = 全部
global_search_db = DB
global_search_loc = LOC
global_search_txt = Text
global_search_schemas = Schemas

## Filter Dialogues

global_search_db_matches = DB 匹配
global_search_loc_matches = Loc 匹配
global_search_txt_matches = Text 匹配
global_search_schema_matches = Schema 匹配

global_search_match_packedfile_column = PackedFile/列
global_search_match_packedfile_text = PackedFile/文本

global_search_versioned_file = VersionFiled (Type, Name)/Column Name
global_search_definition_version = Definition Version
global_search_column_index = Column Index

## tips

tt_global_search_use_regex_checkbox = 是否启用正则表达式搜索.如果填写的表达式无效,则会使用普通搜索模式.
tt_global_search_case_sensitive_checkbox = 是否启用区分大小写的搜索.效果如字面描述.
tt_global_search_search_on_all_checkbox = 在搜索中包含所有可用的 PackedFiles/Schemas.
tt_global_search_search_on_dbs_checkbox = 在搜索中包含 DB.
tt_global_search_search_on_locs_checkbox = 在搜索中包含 LOC.
tt_global_search_search_on_texts_checkbox = 在搜索中包含任何 Text 类型文件.
tt_global_search_search_on_schemas_checkbox = 在搜索中包含当前加载的 Schema.

### Open PackedFile Dialog

open_packedfile_dialog_1 = 你确定吗?
open_packedfile_dialog_2 = 你将要替换/删除的一个或多个 Pack 文件已被打开.你确定要这么做吗?点击是将关闭它.

### TreeView Text/Filter

treeview_aai = 大小写区分
treeview_autoexpand = 自动展开匹配结果
treeview_expand_all = &展开全部
treeview_collapse_all = &收起全部

### TreeView Tips

tt_context_menu_add_file = 将一个或多个文件添加到当前Pack文件.现有文件不会被覆盖!
tt_context_menu_add_folder = 将一个文件夹添加到当前Pack文件.现有文件不会被覆盖!
tt_context_menu_add_from_packfile = 将一个文件从另一个Pack文件中添加到当前打开的Pack文件.现有文件不会被覆盖!
tt_context_menu_check_tables = 检查当前打开的Pack文件是否存在依赖项错误.
tt_context_menu_new_folder = 打开对话框以创建一个空文件夹.由于Pack文件的保存方式,如果该文件在保存时为空,则不会保留.
tt_context_menu_new_packed_file_db = 打开对话框以创建一个DB数据表文件(适用于游戏中大多数数据).
tt_context_menu_new_packed_file_loc = 打开对话框以创建一个Loc文件(游戏使用该文件存储显示的文本).
tt_context_menu_new_packed_file_text = 打开对话框以创建一个纯文本文件.它允许使用不同的拓展名,例如'Xml','Lua','Txt'....
tt_context_menu_new_queek_packed_file = 打开对话框,根据当前所选位置创建一个Packed文件.例如,如果你选中 /text,则会创建一个Loc文件.
tt_context_menu_mass_import_tsv = 同时导入一堆TSV文件.RPFM会自动检查这些文件是否是DB数据表文件,然后将它们一次性导入.现有文件将会被覆盖!
tt_context_menu_mass_export_tsv = 将当前Pack文件中的所有DB数据表文件和Loc文件导出为TSV文件.现有文件将会被覆盖!
tt_context_menu_merge_tables = 将多个DB/LOC文件合并为一个.
tt_context_menu_update_tables = 将表格更新为'当前所选游戏'的最后一个已知工作版本.
tt_context_menu_delete = 删除选中的文件/文件夹.

tt_context_menu_extract = 从Pack文件中导出所选的文件/文件夹.
tt_context_menu_rename = 重命名所选的文件/文件夹.请记住不要使用空格,且不要重名.
tt_context_menu_open_decoder = 在解码器中打开所选的DB文件.创建/更新Schemas.
tt_context_menu_open_dependency_manager = 打开该Pack文件所引用的Pack文件.
tt_context_menu_open_containing_folder = 在文件管理器中打开当前Pack文件的目录.
tt_context_menu_open_with_external_program = 在外部程序中打开Packed文件.
tt_context_menu_open_notes = 在辅助视图中打开Pack文件的注释,而无需在主视图中关闭当前打开的Pack文件.
tt_filter_autoexpand_matches_button = 自动展开匹配结果.注意:如果存在较多的匹配结果(大于10K,例如Data.pack),则会使程序挂起一会儿.
tt_filter_case_sensitive_button = 为树形图启用/禁用大小写区分的筛选.

packedfile_noneditable_sequence = 不可编辑的序列

### Rename Dialogues

rename_selection = 重命名 Selection
rename_selection_instructions = 使用说明
rename_selection_placeholder = 随便写些什么, {"{"}x{"}"} 是你的当前名称.

### Mass-Import

mass_import_tsv = 批量导入TSV文件.
mass_import_num_to_import = 导入文件: 0.
mass_import_use_original_filename = 使用原始文件名:
mass_import_import = 导入
mass_import_default_name = new_imported_file

mass_import_select = 请选择要导入的TSV文件...

files_to_import = 导入文件: {"{"}{"}"}.

### Table

decoder_title = PackedFile 解码器
table_dependency_manager_title = 依赖项管理器
table_filter_case_sensitive = 区分大小写
table_enable_lookups = 使用查找

### Contextual Menu for TreeView

context_menu_add = &添加...
context_menu_create = &创建...
context_menu_open = &打开...

context_menu_add_file = &添加文件
context_menu_add_files = 添加文件
context_menu_add_folder = 添加&目录
context_menu_add_folders = 添加目录
context_menu_add_from_packfile = 从 &Pack 文件中添加
context_menu_select_packfile = 选择 Pack 文件
context_menu_extract_packfile = 导出 Pack 文件

context_menu_new_folder = &创建文件夹
context_menu_new_packed_file_db = 创建 &DB
context_menu_new_packed_file_loc = &创建 Loc
context_menu_new_packed_file_text = 创建 &Text
context_menu_new_queek_packed_file = 新 Queek 文件

context_menu_mass_import_tsv = 批量导入TSV文件
context_menu_mass_export_tsv = 批量导出TSV文件
context_menu_mass_export_tsv_folder = 选择目标文件夹
context_menu_rename = &重命名
context_menu_delete = &删除
context_menu_extract = &导出

context_menu_open_decoder = &在解码器中打开
context_menu_open_dependency_manager = 打开&依赖项管理器
context_menu_open_containing_folder = 打开&包含文件夹
context_menu_open_with_external_program = 用&外部程序打开
context_menu_open_notes = 打开&提示

context_menu_check_tables = &检查表
context_menu_merge_tables = &合并表
context_menu_update_table = &更新表

### Shortcuts

menu_bar_packfile_section = Pack 文件菜单
menu_bar_mymod_section = 我的MOD菜单
menu_bar_view_section = 查看菜单
menu_bar_game_selected_section = 游戏选择菜单
menu_bar_about_section = 关于菜单
packfile_contents_tree_view_section = Pack 文件相关内容菜单
packed_file_table_section = Packed 文件相关内容菜单
packed_file_decoder_section = Packed 文件解码器

shortcut_esc = Esc
shortcut_csp = Ctrl+Shift+P

shortcut_title = 快捷键
shortcut_text = 快捷键
shortcut_section_action = Section/Action

### Settings

settings_title = 设置

settings_paths_title = 路径
settings_paths_mymod = 我的MOD文件夹
settings_paths_mymod_ph = 这是你要保存所有 \"我的MOD\" 相关文件的目录.

settings_game_label = TW: {"{"}{"}"} 文件夹
settings_game_line_ph = 这是你安装 {"{"}{"}"} 的所在目录, 游戏启动.exe文件所在的目录.

settings_ui_title = UI设置
settings_table_title = 表设置

settings_ui_language = 语言切换 (需要重新启动):
settings_ui_dark_theme = 使用暗色主题 (需要重新启动):
settings_ui_table_adjust_columns_to_content = 自动调整列宽:
settings_ui_table_disable_combos = 禁用表上的组合框:
settings_ui_table_extend_last_column_label = 拓展表格的最后一列:
settings_ui_table_tight_table_mode_label = 在表格中启用'紧密模式':
settings_ui_table_remember_column_visual_order_label = 保留列的显示顺序:
settings_ui_table_remember_table_state_permanently_label = 保留跨Pack文件的表状态:
settings_ui_window_start_maximized_label = 启动时最大化:
settings_ui_window_hide_background_icon = 隐藏选中游戏的图标:

settings_select_folder = 选择文件夹

settings_extra_title = 额外设置
settings_default_game = 默认选中游戏:
settings_check_updates_on_start = 启动时自动检查更新:
settings_check_schema_updates_on_start = 启动时自动检查Schema更新:
settings_allow_editing_of_ca_packfiles = 允许编辑CA Pack文件:
settings_optimize_not_renamed_packedfiles = 优化未重命名的Pack文件:
settings_use_dependency_checker = 为DB表启用依赖性检查器:
settings_use_lazy_loading = 为Pack文件启用延迟加载:

settings_debug_title = 调试设置
settings_debug_missing_table = 检查缺少的表定义
settings_debug_enable_debug_menu = 启用调试菜单

settings_text_title = 文本编辑器设置

### Settings Tips

tt_ui_global_use_dark_theme_tip = <i>是否启用暗色主题</i>
tt_ui_table_adjust_columns_to_content_tip = 如果启用,则在打开DB/Loc文件时,所有列将根据内容大小调整宽度.
    否则,列将会有一个预定义宽度.无论使用哪种方式,你都可以在打开后重新调整其宽度.
    注意:这将会使打开非常大的文件时花费更多时间来加载.
tt_ui_table_disable_combos_tip = 如果禁用,则编辑表中的任何引用列时将不会显示任何自动补全.
    Now shut up Baldy.
tt_ui_table_extend_last_column_tip = 如果启用,则DB/Loc文件中的最后一列将自动拓展,以填充右侧的空白处(如果有).
tt_ui_table_tight_table_mode_tip = 如果启用,表中垂直的无用空间将减少,因此你可以同时看到更多数据.
tt_ui_table_remember_column_visual_order_tip = 如果启用,在RPFM关闭并重新打开DB/Loc文件时会记住列的视觉顺序.
tt_ui_table_remember_table_state_permanently_tip = 如果启用,在RPFM重新启动时,RPFM也会记住DB/Loc文件的状态(搜索筛选项,移动过的列,正在排序的列...).如果你不希望出现这种情况,请禁用此功能.
tt_ui_window_start_maximized_tip = 如果启用,RPFM将在启动时最大化显示.


tt_extra_network_check_updates_on_start_tip = 如果启用,RPFM将在启动时自动检查是否有可用更新并通知你,是否下载取决于你.
tt_extra_network_check_schema_updates_on_start_tip = 如果启用,RPFM将在启动时自动检查Schema是否有可用更新并自动下载.
tt_extra_packfile_allow_editing_of_ca_packfiles_tip = 默认情况下,只有类型为'Mod'和'Movie'的Pack文件是可编辑的,如果启用此选项则可以编辑其他类型的Pack文件,请注意不要覆盖游戏的原始Pack文件!
tt_extra_packfile_optimize_not_renamed_packedfiles_tip = 如果启用,则在使用'优化Pack文件'功能时,RPFM将优化与基本游戏中重名的DB/Loc文件.
    通常情况下,这些文件的会覆盖基本游戏中的同名文件(因此此功能默认为禁用).但有些时候也会对其进行优化(AssKit包含太多文件),因此也有其用处,这就是保留该设置的原因.
tt_extra_packfile_use_dependency_checker_tip = 如果启用,则在打开DB文件时,RPFM会尝试获取其依赖关系并将所有对另一个表的引用情况标记为'未找到(红色)','未找到参考表(蓝色)'或'正确(黑色)'.同时会使打开大型表的速度变慢.
tt_extra_packfile_use_lazy_loading_tip = 如果启用,则Pack将从硬盘上按需加载数据而不是一次性加载所有数据到内存中.这样可以大大减少内存占用,但是如果Pack文件在打开时删除或更改了内容,则文件可能会无法恢复,您将会丢失其中的内容.
    如果你的MOD在 Warhammer 2's /data 目录中请禁用此功能, 因为 Assembly Kit 文件过大会导致Pack文件在此功能被开启后损坏或被删除.

tt_debug_check_for_missing_table_definitions_tip = 如果启用,则RPFM会在打开当前Pack文件或更改'选中游戏'时尝试解码每个Pack文件中的每个表,并将它没有Schema的表输出到\"missing_table_definitions.txt\" 文件中.
    调试功能,非常慢.除非你真的想使用它,否则请不要开启.

### CA_VP8 Videos

format = 格式:
version = 版本:
header_len = 标题长度:
codec_four_cc = Codec Four CC:
width = 宽度:
height = 高度:
ms_per_frame = Ms Per Frame:
num_frames = Number of Frames:
largest_frame = Largest Frame:
mystery_number = 我不知道这是什么:
offset_frame_table = Frame Table's Offset:
framerate = Framerate:
timebase = Timebase:
x2 = 我不知道这是什么:

convert_to_camv = 转换为 CAMV
convert_to_ivf = 转换为 IVF

notes = 提示

external_current_path = 当前编辑文件路径:
stop_watching = 停止观看文件
open_folder = 在文件管理器中打开文件夹

game_selected_changed_on_opening = 当前选中游戏已更改为 {"{"}{"}"},因为你开的Pack文件与所选游戏不兼容.

### Extra stuff I don't remember where it goes.

rpfm_title = Rusted PackFile Manager
delete_mymod_0 = <p>You are about to delete this <i>'MyMod'</i> from your disk.</p><p>There is no way to recover it after that.</p><p>Are you sure?</p>
delete_mymod_1 = <p>There are some changes yet to be saved.</p><p>Are you sure?</p>

api_response_success_new_update = "<h4>New major update found: {"{"}{"}"}</h4> <p>Download and changelog available here:<br><a href="{"{"}{"}"}">{"{"}{"}"}</a></p>
api_response_success_new_update_hotfix = <h4>New minor update/hotfix found: {"{"}{"}"}</h4> <p>Download and changelog available here:<br><a href="{"{"}{"}"}">{"{"}{"}"}</a></p>
api_response_success_no_update = <h4>No new updates available</h4> <p>More luck next time :)</p>
api_response_success_unknown_version = <h4>Error while checking new updates</h4> <p>There has been a problem when getting the lastest released version number, or the current version number. That means I fucked up the last release title. If you see this, please report it here:\n<a href=\"https://github.com/Frodo45127/rpfm/issues\">https://github.com/Frodo45127/rpfm/issues</a></p>
api_response_error = <h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>

api_response_schema_success_no_update = <h4>No new schema updates available</h4> <p>More luck next time :)</p>
api_response_schema_error = <h4>Error while checking new updates :(</h4> <p>If you see this message, there has been a problem with your connection to the Github.com server. Please, make sure you can access to <a href=\"https://api.github.com\">https://api.github.com</a> and try again.</p>

schema_update_0 = <h4>New schema update available</h4> <table>
schema_update_1 = <p>Do you want to update the schemas?</p>
schema_update_success = <h4>Schemas updated and reloaded</h4><p>You can continue using RPFM now.</p>

files_extracted_success = {"{"}{"}"} files extracted. No errors detected.
mymod_delete_success = MyMod successfully deleted: \"{"{"}{"}"}\"

generate_pak_success = PAK File succesfully created and reloaded.
game_selected_unsupported_operation = This operation is not supported for the Game Selected.

optimize_packfile_success = PackFile optimized.
update_current_schema_from_asskit_success = Currently loaded schema updated.
generate_schema_diff_success = Diff generated succesfully.
settings_font_title = 字体设置

title_success = 成功!
title_error = 错误!

rename_instructions = It's easy, but you'll not understand it without an example, so here it's one:
     - Your files/folders says 'you' and 'I'.
     - Write 'whatever {"{"}x{"}"} want' in the box below.
     - Hit 'Accept'.
     - RPFM will turn that into 'whatever you want' and 'whatever I want' and call your files/folders that.
    And, in case you ask, works with numeric cells too, as long as the resulting text is a valid number.

update_table_success = Table updated from version '{"{"}{"}"}' to version '{"{"}{"}"}'.
no_errors_detected = No errors detected.
original_data = Original Data: '{"{"}{"}"}'
column_tooltip_1 = This column is a reference to:
column_tooltip_2 = And many more. Exactly, {"{"}{"}"} more. Too many to show them here.
column_tooltip_3 = Fields that reference this column:

tsv_select_title = Select TSV File to Import...
tsv_export_title = Export TSV File...

rewrite_selection_title = Rewrite Selection
rewrite_selection_instructions_title = Instructions
rewrite_selection_instructions = Legend says:
     - {"{"}x{"}"} means current value.
     - {"{"}y{"}"} means current column.
     - {"{"}z{"}"} means current row.
rewrite_selection_is_math = 是一个数字运算符?
rewrite_selection_placeholder = 随便写些什么在这里.
rewrite_selection_accept = 接受

context_menu_apply_submenu = &应用...
context_menu_clone_submenu = &克隆...
context_menu_copy_submenu = &复制...
context_menu_add_rows = &添加行
context_menu_insert_rows = &插入行
context_menu_delete_rows = &删除行
context_menu_rewrite_selection = &重写选中项
context_menu_clone_and_insert = &克隆并插入
context_menu_clone_and_append = 克隆并&追加
context_menu_copy = &复制
context_menu_copy_as_lua_table = &复制为 &LUA 表
context_menu_paste = &粘贴
context_menu_search = &搜索
context_menu_sidebar = 侧边栏
context_menu_import_tsv = &导入TSV
context_menu_export_tsv = &导出TSV
context_menu_invert_selection = 反选选中项
context_menu_reset_selection = 重置选中项
context_menu_resize_columns = 调整列宽度
context_menu_undo = &撤销
context_menu_redo = &重置

header_column = <b><i>列名称</i></b>
header_hidden = <b><i>隐藏</i></b>
header_frozen = <b><i>冻结</i></b>

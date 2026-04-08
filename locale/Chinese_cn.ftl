### RPFM-UI 本地化 - 简体中文 (汉化者：热心市民石先生)

## 这两个部分需要为特殊版本进行更改，因此放在最前面。

title_only_for_the_brave_old = 仅供勇者体验
message_only_for_the_brave_old = <p>此版本已被标记为“仅供勇者体验”。这意味着它是一个包含某些极不稳定/未经测试功能的测试版本，可能会给用户带来问题。但…你可以比其他人先体验新功能。</p>

    <p>如果你不想承担风险，请将更新通道改回稳定版 (stable) 并检查更新。这会将你的 RPFM 恢复到最新的稳定版本。另外，你需要进入偏好设置 (Preferences) 并清理依赖 (dependencies) 和架构 (schemas) 文件夹。</p>

    <p>因此，在“仅供勇者体验”版本中，强烈建议你在使用 RPFM 之前备份你的 Mod。以下是此版本中不稳定的功能列表：</p>
    <ul>
        <li>后端重写：整个后端已重写/重构，使其速度更快、更易于维护，并消除了旧代码带来的许多障碍。</li>
        <li>CLI 重写：命令行界面 (CLI) 已从头开始重写，以确保它真正可用，不像旧版那样。</li>
        <li>UI 清理：在让 UI 适配新后端时，大量 UI 代码得到了清理/优化，从而使各处的操作速度整体提升，特别是在打开程序、打开 Pack 文件以及更改所选游戏时。</li>
    </ul>
    <p>目前在此测试版中，你需要注意，某些 UI 部分尚未针对新后端进行更新，可能无法工作或导致程序崩溃。已知损坏的功能包括：</p>
    <ul>
        <li>数据表/重命名引用：仅适用于 DB 条目，不适用于 Loc 条目。</li>
        <li>请记得清理并更新你的架构 (schemas) 并重新生成依赖缓存。</li>
    </ul>
    <p>理论上... 其他一切都应该能正常工作。</p>

title_only_for_the_brave = 重要信息
message_only_for_the_brave = <p>如果你是从 3.X 版本的 RPFM 升级过来的，有一些事情你必须注意：</p>
    <ul>
        <li>启动时提示“未提供 Pack (No Packs Provided)”错误：删除 rpfm.exe，并使用 rpfm_ui.exe 代替。</li>
        <li>数据表无法打开/在搜索中不显示：进入 Pack/设置 (Settings)，点击清除架构文件夹 (Clear Schema Folder)，然后进入关于 (About)/检查架构更新 (Check Schema Updates) 并更新它们。之后重启 RPFM。</li>
        <li>“加载所有 CA Pack 文件 (Load All CA Packs)”无法点击：这是因为你已经生成了依赖缓存，它已经将所有 CA Pack 文件加载到了依赖面板中。当你已经加载了它们时，再使用“加载所有 CA Pack 文件”是没有意义的。</li>
        <li>设置不见了！4.0 使用了不同的设置系统。你需要重新配置它。</li>
        <li>批量导出已合并到添加/提取功能中。如果你想批量导入/导出数据表，只需添加/提取它们，它们就会被即时转换成 tsv 格式或从 tsv 格式转换过来。</li>
    </ul>

## General Localization

gen_loc_accept = 接受
gen_loc_create = 创建
gen_loc_packedfile = 文件
gen_loc_packfile = Pack
gen_loc_packfile_contents = Pack 内容

gen_loc_column = 列
gen_loc_row = 行
gen_loc_match = 匹配
gen_loc_length = 长度

trololol = queek_headtaker_yes_yes

## mod.rs localization

## Menu Bar

menu_bar_packfile = PackFile
menu_bar_view = 视图
menu_bar_mymod = MyMod
menu_bar_game_selected = 当前游戏
menu_bar_special_stuff = 特殊功能
menu_bar_templates = 模板
menu_bar_about = 关于
menu_bar_debug = 调试

## PackFile Menu

new_packfile = 新建 PackFile
open_packfile = 打开 PackFile
save_packfile = 保存 PackFile
save_packfile_as = 另存 PackFile 为…
save_packfile_for_release = 保存 PackFile 以供发布
packfile_install = 安装
packfile_uninstall = 卸载
load_all_ca_packfiles = 加载所有 CA PackFile
settings = 设置
quit = 退出
open_recent = 打开最近文件
open_from_content = 从 Content 目录打开
open_from_data = 从 Data 目录打开
change_packfile_type = 更改 PackFile 类型

## Change Packfile Type Menu

packfile_type_boot = Boot
packfile_type_release = Release
packfile_type_patch = Patch
packfile_type_mod = Mod
packfile_type_movie = Movie
packfile_type_other = Other

change_packfile_type_header_is_extended = 扩展头部
change_packfile_type_index_includes_timestamp = 索引包含时间戳
change_packfile_type_index_is_encrypted = 索引已加密
change_packfile_type_data_is_encrypted = 数据已加密
change_packfile_type_data_is_compressed = 数据已压缩

## MyMod Menu

mymod_new = 新建 MyMod
mymod_delete_selected = 删除选中的 MyMod
mymod_import = 导入
mymod_export = 导出

mymod_name = Mod 名称:
mymod_name_default = 例如: one_ring_for_me
mymod_game = Mod 所属游戏:

## View Menu

view_toggle_packfile_contents = 切换 PackFile 内容面板
view_toggle_global_search_panel = 切换全局搜索窗口
view_toggle_diagnostics_panel = 切换诊断窗口
view_toggle_dependencies_panel = 切换依赖面板

## Game Selected Menu

game_selected_launch_game = 启动当前游戏
game_selected_open_game_data_folder = 打开游戏的 Data 文件夹
game_selected_open_game_assembly_kit_folder = 打开游戏的 Assembly Kit 文件夹
game_selected_open_config_folder = 打开 RPFM 的配置文件夹

## Special Stuff

special_stuff_optimize_packfile = 优化 PackFile
special_stuff_patch_siege_ai = 修复攻城 AI
special_stuff_live_export = 实时导出 (Live Export)
special_stuff_select_ak_folder = 选择 Assembly Kit 文件夹
special_stuff_select_raw_db_folder = 选择 Raw DB 文件夹

## Templates Menu
templates_open_custom_templates_folder = 打开自定义模板文件夹
templates_open_official_templates_folder = 打开官方模板文件夹
templates_save_packfile_to_template = 将 PackFile 保存为模板
templates_load_custom_template_to_packfile = 加载自定义模板至 PackFile
templates_load_official_template_to_packfile = 加载官方模板至 PackFile

## About Menu

about_about_qt = 关于 Qt
about_about_rpfm = 关于 RPFM
about_check_updates = 检查更新
about_check_schema_updates = 检查 Schema (架构) 更新

## Debug Menu

update_current_schema_from_asskit = 使用 Assembly Kit 更新当前加载的 Schema
generate_schema_diff = 生成 Schema 差异分析 (Diff)

## app_ui_extra.rs localisation

## Update Stuff

update_checker = 更新检查器
update_schema_checker = Schema (架构) 更新检查器
update_template_checker = 模板更新检查器
update_searching = 正在搜索更新…
update_button = 更新
update_in_prog = <p>正在下载更新，请勿关闭此窗口…</p> <p>这可能需要一些时间。</p>
update_no_local_schema = <p>未找到本地 Schema (架构)。是否要下载最新的架构？</p><p><b>注意：</b>架构是打开数据表、Loc 文本和其他 PackedFile 的必要条件。没有架构意味着您无法编辑数据表。</p>
update_no_local_template = <p>未找到本地模板。是否要下载最新的模板？</p><p><b>注意：</b>模板有助于通过几次点击快速启动 Mod 开发。</p>
update_no_local_lua_autogen = <p>未找到本地 TW Autogen 数据。是否要下载最新的数据？</p><p><b>注意：</b>TW Autogen 数据可用于（在创建 MyMod 时）在 VSCode 或 Sublime Text 中自动为您的 Mod 设置 Lua 开发环境。</p><p>有关其他信息和支持的游戏，请查看 <a href='https://github.com/chadvandy/tw_autogen'>https://github.com/chadvandy/tw_autogen</a></p>

## Folder Dialogues

new_folder_default = new_folder
new_folder = 新建文件夹

## PackedFile Dialogues

new_file_default = new_file
new_db_file = 新建 DB PackedFile
new_loc_file = 新建 Loc PackedFile
new_txt_file = 新建文本 PackedFile
new_animpack_file = 新建 AnimPack
new_packedfile_name = 新 PackedFile 名称

packedfile_filter = 在此输入以过滤列表中的表格。也支持正则表达式 (Regex)！
table_filter = 在此输入以过滤表格。也支持正则表达式 (Regex)！
merge_tables = 合并表格
merge_tables_new_name = 在此处填写新文件的名称。
merge_tables_delete_option = 删除原始表格

## External FileDialog

open_packfiles = 打开 PackFile

### tips.rs

## PackFile menu tips

tt_packfile_new_packfile = 创建一个新的 PackFile 并打开它。如果您想保留它，请记得稍后保存！
tt_packfile_open_packfile = 打开一个现有的 PackFile，或将多个现有的 PackFile 合并打开。
tt_packfile_save_packfile = 将当前打开的 PackFile 中所做的更改保存到磁盘。
tt_packfile_save_packfile_as = 将当前打开的 PackFile 另存为新的 PackFile，而不是覆盖原始文件。
tt_packfile_save_packfile_for_release = 对 Pack 文件执行自动清理，然后保存。
tt_packfile_install = 将当前选中的 PackFile 复制到所选游戏的 data 文件夹中。
tt_packfile_uninstall = 从所选游戏的 data 文件夹中移除当前选中的 PackFile。
tt_packfile_load_all_ca_packfiles = 尝试同时将所选游戏的每个原版 PackFile 中的所有 PackedFile 加载到 RPFM 中，使用懒加载 (lazy-loading) 方式加载。请注意，如果您尝试保存它，您的电脑可能会卡死。
tt_packfile_settings = 打开设置对话框。
tt_packfile_quit = 退出程序。

tt_change_packfile_type_boot = 将 PackFile 的类型更改为 Boot。您绝对不应该使用它。
tt_change_packfile_type_release = 将 PackFile 的类型更改为 Release。您绝对不应该使用它。
tt_change_packfile_type_patch = 将 PackFile 的类型更改为 Patch。您绝对不应该使用它。
tt_change_packfile_type_mod = 将 PackFile 的类型更改为 Mod。对于应在 Mod 管理器中显示的 Mod，您应该使用此类型。
tt_change_packfile_type_movie = 将 PackFile 的类型更改为 Movie。对于将始终处于激活状态且不会在 Mod 管理器中显示的 Mod，您应该使用此类型。
tt_change_packfile_type_other = 将 PackFile 的类型更改为 Other。这适用于没有写入支持的 PackFile，因此您绝对不应该使用它。
tt_change_packfile_type_data_is_encrypted = 如果勾选，此 PackFile 中的 PackedFile 数据将被加密。不支持保存此类 PackFile。
tt_change_packfile_type_index_includes_timestamp = 如果勾选，此 PackFile 的 PackedFile 索引将包含每个 PackedFile 的“最后修改”日期。请注意，启用此功能的 PackFile 将不会在官方启动器中作为 Mod 显示。
tt_change_packfile_type_index_is_encrypted = 如果勾选，此 PackFile 的 PackedFile 索引将被加密。不支持保存此类 PackFile。
tt_change_packfile_type_header_is_extended = 如果勾选，此 PackFile 的头部将扩展 20 字节。仅在带有加密的竞技场 (Arena) PackFile 中可见。不支持保存此类 PackFile。
tt_change_packfile_type_data_is_compressed = 如果勾选，则在保存时压缩打开的 PackFile 中的每个 PackedFile 的数据。如果您想解压 PackFile，请取消勾选此项，然后保存。

## MyMod menu tips

tt_mymod_new = 打开对话框以创建新的 MyMod。
tt_mymod_delete_selected = 删除当前选中的 MyMod。
tt_mymod_import = 将 MyMod 文件夹中的所有内容移动到 .pack 文件中。如果 MyMod 文件夹中有任何文件被移除，它们也将在 .pack 文件中被删除。
tt_mymod_export = 将 .pack 文件中的所有内容移动到 MyMod 文件夹中。如果 .pack 文件中有任何文件被移除，它们也将在 MyMod 文件夹中被删除。

## GameSelected menu tips

tt_game_selected_launch_game = 尝试在 Steam 上启动当前选中的游戏。
tt_game_selected_open_game_data_folder = 尝试在默认文件管理器中打开当前选中游戏的 Data 文件夹（如果存在）。
tt_game_selected_open_game_assembly_kit_folder = 尝试在默认文件管理器中打开当前选中游戏的 Assembly Kit 文件夹（如果存在）。
tt_game_selected_open_config_folder = 尝试打开 RPFM 的配置文件夹，其中包含 config/schemas/ctd 报告。
tt_game_selected_warhammer_3 = 将 '全面战争：战锤 3 (TW:Warhammer 3)' 设为 '当前游戏 (Game Selected)'。
tt_game_selected_troy = 将 '全面战争传奇：特洛伊 (TW:Troy)' 设为 '当前游戏'。
tt_game_selected_three_kingdoms = 将 '全面战争：三国 (TW:Three Kingdoms)' 设为 '当前游戏'。
tt_game_selected_warhammer_2 = 将 '全面战争：战锤 2 (TW:Warhammer 2)' 设为 '当前游戏'。
tt_game_selected_warhammer = 将 '全面战争：战锤 (TW:Warhammer)' 设为 '当前游戏'。
tt_game_selected_thrones_of_britannia = 将 '全面战争传奇：不列颠王座 (TW: Thrones of Britannia)' 设为 '当前游戏'。
tt_game_selected_attila = 将 '全面战争：阿提拉 (TW:Attila)' 设为 '当前游戏'。
tt_game_selected_rome_2 = 将 '全面战争：罗马 2 (TW:Rome 2)' 设为 '当前游戏'。
tt_game_selected_shogun_2 = 将 '全面战争：幕府将军 2 (TW:Shogun 2)' 设为 '当前游戏'。
tt_game_selected_napoleon = 将 '全面战争：拿破仑 (TW:Napoleon)' 设为 '当前游戏'。
tt_game_selected_empire = 将 '帝国：全面战争 (TW:Empire)' 设为 '当前游戏'。
tt_game_selected_arena = 将 '全面战争：竞技场 (TW:Arena)' 设为 '当前游戏'。

## Special Stuff menu tips

tt_optimize_packfile = 检查并移除 DB 数据表和 Loc 文本中与原版游戏未发生改变的数据（Locs 仅适用于英文用户）。这意味着您的 Mod 将只包含您更改过的内容，从而避免与其他 Mod 发生冲突。
tt_patch_siege_ai = 对导出的地图 PackFile 进行修补与清理。它会修复攻城 AI（如果存在）并移除导致 PackFile 臃肿的无用 xml 文件，从而减小文件体积。

## About menu tips

tt_about_about_qt = 关于 Qt 的信息，Qt 是用于制作此程序的 UI 工具包。
tt_about_about_rpfm = 关于 RPFM 的信息。
tt_about_open_manual = 在 PDF 阅读器中打开 RPFM 的使用手册。
tt_about_patreon_link = 打开 RPFM 的 Patreon 页面。即使您不打算成为赞助者，也去看看吧。我会时不时发布有关下一次更新和开发中功能的信息。
tt_about_check_updates = 检查 RPFM 是否有可用的更新。
tt_about_check_schema_updates = 检查架构 (schemas) 是否有可用的更新。这是在游戏发布补丁后您需要使用的功能。

## global_search_ui/mod.rs

global_search = 全局搜索
global_search_info = 搜索信息
global_search_search = 搜索
global_search_replace = 替换
global_search_replace_all = 全部替换
global_search_clear = 清除
global_search_case_sensitive = 区分大小写
global_search_use_regex = 使用正则表达式 (Regex)
global_search_search_on = 搜索范围

global_search_all = 全部
global_search_anim = 动画 (Anim)
global_search_anim_fragment_battle = 战斗动画片段 (Anim Fragment Battle)
global_search_anim_pack = 动画包 (Anim Pack)
global_search_anims_table = 动画数据表 (Anims Table)
global_search_audio = 音频 (Audio)
global_search_bmd = BMD
global_search_db = 数据库 (DB)
global_search_esf = ESF
global_search_group_formations = 阵型组 (Group Formations)
global_search_image = 图像 (Image)
global_search_loc = 文本 (Loc)
global_search_matched_combat = 匹配战斗 (Matched Combat)
global_search_pack = Pack
global_search_portrait_settings = 肖像设置 (Portrait Settings)
global_search_rigid_model = 刚体模型 (Rigid Model)
global_search_schemas = 架构 (Schemas)
global_search_sound_bank = 音频库 (Sound Bank)
global_search_text = 纯文本 (Text)
global_search_uic = UIC
global_search_unit_variant = 单位变体 (Unit Variant)
global_search_unknown = 未知 (Unknown)
global_search_video = 视频 (Video)

## Filter Dialogues

global_search_file_matches = 文件匹配项
global_search_schema_matches = 架构匹配项

global_search_match_packedfile_column = 文件路径/匹配

global_search_table_name = 表名
global_search_version = 版本
global_search_column_name = 列名
global_search_column = 列索引

## tips

tt_global_search_use_regex_checkbox = 启用正则表达式搜索。请记住，如果提供的正则表达式无效，RPFM 将回退到常规的模式搜索。
tt_global_search_case_sensitive_checkbox = 启用区分大小写搜索。非常容易理解。
tt_global_search_search_on_all_checkbox = 在搜索中包含所有可搜索的 PackedFile/Schemas。
tt_global_search_search_on_dbs_checkbox = 在搜索中包含 DB 数据表。
tt_global_search_search_on_locs_checkbox = 在搜索中包含 LOC 文本。
tt_global_search_search_on_texts_checkbox = 在搜索中包含任何类型的纯文本 PackedFile。
tt_global_search_search_on_schemas_checkbox = 在搜索中包含当前加载的 Schema (架构)。

## Open PackedFile Dialog

open_packedfile_dialog_1 = 您确定吗？
open_packedfile_dialog_2 = 您要替换/删除的一个或多个 PackedFile 处于打开状态。您确定要关闭它吗？

## TreeView Text/Filter

treeview_aai = AaI
treeview_autoexpand = 自动展开匹配项
treeview_expand_all = 全部展开
treeview_collapse_all = 全部折叠

## TreeView Tips

tt_context_menu_add_file = 向当前打开的 PackFile 添加一个或多个文件。现有文件不会被覆盖！
tt_context_menu_add_folder = 向当前打开的 PackFile 添加一个文件夹。现有文件不会被覆盖！
tt_context_menu_copy_to_pack = Copy the selected files/folders to another open PackFile.
tt_context_menu_check_tables = 检查当前打开的 PackFile 中的所有 DB 数据表是否存在依赖错误。
tt_context_menu_new_folder = 打开创建空文件夹的对话框。由于 PackFile 的构建方式，如果文件夹保持为空，它们在保存时不会被保留。
tt_context_menu_new_packed_file_anim_pack = 打开创建 AnimPack 的对话框。
tt_context_menu_new_packed_file_db = 打开创建 DB 数据表（被游戏用来处理……大多数事物）的对话框。
tt_context_menu_new_packed_file_loc = 打开在所选文件夹中创建 Loc 文件（被游戏用来存储您在游戏中看到的文本）的对话框。
tt_context_menu_new_packed_file_text = 打开创建纯文本文件的对话框。它接受不同的扩展名，例如 '.xml'、'.lua'、'.txt' 等。
tt_context_menu_new_queek_packed_file = 打开根据上下文创建 Packedfile 的对话框。例如，如果您在 /text 目录中启动此操作，它将创建一个 loc PackedFile。
tt_context_menu_mass_import_tsv = 同时导入大量 TSV 文件。它会自动检查它们是否是 DB 数据表、Loc 或无效的 TSV，并一次性将它们全部导入。现有文件将被覆盖！
tt_context_menu_mass_export_tsv = 同时将此 PackFile 中的每个 DB 数据表和 Loc PackedFile 导出为 TSV 文件。现有文件将被覆盖！
tt_context_menu_merge_tables = 将多个 DB 数据表/Loc PackedFile 合并为一个。
tt_context_menu_update_tables = 将数据表更新到当前选中游戏已知可用的最新版本。
tt_context_menu_delete = 删除选中的文件/文件夹。
tt_context_menu_extract = 从 PackFile 中提取选中的文件/文件夹。
tt_context_menu_rename = 重命名选中的文件/文件夹。请记住，不允许使用空格，并且同一文件夹中的重复名称将不会被重命名。
tt_context_menu_open_decoder = 在 DB 解码器中打开选中的数据表。用于创建/更新 Schema (架构)。
tt_context_menu_open_dependency_manager = 打开从此 PackFile 引用的 PackFile 列表。
tt_context_menu_open_containing_folder = 在默认文件管理器中打开当前打开的 PackFile 所在位置。
tt_context_menu_open_with_external_program = 在外部程序中打开此 PackedFile。
tt_context_menu_open_notes = 在辅助视图中打开 PackFile 的注释，而无需关闭主视图中当前打开的 PackedFile。
tt_filter_autoexpand_matches_button = 自动展开匹配项。注意：在大型 PackFile（包含超过 10,000 个文件，如 data.pack）中进行筛选并展开所有匹配项，可能会使程序卡顿一段时间。您已被警告。
tt_filter_case_sensitive_button = 启用/禁用树状视图 (TreeView) 的区分大小写过滤。

packedfile_editable_sequence = 可编辑序列 (Editable Sequence)

### Rename Dialogues

rename_move_selection = 重命名/移动选中项
rename_move_selection_instructions = 说明
rename_move_checkbox = 启用全路径移动
rename_move_selection_placeholder = 新路径/名称

### Mass-Import

mass_import_tsv = 批量导入 TSV 文件
mass_import_num_to_import = 要导入的文件数：0。
mass_import_use_original_filename = 使用原始文件名：
mass_import_import = 导入
mass_import_default_name = new_imported_file

mass_import_select = 选择要导入的 TSV 文件…

files_to_import = 要导入的文件数：{"{"}{"}"}。

### Table

decoder_title = PackedFile 解码器
table_dependency_manager_title = 依赖管理器 (Dependency Manager)
table_filter_case_sensitive = 区分大小写
table_enable_lookups = 使用查找 (Lookups)

### Contextual Menu for TreeView

context_menu_add = 添加…
context_menu_create = 创建…
context_menu_open = 打开…

context_menu_add_file = 添加文件
context_menu_add_files = 添加多个文件
context_menu_add_folder = 添加文件夹
context_menu_add_folders = 添加多个文件夹
context_menu_copy_to_pack = Copy To Pack
context_menu_copy_to_pack_no_packs = No other packs open
context_menu_extract_packfile = 提取 PackFile

context_menu_new_folder = 创建文件夹
context_menu_new_packed_file_anim_pack = 创建 AnimPack
context_menu_new_packed_file_db = 创建 DB 数据表
context_menu_new_packed_file_loc = 创建 Loc 文本
context_menu_new_packed_file_portrait_settings = 创建肖像设置 (Portrait Settings)
context_menu_new_packed_file_text = 创建纯文本 (Text)
context_menu_new_queek_packed_file = 新建快速文件 (New Quick File)

context_menu_mass_import_tsv = 批量导入 TSV
context_menu_mass_export_tsv = 批量导出 TSV
context_menu_mass_export_tsv_folder = 选择目标文件夹
context_menu_move = 重命名/移动
context_menu_delete = 删除
context_menu_extract = 提取

context_menu_open_decoder = 使用解码器打开
context_menu_open_dependency_manager = 打开依赖管理器
context_menu_open_containing_folder = 打开所在文件夹
context_menu_open_with_external_program = 使用外部程序打开
context_menu_open_notes = 打开注释

context_menu_check_tables = 检查数据表
context_menu_merge_tables = 合并数据表
context_menu_update_table = 更新数据表

### Shortcuts

menu_bar_packfile_section = PackFile 菜单
menu_bar_mymod_section = MyMod 菜单
menu_bar_view_section = 视图菜单
menu_bar_game_selected_section = 当前游戏菜单
menu_bar_about_section = 关于菜单
packfile_contents_tree_view_section = PackFile 内容右键菜单
packed_file_table_section = 数据表 PackedFile 右键菜单
packed_file_decoder_section = PackedFile 解码器

shortcut_esc = Esc
shortcut_csp = Ctrl+Shift+P

shortcut_title = 快捷键
shortcut_text = 快捷键
shortcut_section_action = 区域/操作

### Settings

settings_title = 设置

settings_game_paths_title = 游戏路径
settings_extra_paths_title = 额外路径
settings_paths_mymod = MyMod 文件夹
settings_paths_mymod_ph = 这是您希望存储所有 "MyMod" 相关文件的文件夹。
settings_paths_zip = 7Zip Exe 路径
settings_paths_zip_ph = 这是 7Zip 可执行文件的完整路径。
settings_game_label = 游戏文件夹
settings_asskit_label = Assembly Kit 文件夹
settings_game_line_ph = 这是您安装了 {"{"}{"}"} 且 .exe 文件所在的文件夹。
settings_asskit_line_ph = 这是您为 {"{"}{"}"} 安装了 Assembly kit 的文件夹。
settings_ui_title = UI 设置
settings_table_title = 数据表设置

settings_ui_language = 语言（需要重启）:
settings_ui_dark_theme = 使用深色主题:
settings_ui_table_adjust_columns_to_content = 调整列宽以适应内容:
settings_ui_table_disable_combos = 在数据表上禁用组合框 (ComboBoxes):
settings_ui_table_extend_last_column_label = 在数据表上延伸最后一列:
settings_ui_table_tight_table_mode_label = 在数据表上启用“紧凑模式”:
settings_ui_table_remember_column_visual_order_label = 记住列的视觉顺序:
settings_ui_table_remember_table_state_permanently_label = 跨 PackFile 记住数据表状态:
settings_ui_window_start_maximized_label = 启动时最大化:

settings_select_file = 选择文件
settings_select_folder = 选择文件夹

settings_extra_title = 额外设置
settings_default_game = 默认游戏:
settings_check_updates_on_start = 启动时检查更新:
settings_check_schema_updates_on_start = 启动时检查 Schema (架构) 更新:
settings_check_template_updates_on_start = 启动时检查模板更新:
settings_allow_editing_of_ca_packfiles = 允许编辑 CA PackFile:
settings_optimize_not_renamed_packedfiles = 优化未重命名的 PackedFile:
settings_use_lazy_loading = 对 PackFile 使用懒加载 (Lazy-Loading):
settings_disable_uuid_regeneration_tables = 在 DB 数据表上禁用 UUID 重新生成:
settings_packfile_treeview_resize_to_fit = 调整 TreeView (树状视图) 大小以适应内容:
settings_table_resize_on_edit = 编辑时调整数据表大小以适应内容:

settings_debug_title = 调试设置
settings_debug_missing_table = 检查缺失的数据表定义
settings_debug_enable_debug_menu = 启用调试菜单

settings_diagnostics_title = 诊断设置
settings_diagnostics_show_panel_on_boot = 启用诊断工具:
settings_diagnostics_trigger_on_open = 打开 PackFile 时触发诊断检查:
settings_diagnostics_trigger_on_edit = 编辑数据表时触发诊断检查:

settings_text_title = 文本编辑器设置

settings_warning_message = <p><b style="color:red;">警告：这些设置中的大多数需要您重启程序才能生效！</b></p><p></p>

### Settings Tips

tt_ui_global_use_dark_theme_tip = <i>至尊戒，驭众戒；至尊戒，寻众戒；至尊戒，引众戒，禁锢黑暗中</i>
tt_ui_table_adjust_columns_to_content_tip = 如果启用此项，当您打开 DB 数据表或 Loc 文本时，所有列都将根据其内容大小自动调整宽度。否则，列将具有预定义的大小。无论哪种方式，您都能在初始调整后手动调整它们的大小。注意：这会使非常大的表格需要更长的加载时间。
tt_ui_table_disable_combos_tip = 如果您禁用此项，数据表中的引用列将不再显示组合框。这意味着 DB 数据表上没有组合框，也没有自动完成功能。现在闭嘴吧，光头 (Baldy)。
tt_ui_table_extend_last_column_tip = 如果启用此项，DB 数据表和 Loc PackedFile 上的最后一列将自行延伸以填满其右侧的空白区域（如果有）。
tt_ui_table_tight_table_mode_tip = 如果启用此项，表格中无用的垂直空间将被压缩，以便您能同时看到更多数据。
tt_ui_table_remember_column_visual_order_tip = 启用此项以使 RPFM 在关闭并重新打开 DB 数据表/LOC 时记住列的视觉顺序。
tt_ui_table_remember_table_state_permanently_tip = 如果您启用此项，RPFM 将在您关闭并再次打开 RPFM 时记住 DB 数据表或 Loc PackedFile 的状态（过滤的数据、移动的列、对表格进行排序的列等）。如果您不想要这种行为，请保持禁用状态。
tt_ui_window_start_maximized_tip = 如果启用此项，RPFM 启动时将最大化。
tt_extra_network_check_updates_on_start_tip = 如果启用此项，RPFM 将在程序启动时检查更新，并在有可用更新时通知您。是否下载由您决定。
tt_extra_network_check_schema_updates_on_start_tip = 如果启用此项，RPFM 将在程序启动时检查架构更新，并在有可用更新时允许您自动下载。
tt_extra_packfile_allow_editing_of_ca_packfiles_tip = 默认情况下，只能编辑 'Mod' 和 'Movie' 类型的 PackFile，因为只有这两种类型用于制作 mod。如果启用此项，您也将能够编辑 'Boot'、'Release' 和 'Patch' PackFile。只是要小心不要覆盖了游戏的原始 PackFile！
tt_extra_packfile_optimize_not_renamed_packedfiles_tip = 如果启用此项，在运行 '优化 PackFile (Optimize PackFile)' 功能时，RPFM 将优化与原版对应文件同名的数据表和 Loc。通常，这些文件旨在完全覆盖其原版对应文件，因此默认情况下（此设置关闭）优化器会忽略它们。但有时优化它们也很有用（AssKit 包含了太多文件），这就是存在此设置的原因。
tt_extra_packfile_use_lazy_loading_tip = 如果启用此项，PackFile 将按需从磁盘加载其数据，而不是将整个 PackFile 加载到内存 (Ram) 中。这大大减少了内存使用量，但是如果其他程序在 PackFile 打开时更改/删除了该文件，则 PackFile 很可能无法恢复，并且您将丢失其中的所有内容。如果您主要在战锤 2 的 /data 文件夹中制作 mod，请保持此项处于禁用状态，因为 Assembly Kit 中的错误会导致在启用此项时损坏/删除 PackFile。
tt_extra_disable_uuid_regeneration_on_db_tables_label_tip = 如果您计划将二进制数据表置于 Git/Svn/任何类型的版本控制软件下，请勾选此项。
tt_debug_check_for_missing_table_definitions_tip = 如果启用此项，RPFM 在打开 PackFile 或更改当前游戏时将尝试解码当前 PackFile 中的每个数据表，并将所有没有架构 (schema) 的数据表输出到 \"missing_table_definitions.txt\" 文件中。这是调试功能，速度非常慢。除非您真的想使用它，否则请勿启用。
tt_diagnostics_enable_diagnostics_tool_tip = 启用此项以使诊断面板在启动时出现。
tt_diagnostics_trigger_diagnostics_on_open_tip = 启用此项以在打开 PackFile 时触发完整的 PackFile 诊断检查。
tt_diagnostics_trigger_diagnostics_on_table_edit_tip = 启用此项以在每次编辑数据表时触发有限的诊断检查。

### CA_VP8 Videos

format = 格式:
version = 版本:
header_len = 标头长度:
codec_four_cc = 编解码器 Four CC:
width = 宽度:
height = 高度:
ms_per_frame = 每帧毫秒数:
num_frames = 帧数:
largest_frame = 最大帧:
mystery_number = 我不知道这是什么:
offset_frame_table = 帧表偏移量:
framerate = 帧率:
timebase = 时基 (Timebase):
x2 = 我不知道这是什么:

convert_to_camv = 转换为 CAMV
convert_to_ivf = 转换为 IVF

notes = 注释

external_current_path = 当前编辑路径:
stop_watching = 停止监视文件
open_folder = 在文件管理器中打开文件夹

game_selected_changed_on_opening = 当前游戏已更改为 {"{"}{"}"}，因为您打开的 PackFile 与您选择的游戏不兼容。

### Extra stuff I don't remember where it goes.

rpfm_title = Rusted PackFile Manager
delete_mymod_0 = <p>您确定要从磁盘中删除此 <i>'MyMod'</i> 吗？</p><p>删除后将无法恢复。</p>
delete_mymod_1 = <p>还有一些未保存的更改。</p><p>您确定要退出吗？</p>
close_tool = <p>您确定要关闭工具吗？</p><p>更改将会丢失。</p>

api_response_success_new_stable_update = <h4>发现新的主要稳定版本更新：{"{"}{"}"}</h4> <p>请在点击“更新”之前确保保存您正在进行的工作，否则您可能会丢失它。</p>
api_response_success_new_beta_update = <h4>发现新的测试版本 (Beta) 更新：{"{"}{"}"}</h4><p>请在点击“更新”之前确保保存您正在进行的工作，否则您可能会丢失它。</p>
api_response_success_new_update_hotfix = <h4>发现新的次要更新/热修复 (Hotfix)：{"{"}{"}"}</h4><p>请在点击“更新”之前确保保存您正在进行的工作，否则您可能会丢失它。</p>
api_response_success_no_update = <h4>没有可用的新更新</h4> <p>祝您下次好运 :)</p>
api_response_success_unknown_version = <h4>检查新更新时出错</h4> <p>在获取最新发布的版本号或当前版本号时出现了问题。这意味着我把上一个发布版本的标题弄糟了。如果您看到此消息，请在此处报告：\n<a href=\"https://github.com/Frodo45127/rpfm/issues\">https://github.com/Frodo45127/rpfm/issues</a></p>
api_response_error = <h4>检查新更新时出错 :(</h4> {"{"}{"}"}

schema_no_update = <h4>没有可用的新架构 (Schema) 更新</h4> <p>祝您下次好运 :)</p>
schema_new_update = <h4>有新的架构 (Schema) 更新可用</h4> <p>您想更新架构吗？</p>

template_no_update = <h4>没有可用的新模板更新</h4> <p>祝您下次好运 :)</p>
template_new_update = <h4>有新的模板更新可用</h4> <p>您想更新模板吗？</p>

lua_autogen_no_update = <h4>没有可用的新 TW Autogen 更新</h4> <p>祝您下次好运 :)</p>
lua_autogen_new_update = <h4>有新的 TW Autogen 更新可用</h4> <p>您想更新 TW Autogen 数据吗？</p>

api_response_schema_error = <h4>检查新更新时出错 :(</h4> <p>如果您看到此消息，说明您连接到 Github.com 服务器时出现了问题。请确保您可以访问 <a href=\"https://api.github.com\">https://api.github.com</a> 然后重试。</p>
schema_update_success = <h4>架构已更新并重新加载</h4><p>您现在可以继续使用 RPFM 了。</p>
template_update_success = <h4>模板已更新并重新加载</h4><p>您现在可以继续使用 RPFM 了。</p>
lua_autogen_update_success = <h4>TW Autogen 数据已更新。</h4><p>您现在可以继续使用 RPFM 了。</p>


files_extracted_success = 文件提取成功。
mymod_delete_success = MyMod 成功删除："{"{"}{"}"}"

game_selected_unsupported_operation = 当前选择的游戏不支持此操作。

optimize_packfile_success = PackFile 已优化。
update_current_schema_from_asskit_success = 当前加载的 Schema (架构) 已更新。
generate_schema_diff_success = 差异分析 (Diff) 生成成功。
settings_font_title = 字体设置

title_success = 成功！
title_error = 错误！
rename_move_instructions = <p>非常简单：</p>
    <ul>
        <li>操作模式：</li>
        <ul>
            <li><b>选中单个文件/文件夹，启用全路径移动</b>：您可以替换路径的任何部分（包括前缀/后缀），文件/文件夹将被移动到新路径。</li>
            <li><b>选中单个文件/文件夹，禁用全路径移动</b>：您可以替换文件/文件夹名称（包括前缀/后缀），文件/文件夹将被重命名为新名称。</li>
            <li><b>在同一文件夹中选中多个文件/文件夹，启用全路径移动</b>：您可以替换路径的任何部分（包括最终名称的前缀/后缀），文件/文件夹将被移动到新路径。</li>
            <li><b>在同一文件夹中选中多个文件/文件夹，禁用全路径移动</b>：您可以对所有文件/文件夹应用前缀/后缀。</li>
            <li><b>从不同文件夹中选中多个文件/文件夹，禁用全路径移动</b>：您可以替换路径的任何部分，文件/文件夹将被移动到新路径。</li>
        </ul>
        <li>额外提示：</li>
        <ul>
            <li>使用 '/' 作为路径分隔符。不能在文件/文件夹名称中使用 '/'。路径不要以 '/' 开头。</li>
            <li>通过将文件/文件夹名称替换为 'yourprefix{"{"}x{"}"}yoursuffix'（包含 {"{"}{"}"}），您可以将前缀/后缀应用于文件/文件夹名称（这对于批量重命名很有用）。</li>
        </ul>
    </ul>

update_table_success = <p>数据表已从版本 '{"{"}{"}"}' 更新到版本 '{"{"}{"}"}'。</p>
update_table_success_files_deleted = </br>已删除的字段：<ul>{"{"}{"}"}</ul>
update_table_success_files_added = </br>已添加的字段：<ul>{"{"}{"}"}</ul>

no_errors_detected = 未检测到错误。
original_data = 原始值：'{"{"}{"}"}'
vanilla_data = 原版/父级值：'{"{"}{"}"}'
column_tooltip_1 = 此列引用了：
column_tooltip_2 = 以及更多。准确地说，还有 {"{"}{"}"} 个。太多了，无法在此处全部显示。
column_tooltip_3 = 引用此列的字段：
column_tooltip_4 = 此字段期望的是文件路径。
column_tooltip_5 = 此字段期望的是以下路径下的文件名：

tsv_select_title = 选择要导入的 TSV 文件…
tsv_export_title = 导出 TSV 文件…

rewrite_selection_title = 重写选中项 (Rewrite Selection)
rewrite_selection_instructions_title = 说明
rewrite_selection_instructions = <p>图例：</p>
    <ul>
        <li>{"{"}x{"}"} 表示当前值。</li>
        <li>{"{"}y{"}"} 表示当前列。</li>
        <li>{"{"}z{"}"} 表示当前行。</li>
    </ul>
rewrite_selection_is_math = 这是一个数学运算吗？
rewrite_selection_placeholder = 在此写下您想要的任何内容。
rewrite_selection_accept = 接受

context_menu_apply_submenu = 应用…
context_menu_clone_submenu = 克隆…
context_menu_copy_submenu = 复制…
context_menu_add_rows = 添加行
context_menu_insert_rows = 插入行
context_menu_delete_rows = 删除行
context_menu_rewrite_selection = 重写选中项 (Rewrite Selection)
context_menu_clone_and_insert = 克隆并插入
context_menu_clone_and_append = 克隆并追加
context_menu_copy = 复制
context_menu_copy_as_lua_table = 复制为 LUA 表
context_menu_copy_to_filter_value = 复制到过滤器值
context_menu_paste = 粘贴
context_menu_search = 搜索
context_menu_sidebar = 侧边栏
context_menu_import_tsv = 导入 TSV
context_menu_export_tsv = 导出 TSV
context_menu_invert_selection = 反选
context_menu_reset_selection = 重置选择
context_menu_resize_columns = 调整列宽
context_menu_undo = 撤销
context_menu_redo = 重做
context_menu_cascade_edition = 重命名引用 (Rename References)

header_column = <b><i>列名</i></b>
header_hidden = <b><i>隐藏</i></b>
header_frozen = <b><i>冻结</i></b>

file_count = 文件数量：
file_paths = 文件路径：
animpack_unpack = 解包 (Unpack)

special_stuff_repack_animtable = 重新打包动画表 (RePack AnimTable)
tt_repack_animtable = 此操作会将动画表（如果找到）重新打包回 AnimPack。

load_template = 加载模板
load_templates_dialog_title = 加载模板
load_templates_dialog_accept = 加载模板

nested_table_title = 嵌套表 (Nested Table)
nested_table_accept = 接受

about_check_template_updates = 检查模板更新
uodate_templates_success = 模板更新成功。
tt_uodate_templates = 此命令尝试更新您的模板。

integer_1 = 未知整数 1：
integer_2 = 未知整数 2：

settings_update_channel = 更新通道
update_success_main_program = <h4>RPFM 更新成功！</h4> <p>要检查此更新中的更改，请点击此链接：<a href='file:///{"{"}{"}"}'>Changelog.md</a>。如果您正在更新到测试版 (beta)，相关的更改在 "Unreleased (未发布)" 部分。</p> <p>请重启程序以应用更改。</p>

settings_autosave_interval = 自动保存间隔（分钟）
autosaving = 正在自动保存…
autosaved = 已自动保存
error_autosave_non_editable = 此 PackFile 无法自动保存。
settings_ui_table_use_old_column_order_label = 使用旧的列顺序（键名排在前面）:

context_menu_paste_as_new_row = 粘贴为新行

gen_loc_diagnostics = 诊断 (Diagnostics)
diagnostics_button_check_packfile = 检查 PackFile
diagnostics_button_check_current_packed_file = 仅检查打开的 PackedFile
diagnostics_button_error = 错误 (Error)
diagnostics_button_warning = 警告 (Warning)
diagnostics_button_info = 信息 (Info)
diagnostics_button_only_current_packed_file = 仅打开的 PackedFile

diagnostics_colum_level = 级别
diagnostics_colum_diag = 诊断
diagnostics_colum_cells_affected = 受影响的单元格
diagnostics_colum_path = 路径
diagnostics_colum_message = 消息

context_menu_copy_path = 复制路径
mymod_open_mymod_folder = 打开 MyMod 文件夹
open_from_autosave = 从自动保存中打开

all = 全部
settings_expand_treeview_when_adding_items = 添加项目时展开 TreeView:
settings_expand_treeview_when_adding_items_tip = 如果您希望在将文件夹添加到 TreeView 时展开它们，请将此项设置为 true。将其设置为 false 则不展开它们。

label_outdated_table = 过时的数据表
label_invalid_reference = 无效的引用
label_empty_row = 空行
label_empty_key_field = 空的主键字段
label_empty_key_fields = 空的主键字段 (复数)
label_duplicated_combined_keys = 重复的组合主键
label_no_reference_table_found = 未找到引用数据表
label_no_reference_table_nor_column_found_pak = 未找到引用的数据表/列
label_no_reference_table_nor_column_found_no_pak = 未找到引用的数据表/列/依赖项
label_invalid_escape = 无效的转义符
label_duplicated_row = 重复的行
label_invalid_dependency_packfile = 无效的依赖 PackFile
label_dependencies_cache_not_generated = 未生成依赖缓存

diagnostics_button_show_more_filters = 显示更多过滤器
diagnostics_colum_report_type = 报告类型

diagnostic_type = 诊断报告类型
diagnostic_show = 显示？
dependency_packfile_list_label = <p><b style="color:red;">警告：勾选“在游戏内之前加载 (Load Before Ingame?)”复选框将强制在当前 Pack 之前加载该 Pack（可能会改变加载顺序），即便是它没有在 Mod 管理器中被选中也是如此！</b></p><p></p>

context_menu_open_packfile_settings = 打开 PackFile 设置
pfs_diagnostics_files_to_ignore_label =
    <span>&nbsp;</span>
    <h3>在诊断检查中忽略的 PackedFile</h3>
pfs_diagnostics_files_to_ignore_description_label =
    <span>&nbsp;</span>
    <p>执行诊断检查时，将忽略此列表上的 PackedFile。它们仍将被用作其他用途的数据源（如提供引用数据），但不会被分析。</p><p><b>每行一个路径。用 # 注释行。</b>以下是有效的示例：</p>
    <ul style="list-style-type: none">
        <li>
            <code>db/land_units_tables</code>
            <ul><li>该文件夹中的所有数据表都将被忽略。</li></ul>
        </li>
        <li>
            <code>db/land_units_tables/table1</code>
            <ul><li>那个确切的数据表将被忽略。</li></ul>
        </li>
        <li>
            <code>db/land_units_tables/table2;field1,field2</code>
            <ul><li>仅忽略该特定数据表的这两个字段。</li></ul>
        </li>
        <li>
            <code>db/land_units_tables;field1,field2</code>
            <ul><li>仅忽略该文件夹中所有数据表的这两个字段。</li></ul>
        </li>
        <li>
            <code>db/land_units_tables/table1;;DiagId1,DiagId2</code>
            <ul><li>仅忽略该特定数据表的这两项诊断检查。手册中提供了可用的过滤器键名。</li></ul>
        </li>
    </ul>
    <br>

pfs_import_files_to_ignore_label = <h3>导入时忽略的文件</h3>
pfs_import_files_to_ignore_description_label = <p>从此列表中的文件将在从 MyMod 文件夹导入时被忽略。仅限 MyMod。路径是相对的，帝国的荣光是绝对的。</p>
pfs_disable_autosaves_label = <span>&nbsp;</span><h3>禁用此 PackFile 的自动保存</h3>
pfs_disable_autosaves_description_label = <p>勾选此项，仅禁用这个特定 Pack 的自动保存功能。</p>
pfs_do_not_generate_existing_locs_label = <span>&nbsp;</span><h3>不生成现有的 Loc 文本</h3>
pfs_do_not_generate_existing_locs_description_label = <p>如果您勾选此项，“生成缺失的 Loc 文本 (Generate Missing Locs)”将不会包含原版或父文件中已存在的 Loc 条目。</p>
    <p>如果您以前使用过此功能，这意味着将不再生成以“aaa_”开头的 loc 文件。</p>

instructions_ca_vp8 = 非常简单，视频可以有两种格式：CAMV（游戏使用）和 IVF（可在带有 VP8 编解码器的媒体播放器上播放）。要导出视频，请将其转换为 IVF 并提取。要让它在游戏中加载，请将其转换为 CAMV 并保存 PackFile。
settings_debug_spoof_ca_authoring_tool = 伪装 CA 的创作工具 (Spoof CA's Authoring Tool)
tt_settings_debug_spoof_ca_authoring_tool = 勾选此项将使 RPFM 保存的所有 PFH6 PackFile 被标记为“使用 CA-TOOL 保存”。仅供测试。

template_name = 名称:
template_description = 描述:
template_author = 作者:
template_post_message = 发布消息:
save_template = 将 PackFile 保存为模板

new_template_sections = 分区 (Sections)
new_template_options = 选项 (Options)
new_template_params = 参数 (Parameters)
new_template_info = 基本信息 (Basic Info)

new_template_sections_description = <p>此模板将被划分为的分区或步骤。</p>
 <p>默认情况下，所有步骤将按此处的顺序显示，但您可以隐藏它们，使其仅在选择了某些选项时出现。列的含义：
    <ul>
       <li>Key (键): 分区的内部名称。</li>
       <li>Name (名称): 用户在使用模板时将看到的文本。</li>
       <li>Required Options (必需选项): 出现此分区所需的选项。</li>
    </ul>
 </p>

new_template_options_description = <p>这些是选项/标志/或者任何您想怎么称呼的东西。</p>
 <p>它们控制将模板加载到 PackFile 时可以启用/禁用模板的哪些部分。例如，在抛射物的模板中，一个选项可以是“具有自定义爆炸？”或“具有自定义显示抛射物？”。</p>
 列的含义：
 <ul>
    <li>第一列是选项的内部名称。</li>
    <li>第二列是用户在使用模板时将看到的文本。</li>
 </ul>

new_template_params_description = <p>这些是可以由用户在将模板加载到 PackFile 时应用于模板的参数。</p>
 <p>它们允许用户针对其用途个性化模板的各个部分，例如更改文件的名称，单元格上的值，…</p>
 列的含义：
 <ul>
    <li>第一列是参数的内部名称。</li>
    <li>第二列是用户在使用模板时将看到的文本。</li>
 </ul>

new_template_info_description = <p>这是您可以设置此模板一些元数据的地方。</p>

key = 键 (Key)
name = 名称 (Name)
section = 分区 (Section)
required_options = 必需选项 (Required Options)
param_type = 参数类型 (Param Type)

load_template_info_section = 模板信息
load_template_options_section = 选项
load_template_params_section = 参数

close_tab = 关闭标签页
close_all_tabs = 关闭所有标签页
close_all_other_tabs = 关闭其他标签页
close_tabs_to_left = 关闭左侧标签页
close_tabs_to_right = 关闭右侧标签页
prev_tab = 下一个标签页
next_tab = 上一个标签页

settings_debug_clear_autosave_folder = 清除自动保存文件夹
settings_debug_clear_schema_folder = 清除架构 (Schema) 文件夹
settings_debug_clear_layout_settings = 清除布局设置
tt_settings_debug_clear_autosave_folder = 使用此功能清除整个自动保存文件夹，以释放磁盘空间，或应用对自动保存数量所做的更改（如果有）。
tt_settings_debug_clear_schema_folder = 使用此功能清除整个架构文件夹。以防更新程序失败。
tt_settings_debug_clear_layout_settings = 使用此功能清除布局特殊设置，并将 UI 恢复至其初始状态。
autosaves_cleared = 自动保存文件夹已删除。下次启动程序时会重新生成。
schemas_cleared = 架构文件夹已删除。请务必记住重新下载架构，以便能够打开数据表。
dependencies_cache_cleared = 依赖文件夹已删除。
settings_autosave_amount = 自动保存数量 (最少 1)
tt_settings_autosave_amount = 设置 RPFM 允许保留的自动保存文件数量。如果您减少了这个数字，需要点击“清除自动保存文件夹”来删除多余的自动保存。请注意，这会重置整个自动保存文件夹。

restart_button = 重启
error_not_booted_from_launcher = 此 RPFM 窗口不是从 "rpfm.exe" 文件启动的，而是直接从 "rpfm_ui.exe" 文件启动的。从 2.3.102 版本开始，您应该从 "rpfm.exe"（或同等文件）启动它，以支持有关更新系统的某些功能。
install_success = PackFile 安装成功。
uninstall_success = PackFile 卸载成功。
outdated_table_explanation = 数据表具有内部版本号，每当 CA 对该表进行更新并改变其结构时，该版本号就会更改。
    过时的数据表意味着您的表格可能存在较新版本中引入的结构差异，例如新列或已更改的列。
    根据表格和具体更改，这可能会导致各种后果，从无法使用某些新功能到直接崩溃。
    强烈建议在游戏发布补丁后始终更新您的数据表：打开您的 PackFile，右键单击您的数据表，然后点击“更新数据表 (Update Table)”。
    请注意，更新时 RPFM 会用默认数据填充新列。
    更新表格后，请确保其数据仍然正确！
    否则，您可能会发现您需要在新列中填入某些内容，游戏才不会崩溃…

invalid_reference_explanation = 某些表格列会引用其他表格的列。“无效引用”意味着单元格中存在的数据并不存在于该单元格引用的任何表格中。
    这通常是由于拼写错误、表格更新、或者是某个子 Mod 没有引用父 Mod 导致的…

    这是启动时崩溃的最常见原因之一，如果您想避免崩溃，必须在它们弹出时确保修复这些问题。
    一种特殊情况是，如果此 Mod 是另一个 Mod 的子 Mod。
    在这种情况下，您必须打开您的 PackFile，右键单击它，然后点击“打开/打开依赖管理器 (Open/Open Dependency Manager)”。
    然后，将父 Mod 的全名添加到列表中。例如，“Luccini.pack”。
    这将使 RPFM 在检查此类错误时将该 Mod 考虑在内。

empty_row_explanation = 表格中的空行没有任何用途，并且从长远来看会导致各种问题。强烈建议移除它们。

empty_key_field_explanation = 数据表可能具有一个或多个“主键 (key)”列，这些列（通常）在整个表中必须是唯一的。空的主键字段可能会导致问题，从效果不生效到游戏崩溃不等。强烈建议修复它们。

empty_key_fields_explanation = 数据表可能具有一个或多个“主键 (key)”列，这些列（通常）在整个表中必须是唯一的。此错误意味着一行中所有的“主键”列都是空的。空的主键字段可能会导致问题，从效果不生效到游戏崩溃不等。强烈建议修复它们。

duplicated_combined_keys_explanation = 数据表可能具有一个或多个“主键 (key)”列，这些列（通常）在整个表中必须是唯一的。此错误意味着您有两行具有相同的主键。重复的条目意味着只有一个会被加载，这可能会导致潜在的问题：当您编辑其中一个的值时，另一个会覆盖它，导致您的更改无效。如果这是由于误报引起的，请右键单击此诊断信息并忽略它，这样它就不会在这种情况下再次弹出。

no_reference_table_found_explanation = 某些表格列会引用其他表格的列。这意味着找到了一个引用了 RPFM 无法找到的表格的列。这要么是 Schema (架构) 的问题，要么只是 CA 忘记更新的表格引用。在任何情况下，此消息仅供参考，您可以忽略它。

no_reference_table_nor_column_found_pak_explanation = 在 Assembly Kit 中找到的某些表格由于不同原因并未在 data.pack 或同等文件中。为了能够快速读取这些表格，RPFM 将它们存储在通过进入“特殊功能 (Special Stuff)”并点击“生成依赖缓存 (Generate Dependencies Cache)”生成的缓存文件中。此消息意味着一个表格正在引用另一个表格上的一列，但在引用的表格中找不到该列。甚至在缓存中存储的表格中也找不到。此消息是无害的，仅用于内部调试。您可以忽略它。

no_reference_table_nor_column_found_no_pak_explanation = 在 Assembly Kit 中找到的某些表格由于不同原因并未在 data.pack 或同等文件中。为了能够快速读取这些表格，RPFM 将它们存储在通过进入“特殊功能 (Special Stuff)”并点击“生成依赖缓存 (Generate Dependencies Cache)”生成的缓存文件中。此消息意味着一个表格正在引用另一个表格上的一列，但在引用的表格中找不到该列，并且 RPFM 没有找到为游戏生成的依赖缓存，因此它不知道问题是由于缺少缓存文件还是由于错误引起的。如果您看到此消息，请通过进入“特殊功能”并点击“生成依赖缓存”来为您的游戏生成缓存。

invalid_escape_explanation = 某些字符，如 \n（换行符）和 \t（制表符）需要以特殊方式转义，以便游戏能够识别它们。此错误意味着 RPFM 检测到了其中一个未正确转义的字符，导致其在游戏中显示不正确。要修复它，请确保您改用 \\n 或 \\t（使用两个斜杠）。

duplicated_row_explanation = 数据表行通常向游戏传达一项特定的数据。例如，某一行可能指示 X 单位具有 X 能力。此错误意味着表中存在 2 个或更多完全相同的行。这可能会导致问题，建议在表中仅保留每行的一个副本。

invalid_loc_key_explanation = RPFM 检测到您的 Loc 文本中的某一行具有包含无效字符的键 (Key)。这可能会导致各种问题，包括崩溃，因此最好尽快修复它。造成这种情况的一个常见原因是旧版 PFM 代码（没错，就是 PFM）中的一个旧 Bug，如果您复制/粘贴 Loc 键，会导致它们在末尾添加无效字符。要修复它，请编辑报告的单元格并删除其上任何无效（通常是不可见）的字符。

invalid_dependency_pack_file_name_explanation = 依赖管理器中的某个 PackFile 名称格式无效。导致此错误的原因包括：
    - 依赖管理器上的空行。
    - PackFile 名称不以 ".pack" 结尾。
    - PackFile 名称包含空格。

pfs_button_apply = 应用设置
cascade_edition_dialog = 重命名引用
template_load_final_message = 至此，模板已完成。请确保遵循此处的步骤（以防模板需要它们）。

is_required = 是必需的
context_menu_generate_ids = 生成 ID
generate_ids = 生成 ID
generate_ids_title = 生成 ID
generate_ids_instructions_title = 说明
generate_ids_instructions = 非常简单，在下方框中写入初始 ID，然后点击接受。
generate_ids_accept = 接受

context_menu_delete_filtered_out_rows = 删除过滤掉的行
are_you_sure_delete_filtered_out_rows = 您确定要删除所有被过滤掉的行吗？
context_menu_go_to = 转到…
context_menu_go_to_definition = 转到定义
context_menu_go_to_file = 转到文件
source_data_for_field_not_found = 找不到所选数据的来源。
file_for_field_not_found = 找不到所选数据中引用的文件。
context_menu_go_to_loc = 转到 Loc 条目：{"{"}{"}"}
loc_key_not_found = 找不到该 loc 条目。

table_filter_show_blank_cells = 显示空白单元格
table_filter_show_edited_cells = 显示已编辑的单元格
special_stuff_rescue_packfile = 救援 PackFile (Rescue PackFile)
are_you_sure_rescue_packfile = 您确定要执行此操作吗？这是一个危险的选项，除非开发者或 RPFM 特别告诉您使用它，否则绝不应该使用。所以再次确认，您确定要使用它吗？

filter_group = 组
are_you_sure_delete = 您确定要删除选中的 PackedFile 吗？
label_invalid_loc_key = 无效的 Loc 键 (Key)
info_title = 信息
category_title = 类别 {"{"}{"}"}
equipment_title = 装备
save_changes = 保存更改
debug_view_save_success = PackedFile 已保存。

special_stuff_generate_dependencies_cache = 生成依赖缓存
tt_generate_dependencies_cache = 为当前选中的游戏生成依赖缓存，以便 RPFM 可以快速访问游戏数据而不会占用过多内存。
generate_dependency_cache_success = 依赖缓存已成功创建并重新加载。

dependencies_cache_not_generated_explanation = 未为当前选择的游戏生成依赖缓存。
    没有它，RPFM 无法执行某些依赖它的操作，例如对数据表进行诊断，或进行数据表的引用检查。
    要生成它，请转到“特殊功能/您的游戏/生成依赖缓存 (Special Stuff/yourgame/Generate Dependencies Cache)”，然后等待它完成。
    请记住在游戏更新补丁后也要执行此操作，以便缓存能随着新更改而更新。

label_invalid_packfile_name = 无效的 PackFile 名称
invalid_packfile_name_explanation = PackFile 名称不能包含空格字符。
    要修复它，请将 PackFile 名称中的任何空格替换为下划线。

label_table_name_ends_in_number = 表名以数字结尾
table_name_ends_in_number_explanation = DB 数据表名称末尾的数字通常会导致一个非常奇怪的问题，即 Mod 会让除了制作它的 Modder 之外的任何人崩溃。
    要修复它，请删除报告的 DB 数据表名称末尾的数字。

label_table_name_has_space = 表名包含空格
table_name_has_space_explanation = Cataph 不喜欢它们。
    此外，这有时会导致表格完全无法加载。
    请将表格名称中的任何空格替换为下划线。

label_table_is_datacoring = 表格正在进行数据核心化 (Datacoring)
table_is_datacoring_explanation = 当您的 Mod 中有一个数据表（或任何文件，说实话）具有与原版文件完全相同的路径时，您的 Mod 会完全覆盖它。
    当这发生在数据表上时，被称为“数据核心化 (Datacoring)”，这是您需要注意的事情。
    Datacoring 会将原版数据表替换为您的表，因此如果被替换表格中的数据未出现在您修改过的表格中，您的 Mod 将与同样替换了该表格或依赖该替换表格中数据的任何其他 Mod 不兼容。
    因此，除了在万不得已的情况下（比如您确实想要从原版表格中删除一行），应该避免“Datacoring”。
    此警告是为了提醒您，您正在（有意或无意地）进行数据核心化。
    如果是无意的，请将报告的表格重命名为其他名称。
    如果是有意的，您可以通过进入 PackFile 设置（“右键单击 PackFile/打开…/打开 PackFile 设置”）并在此处将此警告列入黑名单来隐藏此消息。

label_dependencies_cache_outdated = 依赖缓存已过时
label_dependencies_cache_could_not_be_loaded = 依赖缓存无法加载

dependencies_cache_outdated_explanation = 依赖缓存已过时，必须重新生成。
    这通常是由于游戏更新或有人修改了游戏文件而发生的。
    RPFM 需要最新的依赖缓存才能提供诊断、表格自动完成、表格创建等功能……因此保持更新非常重要。
    要修复它，请转到“特殊功能/您的游戏/生成依赖缓存 (Special Stuff/yourgame/Generate Dependencies Cache)”，然后等待它完成。

dependencies_cache_could_not_be_loaded_explanation = RPFM 无法加载依赖缓存。这可能是由多种原因引起的，例如：
    - RPFM 无法读取游戏文件，因为另一个程序锁定了它们，或者它们丢失了。
    - RPFM 无法读取依赖缓存本身或其文件夹。
    - 还有更多。
    太多了无法一一列举。
    返回的错误消息是：{"{"}{"}"}

generate_dependencies_cache_are_you_sure = 您要生成依赖缓存吗？

optimize_packfile_are_you_sure = <h3>您确定要优化此 PackFile 吗？</h3>
    <p>
        如果您不确定，请在使用此功能之前进行备份，因为我不想再次听到诸如“我按了这个，然后我的 Mod 就消失了！！！”之类的抱怨。
        它的作用是：
        <ul>
            <li><b>删除 DB 数据表上的重复条目</b>（除非该表是 datacoring）。</li>
            <li><b>删除 LOC 文本上的重复条目</b>（除非该表是 datacoring）。</li>
            <li><b>删除 DB 数据表上与默认行未发生更改的行</b>（除非该表是 datacoring）。</li>
            <li><b>删除 LOC 文本上与默认行未发生更改的行</b>（除非该表是 datacoring）。</li>
            <li><b>删除未发生更改的 DB 数据表条目（与原版或父文件相比）</b>（除非该表是 datacoring）。</li>
            <li><b>删除未发生更改的 LOC 文本条目（与原版或父文件相比）</b>（除非该表是 datacoring）。</li>
            <li><b>删除空的 DB 数据表。</b></li>
            <li><b>删除空的 LOC 文本。</b></li>
            <li><b>删除地图 Pack 中无用的 xml</b>，这是 bob 导出地图 Pack 时产生的副产品。</li>
            <li><b>删除 prefabs 中无用的 xml</b>，这是 bob 导出 prefabs 时产生的副产品。</li>
            <li><b>删除无用的 agf 和 model_statistics 文件</b>，这是 bob 导出模型时产生的副产品。</li>
            <li><b>从肖像设置文件中删除未使用/无效的变体和艺术集。</b></li>
            <li><b>删除空的肖像设置 (Portrait Settings) 文件</b>（仅限 .bin 文件，保留 .xml 文件以防您使用 selfie）。</li>
        </ul>
        所以，您确定要执行此操作吗？
    </p>

animpack_view_instructions = <h3>如何使用此视图：</h3>
    <ul>
        <li><b>如果您想将 PackFile 中的内容添加到 AnimPack</b>：双击左侧面板上要添加的文件。</li>
        <li><b>如果您想从 AnimPack 提取文件到 PackFile</b>：双击右侧面板上要添加的文件。</li>
        <li><b>如果您想从 AnimPack 中删除文件</b>：在右侧面板上选择要删除的内容，然后按 Delete 键。</li>
   </ul>

send_table_for_decoding = 发送表格进行解码
cancel = 取消
send = 发送
send_table_for_decoding_explanation = <p>您即将发送一个数据表供 RPFM 的作者解码。</p>
    <p>在点击发送之前，请确保以下数据是正确的，如果不正确请取消：
        <ul>
            <li><b>选择的游戏</b>：{"{"}{"}"}。</li>
            <li><b>要解码的表格类型</b>：{"{"}{"}"}。</li>
        </ul>
        这些正确吗？如果正确，请点击发送，如果没有出现问题，表格应在后台发送。
    </p>
    <p>PS：请在发送数据表之前检查 Schema (架构) 更新。自从我启用此功能以来，我收到的大多数表格都已经被解码了，这意味着你们使用了过时的 Schema。我不想因为被不需要解码的表格刷屏而不得不移除这个功能，所以拜托了，只有当最新的 Schema 中真的没有解码该表格时，才发送它。</p>

field_with_path_not_found_explanation = 报告的单元格中的数据应该包含一个路径/文件名，但是在该 Mod、该 Mod 依赖的任何 Mod 或原版文件中都没有找到该路径/文件名。
    请确保单元格中的值是存在的路径。
    对于仅需要文件名而不是完整路径的单元格，请将鼠标悬停在其列标题上方以了解该文件应位于什么路径下。

label_field_with_path_not_found = 字段中的路径/文件未找到
settings_enable_rigidmodel_editor = 启用 RigidModel (刚体模型) 编辑器：
tt_settings_debug_enable_rigidmodel_editor = 此设置允许您在遇到任何问题时禁用新的 RigidModel 编辑器（仍在测试中），以便您仍然可以不使用它来运行 RPFM。

settings_use_right_side_markers = 使用右侧标记 (Right-Side Markers)：
tt_ui_table_use_right_side_markers_tip = 在标记战争中选择一方。现在就加入右派 (Rights) 吧！

settings_tab_paths = 路径
settings_tab_settings = 设置

settings_ui_table_colour_table_added_label = 已添加
settings_ui_table_colour_table_modified_label = 已修改
settings_ui_table_colour_diagnostic_error_label = 错误
settings_ui_table_colour_diagnostic_warning_label = 警告
settings_ui_table_colour_diagnostic_info_label = 信息

settings_ui_table_colour_light_label = 浅色主题
settings_ui_table_colour_dark_label = 深色主题

label_incorrect_game_path = 游戏路径不正确：
incorrect_game_path_explanation = RPFM 检测到您在设置中设置的游戏路径不正确。这个路径是许多、非常多功能正常工作所必需的。所以请正确设置它。

generate_dependencies_cache_warn = 未找到此游戏安装的 Assembly Kit，或者其路径配置不正确。
    这意味着 RPFM 仍将尝试生成依赖缓存，但诊断工具可能会产生大量误报。

are_you_sure_rename_db_folder = <p>您正试图打破 DB 编辑的黄金法则：<b>永远不要重命名/移动数据表文件夹 (TABLE FOLDERS)</b>。</p>
    <p>这样做会导致您的游戏无法正确加载 Mod，或在启动时崩溃。</p>

    <p>如果您这样做是因为有人告诉您<i>重命名数据表</i>，他/她/它的意思是指重命名数据表文件，而不是数据表文件夹。</p>

    <p>此对话框中甚至有<b>继续 (Yes)</b> 按钮的唯一原因，是为了应对非常特定的情况——您正试图修复其他人重命名的数据表文件夹。</p>
   
    <p>如果这不是您的情况，请<b>点击“否 (No)”或退出此对话框</b>，并记住：<b>永远不要重命名数据表文件夹</b>。</p>

gen_loc_dependencies = 依赖项
context_menu_import = 导入
dependencies_asskit_files = Assembly Kit 文件
dependencies_game_files = 游戏文件
dependencies_parent_files = 父文件
import_from_dependencies = 从依赖项导入
global_search_search_source = 搜索来源
global_search_source_packfile = Packfile
global_search_source_parent = 父文件
global_search_source_game = 游戏文件
global_search_source_asskit = Assembly Kit 数据表
menu_bar_tools = 工具
tools_faction_painter = 派系涂色器 (Faction Painter)
faction_painter_title = 派系涂色器
banner = 旗帜
uniform = 制服
primary = 主色
secondary = 次色
tertiary = 第三色
restore_initial_values = 恢复初始值
restore_vanilla_values = 恢复原版值
packed_file_name = PackedFile 名称
tools_unit_editor = 单位编辑器 (Unit Editor)
unit_editor_title = 单位编辑器

settings_enable_esf_editor = 启用 ESF/CCD/SAVE 编辑器（实验性）:
tt_settings_debug_enable_esf_editor = 此设置允许您启用新的 ESF 编辑器（实验性），但请注意可能存在的问题。

settings_enable_unit_editor = 启用单位编辑器（实验性）:
tt_settings_debug_enable_unit_editor = 此设置允许您启用新的单位编辑器（实验性），但请注意可能存在的问题。
tools_unit_editor_main_tab_title = 单位基本信息
tools_unit_editor_land_unit_tab_title = 陆战
tools_unit_editor_variantmeshes_tab_title = 变体网格 (Variant Mesh)
tools_unit_editor_key_loc_data = 键与文本数据 (Key & Loc Data)
tools_unit_editor_requirements = 需求
tools_unit_editor_campaign = 战役
tools_unit_editor_ui = UI
tools_unit_editor_audio = 音频
tools_unit_battle_visibility = 战斗可见度
tools_unit_multiplayer = 多人游戏
tools_unit_extra_data = 额外数据
copy_unit = 复制单位
generate_dependencies_cache_in_progress_message = 正在生成依赖缓存...这可能需要一些时间。
copy_unit_instructions = <p>在输入框中写入新单位的键 (key)，然后点击接受。</p>
    <p>另外，请注意：</p>
    <ul>
        <li>现有的单位键无效。</li>
        <li>复制单位中的某些键将被更改以匹配新的单位键。</li>
    </ul>

copy_unit_new_unit_name = 单位键 (Unit Key)
settings_disable_file_previews = 禁用 PackedFile 预览
tt_settings_disable_file_previews_tip = 勾选此项可使 RPFM 始终以非预览模式打开 PackedFile，因此在打开另一个 PackedFile 时它们不会被自动关闭。
variant_editor_title = 变体编辑器 (Variant Editor)
variants_variant_filename = 变体网格文件名
variants_mesh_editor_title = 变体网格编辑器
unit_variants_colours_title = 变体颜色
unit_variants_unit_card = 单位兵牌
unit_variants_colours_primary_colour = 主色
unit_variants_colours_secondary_colour = 次色
unit_variants_colours_tertiary_colour = 第三色
faction_list_title = 派系（* 表示没有特定派系）
unit_variants_colours_list_title = 颜色变体 (键/Key)

context_menu_add_faction = 添加派系
context_menu_clone_faction = 克隆派系
context_menu_delete_faction = 删除派系
context_menu_add_colour_variant = 添加颜色变体
context_menu_clone_colour_variant = 克隆颜色变体
context_menu_delete_colour_variant = 删除颜色变体

new_faction_title = 新建/克隆派系
new_faction_instructions = <p>选择您希望此单位拥有特定变体的派系。</p>
    <p>另外，请注意：</p>
    <ul>
        <li>已为变体选择的派系无效。</li>
    </ul>
new_faction_name = 派系

new_colour_variant_title = 新建/克隆颜色变体
new_colour_variant_instructions = <p>在输入框中写入新的颜色变体键 (key)，然后点击接受。</p>
    <p>另外，请注意：</p>
    <ul>
        <li>现有的颜色变体键无效。</li>
        <li>键必须是数字。</li>
    </ul>

new_colour_variant_name = 颜色变体键 (Colour Variant Key)

line_counter = 过滤器中的行数 / 数据表中的行数: {"{"}{"}"} / {"{"}{"}"}
new_tip_tip = 消息:
new_tip_path = 路径:
new_tip_link = 链接:
new_tip_dialog = 新消息
tip_id = Id:
tip_link = 链接:
new_tip = 新建快速笔记
tip_edit = 编辑快速笔记
tip_delete = 删除快速笔记
toggle_quick_notes = 切换快速笔记

debug_colour_light_label = 浅色主题
debug_colour_dark_label = 深色主题

debug_colour_local_tip_label = 本地
debug_colour_remote_tip_label = 远程
banned_tables_warning = <p><b style="color:red;">警告：此数据表受到游戏的持续检查，对其进行任何更改都会导致游戏崩溃。RPFM 不会保存您对其进行的任何编辑，如果您在 PackFile 中编辑了它，建议您将其删除</b></p><p></p>
label_banned_table = 检测到被封禁的数据表 (Banned Table):
banned_table_explanation = 被封禁的数据表 (Banned Tables) 是游戏会主动检查以确保未被篡改的数据表。
    更改它们意味着您的游戏会崩溃。这意味着……在制作 Mod 时，除了提供信息之外，它们并没有什么实际用途。
    RPFM 可以读取这些数据表，但不会保存对它们所做的任何编辑，如果您在 PackFile 中包含了其中一个，最好直接删除它们以避免崩溃。
settings_check_message_updates_on_start = 启动时检查消息更新:
import_schema_patch = 导入架构补丁 (Schema Patch)
import_schema_patch_title = 导入架构补丁
import_schema_patch_button = 导入补丁
import_schema_patch_success = 补丁导入成功。
label_value_cannot_be_empty = 值不能为空:
value_cannot_be_empty_explanation = 此列的值不能为空。
    这基本上意味着如果您将此列的值留空，您的游戏可能会崩溃。
    如果您认为这是误报，欢迎提交架构补丁来修复它。
context_menu_find_references = 查找引用
gen_loc_references = 引用
reference_search_data_source = 数据源
reference_search_path = 路径
reference_search_column_name = 列名
reference_search_column_number = 列索引
reference_search_row_number = 行索引

view_toggle_references_panel = 切换引用窗口

tt_settings_debug_clear_dependencies_cache_folder = 使用此功能清除依赖缓存文件夹。
    以防您不想让 RPFM 占用太多内存 (RAM)。
settings_debug_clear_dependencies_cache_folder = 清除依赖缓存文件夹
context_menu_generate_missing_loc_data = 生成 Loc 文本数据
about_check_lua_autogen_updates = 检查 TW Autogen 更新
settings_check_lua_autogen_updates_on_start = 启动时检查 TW Autogen 更新:
tt_about_check_lua_autogen_updates = 检查 TW Autogen 数据是否有可用更新。
    这有助于使用 lua 脚本开发 MyMod。
update_lua_autogen_checker = 更新 TW Autogen 检查器
new_mymod_instructions = <p>创建新 Mod 前需要考虑的事项：</p>
    <ul>
    <li> 选择您要为其制作 Mod 的游戏。</li>
    <li> 选择一个简单的名称（不应以 *.pack 结尾）。</li>
    <li> 如果要使用多个单词，请使用 "_" 而不是空格。</li>
    <li> 您不能为未在设置中设置路径的游戏创建 Mod。</li>
    <li> 根据在创建 MyMod 时选择的选项，<br/> 某些额外文件可能会自动添加到忽略列表中。</li>
    </ul>

new_mymod_lua_support = Lua 支持
new_mymod_git_support = 使用 GitIgnore 创建 Git 仓库
new_mymod_sublime_support = 创建 Sublime Text 项目
new_mymod_vscode_support = 创建 VSCode 项目
new_mymod_gitignore_contents = GitIgnore 内容:
new_mymod_pack_import_ignore_contents = 导入时忽略的文件:
new_mymod_gitignore_same_as_files_ignored_on_import = 与导入时忽略的文件相同
mymod_error_spaces_on_name = 错误: Mod 名称不能包含空格。
new_mymod_pack_import_ignore_contents_placeholder = 相对路径，每行一个。
new_mymod_gitignore_contents_placeholder = 被 git 忽略的路径，每行一个。
global_search_search_placeholder = 搜索
global_search_replace_placeholder = 替换

github_link = 打开 RPFM Github 页面
discord_link = 打开 "The Modding Den" Discord 频道
open_manual = 打开 RPFM 使用手册
patreon_link = 在 Patreon 上支持我
reload_style_sheet = 重新加载样式表 (StyleSheets)

portrait_settings_head_camera_settings_title = 头部摄像机
portrait_settings_body_camera_settings_title = 身体摄像机
portrait_settings_variants_title = 变体 (Variants)
portrait_settings_head_z = Z:
portrait_settings_head_y = Y:
portrait_settings_head_yaw = 偏航角 (Yaw):
portrait_settings_head_pitch = 俯仰角 (Pitch):
portrait_settings_head_distance = 距离:
portrait_settings_head_theta = Theta角:
portrait_settings_head_phi = Phi角:
portrait_settings_head_fov = 视野 (FOV):
portrait_settings_head_skeleton_node = 骨骼节点 (Skeleton Node):
portrait_settings_body_z = Z:
portrait_settings_body_y = Y:
portrait_settings_body_yaw = 偏航角 (Yaw):
portrait_settings_body_pitch = 俯仰角 (Pitch):
portrait_settings_body_fov = 视野 (FOV):
portrait_settings_body_skeleton_node = 骨骼节点 (Skeleton Node):
portrait_settings_file_diffuse_label = 漫反射 (Diffuse):
portrait_settings_file_mask_1_label = 遮罩 1 (Mask 1):
portrait_settings_file_mask_2_label = 遮罩 2 (Mask 2):
portrait_settings_file_mask_3_label = 遮罩 3 (Mask 3):
portrait_settings_season_label = 季节:
portrait_settings_level_label = 等级:
portrait_settings_age_label = 年龄:
portrait_settings_politician_label = 政治家:
portrait_settings_faction_leader_label = 派系领袖:

context_menu_clone = 克隆
portrait_settings_filter = 过滤器

portrait_settings_list_id_error = 该 ID 已存在。
portrait_settings_id = Id
portrait_settings_id_title = 编辑 ID

move_field_up = 上移字段
move_field_down = 下移字段
move_field_left = 左移字段
move_field_right = 右移字段
delete_field = 删除字段
load_definition = 加载定义
delete_definition = 删除定义

label_invalid_art_set_id = 无效的艺术集 (Art Set) Id
label_invalid_variant_filename = 无效的变体文件名
label_file_diffuse_not_found_for_variant = 未找到变体的漫反射文件
label_file_mask_1_not_found_for_variant = 未找到变体的遮罩 1 文件
label_file_mask_2_not_found_for_variant = 未找到变体的遮罩 2 文件
label_file_mask_3_not_found_for_variant = 未找到变体的遮罩 3 文件
label_datacored_portrait_settings = 正在数据核心化 (Datacoring) 肖像设置文件
invalid_art_set_id_explanation = 您在肖像设置文件中拥有一个不存在于 'campaign_character_arts' 数据表中的艺术集 (Art Set) Id。
    这可能是拼写错误，或者是未使用的艺术集 id……但通常是拼写错误。
invalid_variant_filename_explanation = 您在肖像设置文件中拥有一个包含变体文件名的艺术集 Id，但该变体文件名不存在于 'variants' 数据表中。
    这可能是拼写错误，或者是未使用的变体文件名……但通常是拼写错误。
file_diffuse_not_found_for_variant_explanation = 您在肖像设置文件上的漫反射路径（用于 2D 肖像的文件路径）指向了一个不存在的文件。
    这会导致单位使用默认肖像。

file_mask_1_not_found_for_variant_explanation = 您在肖像设置文件上的遮罩 1 路径指向了一个不存在的文件。
file_mask_2_not_found_for_variant_explanation = 您在肖像设置文件上的遮罩 2 路径指向了一个不存在的文件。
file_mask_3_not_found_for_variant_explanation = 您在肖像设置文件上的遮罩 3 路径指向了一个不存在的文件。
datacored_portrait_settings_explanation = 您正在覆盖/数据核心化 (datacoring) 一个肖像设置文件。

    在 99% 的情况下，您不想这么做，因为这样做可能会导致各处的单位都没有肖像图标/图像。
    通常，您只需要专门为您的单位创建一个新的肖像设置文件，而不是覆盖原版文件。
new_portrait_settings_file = 新建肖像设置文件
new_portrait_settings_copy_column = 复制值？
new_portrait_settings_copy_from_column = 从此条目复制值
new_portrait_settings_copy_to_column = 将值复制到此条目

live_export_success = Script 和 UI 文件夹导出成功。
include_base_folder_on_add_from_folder = 从文件夹添加时包含父文件夹
settings_include_base_folder_on_add_from_folder = 使用“从文件夹添加 (Add From Folder)”时，它会将选定的文件夹本身而不是其子内容添加到 PackFile 中。
delete_empty_folders_on_delete = 移动/删除内容后删除空文件夹
settings_delete_empty_folders_on_delete = 如果启用此项，在执行某些会留下空文件夹的操作后，这些文件夹将被自动删除。
schema_patch_submitted_with_empty_explanation = 补丁未提交，因为解释说明为空。
diagnostics_check_ak_only_refs = 检查仅限 AK 的引用 (可能会触发误报)
title_changes_detected_in_dark_theme_config = 检测到深色主题样式表更改
message_changes_detected_in_dark_theme_config = <p>您看到此消息是因为 RPFM 刚刚更新，要么更新包含了对深色主题的更改，要么您在某个时候对 dark-theme-custom.qss 文件进行了自定义更改。</p>
    <p>如果您没有编辑 dark-theme-custom.qss 文件，请按“是 (Yes)”导入更新后的深色主题。
    如果您使用自定义主题更改了该文件，请按“否 (No)”并手动将您想要的更改从 dark-theme.qss 导入到 dark-theme-custom.qss 中。</p>

pack_map = Pack 地图 (Pack Map)
tile_maps = 图块地图 (Tile Maps)
tiles = 图块 (Tiles)
special_stuff_pack_map = Pack 地图

ignore_parent_folder = 忽略父文件夹
ignore_parent_folder_field = 父文件夹的忽略字段
ignore_file = 忽略文件
ignore_file_field = 文件的忽略字段
ignore_diagnostic_for_parent_folder = 忽略父文件夹的诊断
ignore_diagnostic_for_parent_folder_field = 在父文件夹的字段中忽略诊断
ignore_diagnostic_for_file = 忽略文件的诊断
ignore_diagnostic_for_file_field = 在文件的字段中忽略诊断
ignore_diagnostic_for_pack = 忽略 Pack 的诊断

autosave_folder_size_warning = <p>您的自动保存文件夹大小已超过 25GB。</p>
    <p>此消息旨在提醒您，以免您不知道这些 GB 的空间都去哪了。
    如果您想删除现有的自动保存，请转到 <i>PackFile/设置 (Settings)</i> 并点击 <b>清除自动保存文件夹 (Clear Autosave Folder)</b>。</p>
    <p>这是一次性消息。直到您的自动保存文件夹低于 25GB 并再次超过 25GB 时，它才会再次出现。</p>

portrait_settings_file_name = 文件名
table_filter_use_regex = 使用正则表达式 (Regex)
settings_enable_lookups = 启用查找 (Lookups)

context_menu_profiles_apply = 应用配置文件
context_menu_profiles_delete = 删除配置文件
context_menu_profiles_create = 新建配置文件
context_menu_profiles_set_as_default = 设为默认配置文件
new_profile_title = 新建配置文件
new_profile_instructions = 随便起个名字。
    如果此数据表已存在同名的配置文件，它将被覆盖。
new_profile_placeholder_text = 名称。
new_profile_no_name_error = “随便起个名字”这句话你是有哪里不明白吗？
global_search_all_common = 全部 (通用)
global_search_atlas = 图集 (Atlas)

unit_variant_name = 名称:
unit_variant_details_title = 详细信息
unit_variant_variants_title = 变体 (Variants)
unit_variant_filter = 过滤器
unit_variant_mesh_file = 网格文件:
unit_variant_texture_folder = 纹理文件夹:
unit_variant_unknown_value = 未知值:
unit_variant_id = Id:

unit_variant_new_category_title = 新建/克隆类别
about_check_empire_and_napoleon_ak_updates = 检查旧版 AK (帝国/拿破仑) 更新
update_old_ak_autogen_checker = 更新旧版 AK (帝国/拿破仑) 检查器
old_ak_no_update = <h4>没有可用的旧版 AK 更新</h4> <p>祝您下次好运 :)</p>
old_ak_new_update = <h4>有新的旧版 AK 更新可用</h4> <p>您想更新旧版 AK 数据吗？</p>
update_no_local_old_ak = <p>未找到本地旧版 AK (帝国/拿破仑) 数据。您想下载最新的数据吗？</p>
old_ak_update_success = <h4>旧版 AK 数据已更新。</h4><p>您现在可以继续使用 RPFM 了，但请记住为《帝国》和《拿破仑》重新生成依赖缓存，以便将旧版 AK 数据添加进去。</p>
portrait_settings_file_icon_label = 图标
settings_enable_icons = 启用图标

context_menu_patch_column = 修补列 (Patch Column)
new_column_patch_dialog = 列修补程序 (Column Patcher)
column_patch_instructions = <p>这允许您为当前选定的列制作补丁。每个选项的作用如下：</p>
    <ul>
        <li><b>作为键 (Is Key)</b>：该列将被视为键 (Key) 列。</li>
        <li><b>默认值 (Default Value)</b>：如果设置，创建新行时将使用此值作为默认值。</li>
        <li><b>是文件 (Is File)</b>：如果该列旨在引用文件（不是文件夹）（无论是其名称还是其完整路径），请勾选此项。</li>
        <li><b>文件相对路径 (File Relative Path)</b>：如果该列旨在引用文件（不是文件夹）且不包含完整路径，请将路径放在此处，并用 % 替换列中的任何内容。<br/>例如，如果列是没有扩展名的文件名，请输入 "path/to/file/%.extension"</li>
        <li><b>引用数据 (Reference Data)</b>：如果该列旨在引用另一数据表的列，请在此处输入不带 "_tables" 结尾的数据表名称和列名，用 ";" 分隔。<br/>例如，对于引用 "abilities" 数据表和 "key" 列的列，请输入 "abilities;key"。</li>
        <li><b>查找列 (Lookup Columns)</b>：列名列表，用 ";" 分隔，将在每个单元格中显示为查找。仅适用于引用其他数据表的列（有效值：其他数据表上的列或依赖于其他数据表的 loc 列），<br/>或单键表上的键列（有效值：此表上的列或依赖于此表的 loc 列）。</li>
        <li><b>不能为空 (Cannot Be Empty)</b>：如果空值应在诊断中被标记为错误。</li>
        <li><b>未使用 (Unused)</b>：该列在所选游戏中未使用，如果设置为在设置中隐藏，则可以自动隐藏。</li>
        <li><b>是数值 (Is Numeric Value)</b>：如果该列具有 I64、OptionalI64 或字符串类型之一，但被 I32 值引用（即除了 I32 数字之外的任何值都会引起问题），则应勾选此项。它将强制该列表现得像 I32 列。</li>
        <li><b>描述 (Description)</b>：可以添加到列描述中的文本，当您将鼠标悬停在列标题上时，它将显示在工具提示中。</li>
    </ul>

    <p>如果您认为某个补丁可以提高 RPFM 对所有人的可用性，请随时与工具作者分享，以便它可以在架构更新中分发。</p>
    <p>注意：指向同一数据表的非 loc 字段的查找将不会在更新其源值时更新其值。</p>

is_key = 作为键 (Is Key)
default_value = 默认值
is_filename = 是文件 (Is File)
filename_relative_path = 文件相对路径
is_reference = 引用数据 (Reference Data)
lookup = 查找列 (Lookup Columns)
not_empty = 不能为空
description = 描述
patch_success = 补丁保存成功。它将在您下次重启 RPFM 后生效。
column_tooltip_lookup_remote = 此列从以下数据表和列（或仅列，如果它们是与此表直接相关的 loc）获取查找值，或其 loc 值：
column_tooltip_lookup_local = 此列从以下此表的列获取查找值，或其 loc 值：

anim_fragment_version = 版本
anim_fragment_subversion = 子版本
anim_fragment_min_id = 最小 Id
anim_fragment_max_id = 最大 Id
anim_fragment_skeleton_name = 骨骼名称
anim_fragment_table_name = 数据表名称
anim_fragment_mount_table_name = 坐骑数据表名称
anim_fragment_unmount_table_name = 卸载坐骑数据表名称
anim_fragment_locomotion_graph = 运动图 (Locomotion Graph)
anim_fragment_is_simple_flight = 是简单飞行
anim_fragment_is_new_cavalry_tech = 是新骑兵技术

anim_fragment_battle_skeleton_name = 骨骼名称
anim_fragment_battle_table_name = 数据表名称
anim_fragment_battle_mount_table_name = 坐骑数据表名称
anim_fragment_battle_unmount_table_name = 卸载坐骑数据表名称
anim_fragment_battle_locomotion_graph = 运动图 (Locomotion Graph)
anim_fragment_battle_file_path = 文件路径
anim_fragment_battle_meta_file_path = Meta 文件路径
anim_fragment_battle_snd_file_path = Snd 文件路径
anim_fragment_battle_filename = 文件名
anim_fragment_battle_metadata = 元数据 (Metadata)
anim_fragment_battle_metadata_sound = 音频元数据
anim_fragment_battle_skeleton_type = 骨骼类型
anim_fragment_battle_uk_4 = Uk4 (未知4)

label_locomotion_graph_path_not_found = 未找到运动图
label_file_path_not_found = 未找到文件路径
label_meta_file_path_not_found = 未找到 Meta 文件路径
label_snd_file_path_not_found = 未找到 Snd 文件路径

tools_translator = 翻译器
translator_title = 翻译器
translator_info_title = 信息
translator_original_value_title = 原始值
translator_translated_value_title = 翻译值
translator_info = 它是如何工作的？
    很简单：<ul>
        <li>选择要翻译的行，然后在下方的文本字段中写下翻译。</li>
        <li>要保存一行的翻译，只需选择另一行进行翻译，或使用“上移”或“下移”（或 Ctrl+Up 和 Ctrl+Down）按钮。</li>
        <li>默认情况下，视图仅显示需要翻译或修改的行（因为原始 Mod 更改了它们）。您可以通过调整过滤器来显示隐藏的条目。</li>
        <li>如果要导入已翻译的 Mod，请在 RPFM 中打开原始 Mod，打开翻译器，点击“从已翻译的 Pack 导入 (Import from translated Pack)”，然后选择翻译 Pack。</li>
        <li>完成后，RPFM 会将翻译同时保存到打开的 Pack 中的 .loc 文件，以及“RPFM 配置文件夹/translations_local”上的 json 文件中。该文件用于在翻译更新时（比如原始 Mod 获得更新后）跟踪更改。</li>
        <li>完成后，如果您认为此翻译对其他人有用，请随时将翻译（即配置文件夹中的 json 文件）贡献给 <a href="https://github.com/Frodo45127/total_war_translation_hub">全面战争翻译中心 (Total War Translation Hub)</a>。</li>
    </ul>
translator_move_selection_up = 上移
translator_move_selection_down = 下移
translator_copy_from_source = 从来源复制
translator_import_from_translated_pack = 从已翻译的 Pack 导入
translator_language = 语言:

updater_title = 更新管理器
updater_info_title = 信息
updater_info = <p>这是 Rusted Packfile Manager 的中央更新管理器。每个按钮的含义：</p>
    <ul>
        <li>
            <b>程序更新 (Program Updates)</b>：程序本身的更新。更新后，您可以再次点击它以重新启动进入更新后的程序。
            关于这些更新的一些注意事项：<ul>
                <li>要查看更改，更新后您可以 <a href='file:///{"{"}{"}"}'>点击此处</a> 或者您可以打开 RPFM 文件夹中的 Changelog.md 文件。</li>
                <li>请注意，有两个更新通道：测试版 (beta) 和稳定版 (stable)。<b>您当前处于 {"{"}{"}"} 通道</b>。您可以在设置中更改通道。</li>
                <li>如果您选择“稳定版 (Stable)”通道而您使用的是测试版，则最新的稳定版将始终显示为可用更新，即使它比您的测试版还要旧。这是为了允许回滚。因此，如果您想使用测试版，请确保选择“测试版 (Beta)”通道。</li>
            </ul>
        </li>
        <li><b>架构更新 (Schema Updates)</b>：使 RPFM 能够打开数据表所需的文件。基本上，如果有这方面的更新，就下载它。</li>
        <li><b>Lua Autogen 更新</b>：这是用于 MyMod 和脚本编写的。在创建 MyMod 时，RPFM 可以自动为 VSCode 和 Sublime Text 创建项目。如果您使此项保持最新，RPFM 将配置所述项目，以便您可以直接在所述项目中获得 Lua 脚本的代码检查 (linting) 和自动完成功能。</li>
        <li><b>旧版 AK 更新 (Old AK Updates)</b>：这是为《帝国》和《拿破仑》Mod 开发者准备的。如果您为这些游戏制作 Mod，请下载它，然后为这两款游戏重新生成依赖缓存。RPFM 通常使用 Assembly Kit 作为依赖缓存的一部分，以实现……许多功能。《帝国》和《拿破仑》本身没有 AK，但 CA 很久以前向公众发布了它们的数据表。这会下载所述数据表，以便 RPFM 可以为它们“伪造”一个 Assembly Kit，并启用与其他游戏相同的功能。</li>
    </ul>

updater_update_schemas = 架构更新:
updater_update_program = 程序更新:
updater_update_twautogen = Lua Autogen 更新:
updater_update_old_ak = 旧版 AK 更新:
updater_update_schemas_checking = 正在检查，请稍候...
updater_update_program_checking = 正在检查，请稍候...
updater_update_twautogen_checking = 正在检查，请稍候...
updater_update_old_ak_checking = 正在检查，请稍候...

updater_update_program_available = 发现更新 {"{"}{"}"}！
updater_update_program_no_updates = 未发现更新。

updater_update_schemas_available = 发现更新！
updater_update_schemas_no_updates = 未发现更新。

updater_update_twautogen_available = 发现更新！
updater_update_twautogen_no_updates = 未发现更新。

updater_update_old_ak_available = 发现更新！
updater_update_old_ak_no_updates = 未发现更新。

updater_update_program_error = 更新 RPFM 时出错。
updater_update_program_updated = RPFM 已更新！点击此处重新启动。

updater_update_schemas_error = 更新架构时出错。
updater_update_schemas_updated = 架构已更新！
updater_update_twautogen_error = 更新 Lua Autogen 时出错。
updater_update_twautogen_updated = Lua Autogen 已更新！

updater_update_old_ak_error = 更新旧版 AK 时出错。
updater_update_old_ak_updated = 旧版 AK 已更新！

settings_check_old_ak_updates_on_start = 启动时检查旧版 AK（帝国与拿破仑）更新:

updater_update_program_updating = 正在更新，请稍候...
updater_update_schemas_updating = 正在更新，请稍候...
updater_update_twautogen_updating = 正在更新，请稍候...
updater_update_old_ak_updating = 正在更新，请稍候...
special_stuff_build_starpos = 构建 Startpos
games_closed = 当游戏关闭时点击此项
build_starpos = 构建 Startpos
build_starpos_instructions = <p>说明：</p>
    <ul>
        <li>这仅在 Windows 下经过测试。不知道在 Linux 构建上是否有效。
        <li>您的 Pack 中需要拥有所有相关的 star_pos_*** 数据表。如果您没有它们，请创建它们或从 Assembly Kit 中添加它们，然后重试此操作。</li>
        <li>您还需要在 Pack 中拥有 campaigns 数据表以及 "db/victory_objectives.txt" 文件。</li>
        <li>您需要将您的 Pack 保存在游戏的 /data 文件夹中。如果是新的 Pack 或不在 /data 中，请保存它然后重试此操作。</li>
        <li>您需要正确配置游戏文件夹。如果你没有...你知道该怎么做。</li>
        <li>选择您要为其构建 startpos 的战役。</li>
        <li>点击 <b>构建 Startpos (Build Startpos)</b> 按钮。如果没有出现故障，游戏将启动（您可能需要在启动器中点击“开始游戏”），然后在不久后自动关闭。</li>
        <li>游戏关闭后，点击 <b>当游戏关闭时点击此项 (Hit this when the game is closed)</b> 按钮。这就搞定了。之后，新的 startpos 应该会在您的 Pack 中了。</li>
        <li>只有当您要创建新的战役地图时，才启用 <b>处理 HLP 和 SPD 数据 (Process HLP and SPD data)</b>。该选项指示游戏的 exe 生成 hlp_data.esf 和 spd_data.esf 文件，</br>
            这些文件是战役地图寻路的一部分。<b>大约需要 10 到 30 分钟</b>。只有在制作新的战役地图时才需要执行此操作。</li>
    </ul>

    <p>确认有效的游戏：</p>
    <ul>
        <li>全面战争：法老 王朝 (Pharaoh Dynasties)。</li>
        <li>全面战争：法老 (Pharaoh)。</li>
        <li>全面战争：战锤 3 (Warhammer 3)。</li>
        <li>全面战争传奇：特洛伊 (Troy)。</li>
        <li>全面战争：三国 (Three Kingdoms)（这个会启动游戏，关闭它，然后再次启动它，然后再关闭它）。</li>
        <li>全面战争：战锤 2 (Warhammer 2)。</li>
        <li>全面战争传奇：不列颠王座 (Thrones of Britannia)。</li>
        <li>全面战争：阿提拉 (Attila)。</li>
        <li>全面战争：罗马 2 (Rome 2)。</li>
        <li>全面战争：幕府将军 2 (Shogun 2)。</li>
    </ul>
    <p>如果它不在此列表中，则表示尚未经过测试，可能有效……也可能无效，因此如果您使用它发现任何问题请报告。如果显示“部分”，这意味着我对其进行了一些有限的测试，它有效，但可能仍然包含错误，所以如果您使用它并且某些功能不起作用，请报告。</p>

campaign_id = 战役 ID (Campaign ID):
process_hlp_spd_data = 处理 HLP 和 SPD 数据
process_hlp_data = 处理 HLP 数据

ignore_game_files_in_ak = 忽略 Assembly Kit 中的游戏文件
settings_ignore_game_files_in_ak = 生成依赖缓存时，忽略游戏文件中已存在的 Assembly Kit 中的文件。可减少 RAM 使用量并使 RPFM 加载更快，但您会失去对大约 900 个文件的读取/导入权限。

settings_ui_table_use_old_column_order_for_tsv_label = 导出 TSV 时使用旧的列顺序（键优先）:

special_stuff_update_anim_ids = 更新动画 Ids (Anim Ids)
update_anim_ids = 更新动画 Ids
update_anim_ids_instructions = <p>这允许您通过从特定的 Id 开始对所有动画 Id 应用偏移量，来更新 Pack 中包含的所有 AnimFragment 的动画 Id（包括 AnimPacks 内部的）。</p>
    <p>偏移量可以是负数，以防您弄砸了需要减小它。正如我所说，这适用于所有文件，甚至包括 AnimPacks 内部的。</p>
    <p>如果您的包中只有少数几个需要更新的 AnimFragment，而其他已经更新过了，<b>请勿使用此功能，否则您将破坏已经更新的那些</b>。请使用“重写选中项 (Rewrite Selection)”手动更新它们。</p>


starting_id = 起始 ID:
offset = 偏移量:
instructions = 说明

enable_multifolder_filepicker = 启用多文件夹文件选择器
settings_enable_multifolder_filepicker = 这会将使用“从文件夹添加”时出现的系统文件选择器替换为通用的 Qt 文件选择器，该选择器允许一次导入多个文件夹。
settings_add_rpfm_to_runcher_tools = 将 RPFM 添加为 Runcher 中的工具
add_rpfm_to_runcher_tools_success = RPFM 已成功添加为 Runcher 中的工具。您可能需要重新启动 Runcher 才能找到添加的工具。
settings_paths_secondary = 辅助文件夹
settings_paths_secondary_ph = 您在 Runcher 或任何支持类似功能的启动器中设置的辅助文件夹。
open_from_secondary = 从辅助位置打开

diagnostics_hint = 将鼠标悬停在诊断信息上了解更多。
filter_variant_source = 来源
filter_variant_lookup = 查找
filter_variant_both = 两者

label_lua_invalid_key = Lua 脚本中无效的表格值
text_invalid_key_explanation = 在被标记为从数据表使用值的 lua 表格中，存在不在该数据表中的值。这意味着如果您假设这些值都存在于数据表中并最终使用它们，则脚本可能会崩溃。
translation_download_error = 尝试下载最新 mod 翻译时出错: {"{"}{"}"}。
reload_renderer = 重新加载 3D 渲染器
settings_enable_renderer = 启用 3D 渲染器
context_menu_revert_value = 将值恢复为原版

settings_enable_diff_markers = 启用差异标记 (Diff Markers)

enable_pack_contents_drag_and_drop = 启用 Pack 内容拖放:
settings_enable_pack_contents_drag_and_drop = 这允许您切换 Pack 内容树状视图的拖放 (Drag & Drop) 行为。
hide_unused_columns = 隐藏未使用的列:
settings_hide_unused_columns = 如果启用此项，RPFM 将自动隐藏在 Assembly Kit 中标记为“未使用 (unused)”的列。
unused = 未使用
patch_removed_table = 已删除此数据表的本地补丁。
patch_removed_column = 已删除此列的本地补丁。
remove_patches_for_table = 删除数据表的补丁
remove_patches_for_column = 删除列的补丁

label_missing_loc_data_file_detected = 检测到缺失的 Loc 数据文件
missing_loc_data_file_detected_explanation = 使用“生成缺失的 Loc 数据 (Generate Missing Loc Data)”功能生成的文件本来就不应该保留。
    为什么？因为 RPFM 将在您下次再次使用该功能时覆盖您对它们所做的任何更改。预期的工作流程是：

        - Mod 开发者制作数据表并填充它们。
        - Mod 开发者生成 locs。
        - Mod 开发者重命名 zzz_* 文件或将其内容移动到其他地方。
        - Mod 开发者将它需要编辑的原版现有值从 aaa_* 文件复制到自己的 loc 文件中。
        - Mod 制作继续，直到发布时间。
        - Mod 开发者转到 Special Stuff/游戏/优化 PackFile (Optimize PackFile) 并运行优化器，这应该会删除 aaa 文件（消除覆盖/未翻译问题）和 zzz（如果它是空的）。
        - Mod 开发者发布/更新 mod。

    您不一定非要专门使用该工作流程，但此警告将提醒您，在发布此 mod 之前应删除自动生成的 loc 文件。
compression_format = 压缩格式
compression_format_none = 无
compression_format_lzma1 = Lzma1
compression_format_lz4 = Lz4
compression_format_zstd = Zstd

behavior_title = 自动翻译行为
behavior_info = 当您选择新行时的行为：
behavior_chatgpt = 使用 ChatGPT 自动翻译（需要 API Key，中等质量）
behavior_google_translate = 使用 Google 翻译自动翻译（低质量）
behavior_copy_source = 复制来源值
behavior_empty = 清空翻译值

behavior_edit_info = 当您保存当前行时的行为：
behavior_edit_all_same_values = 在预先翻译的行中同步编辑
behavior_edit_only_this_value = 仅编辑当前行

label_invalid_file_name = 无效文件名
invalid_file_name_explanation = Windows 不允许文件名中包含某些符号。
    具体来说，不允许使用以下任何符号: <, >, :, ", /, \, |, ? 和 *。如果您的文件夹/文件名称包含这些符号，请重命名它们。

audio_wem_warning_label = <p><b style="color:red;">警告：音频播放器尚不支持 .wem 文件。</b></p>

optimizer_instructions_label = <h3>您确定要优化此 PackFile 吗？</h3>
    <p>
        对于那些不知道这是什么的人，根据您在此对话框右侧选择的选项，它会执行以下操作：
        <ul>
            <li><b>Pack:</b></li>
            <ul>
                <li><b>删除从父级/原版文件中直接复制且未做更改的文件。</b></li>
            </ul>
            <li><b>DB/Loc 文件:</b></li>
            <ul>
                <li><b>分析数据核心化 (datacored) 的数据表，并将其删除的键导入 twad_key_deletes（仅限战锤 3）。</b></li>
                <li><b>删除重复的条目</b>（除非数据表是 datacoring 或它们位于不同的文件中）。</li>
                <li><b>删除与原版或父文件未发生更改的条目</b>（除非数据表是 datacoring）。</li>
                <li><b>删除与默认值未发生更改的条目</b>（除非数据表是 datacoring）。</li>
                <li><b>删除空的 DB 或 Loc 数据表。</b></li>
            </ul>
            <li><b>文本文件:</b></li>
            <ul>
                <li><b>删除地图文件夹中未使用的 XML 文件</b>，这是 bob 导出地图 pack 时产生的副产品。</li>
                <li><b>删除 prefab 文件夹中未使用的 XML 文件</b>，这是 bob 导出 prefabs 时产生的副产品。</li>
                <li><b>删除 .agf 文件</b>，这是 bob 导出模型时产生的副产品。</li>
                <li><b>删除 .model_statistics 文件</b>，这是 bob 导出模型时产生的副产品。</li>
            </ul>
            <li><b>肖像设置文件:</b></li>
            <ul>
                <li><b>删除肖像设置文件中未使用/无效的变体和艺术集。</b></li>
                <li><b>删除肖像设置变体中空的掩码 (Masks)。</b></li>
                <li><b>删除空的肖像设置文件</b>（仅限 .bin 文件，保留 .xml 文件以防您使用 selfie）。</li>
                <li><b>删除空的肖像设置文件。</b></li>
            </ul>
        </ul>
        所以，您确定要这样做吗？
    </p>

translator_key = 键:
context = 上下文
settings_ai_title = AI 设置
settings_ai_openai_api_key = OpenAI API Key:
tt_ai_openai_api_key_tip = OpenAI API Key。您必须从 OpenAI 的网站获取。

translator_translate_with_chatgpt = 使用 ChatGPT 翻译
translator_translate_with_google = 使用 Google 翻译翻译

label_file_itm = 文件与父级/原版文件相同
label_file_overwrite = 文件覆盖了不同的父级/原版文件

file_itm_explanation = 此文件与父级或原版文件完全相同。这意味着除了使您的 Pack 变得更大之外，它几乎没有任何作用。
file_overwrite_explanation = 此文件覆盖了父级或原版文件。这不是问题，只是让您更轻松地知道哪些文件正在覆盖原版资产。
behavior_deepl = 使用 DeepL 自动翻译（需要 API Key，最高质量）
translator_translate_with_deepl = 使用 DeepL 翻译

settings_deepl_api_key = DeepL API Key:
tt_deepl_api_key_tip = DeepL API Key。您必须从 DeepL 获取，它是免费的。
is_numeric = 是数值

label_file_duplicated = 文件重复
file_duplicated_explanation = 游戏文件不区分大小写。这意味着如果您有两个名称相同但一个带有大写字符而另一个带有小写字符的文件，
    RPFM 会将它们视为两个不同的文件，但游戏会将它们视为同一个文件并且只会加载其中一个。
    此警告告诉您您的 pack 中存在两个或多个游戏将视为同一个文件的文件，并且只会加载其中一个。
context_menu_add_to_twad_key_deletes = 将选中项添加到键删除 (Key Deletes)
twad_key_deletes_warning = <p>此表是一个特殊的表，用于在运行时从任何其他表中删除行，而无需编辑所述表。此表的主要用例是避免数据核心化 (datacoring)，或者是为了从一个表中删除一行或多行而覆盖整个表的行为。</p>

    <p>为什么？因为 datacored 的表通常很难正确维护，如果两个或多个 mod 对同一个表执行此操作，可能会导致 mod 之间的兼容性问题。改用此表消除了所有这些兼容性问题，使其非常易于维护，并且作为奖励，它在此过程中使您的 mod 变得更小。</p>

    <p><b>关于此表的注意事项：</b></p>
    <ul>
        <li>建议您避免手动向此表添加条目，特别是当您想从多键表（例如联结表 junctions table）中删除键时。请使用以下方法之一：</li>
        <ul>
            <li>如果您的 mod 包含 datacored 表，请转到“Special Stuff/Warhammer 3/Optimize Pack”，然后确保勾选“DB: Import datacores into twad_key_deletes”复选框并点击接受。这将创建此表，并使用在您的 datacored 表中自动检测到的所有已删除行自动填充它。它不会删除 datacored 表，因此在对结果满意后，您必须手动删除这些表。</li>
            <li>如果您只想从表中删除几行，首先确保此表的实例存在于您的 pack 中。如果没有，右键点击打开的 pack，然后点击“新建/新建 DB (Create/Create DB)”并从那里创建它。然后打开您想要从中删除行的表，选择要删除的行，右键点击它们，然后点击“将选中项添加到键删除/yourtablename (Add Selection to Key Deletes/yourtablename)”。</li>
        </ul>
        <li>请注意，向此表中添加一个键相当于对相关表进行数据核心化 (datacoring) 并删除具有相同键的行。这意味着如果您添加了在其他表中被引用的键，最终可能会在运行时导致诊断工具无法检测到的崩溃（至少目前还不行）。请注意这一点。</li>
    </ul>
    <p>

optimizer_title = 优化器 (Optimizer)
optimizer_options_title = 选项
optimizer_pack_title = Pack
optimizer_table_title = DB/Loc 文件
optimizer_text_title = 文本文件
optimizer_pts_title = 肖像设置文件

optimizer_pack_remove_itm_files = 删除未更改的文件
optimizer_db_import_datacores_into_twad_key_deletes = 将 datacores 导入 twad_key_deletes
optimizer_db_optimize_datacored_tables = <b>不推荐:</b> 优化 datacored 表
optimizer_table_remove_duplicated_entries = 删除重复条目
optimizer_table_remove_itm_entries = 删除与原版/父文件未更改的条目
optimizer_table_remove_itnr_entries = 删除与默认值未更改的条目
optimizer_table_remove_empty_file = 删除空文件
optimizer_text_remove_unused_xml_map_folders = 删除地图文件夹中未使用的 XML 文件
optimizer_text_remove_unused_xml_prefab_folder = 删除 prefab 文件夹中未使用的 XML 文件
optimizer_text_remove_agf_files = 删除 .agf 文件
optimizer_text_remove_model_statistics_files = 删除 .model_statistics 文件
optimizer_pts_remove_unused_art_sets = 删除未使用的艺术集 (Art Sets)
optimizer_pts_remove_unused_variants = 删除未使用的变体 (Variants)
optimizer_pts_remove_empty_masks = 删除空的遮罩 (Masks)
optimizer_pts_remove_empty_file = 删除空文件

label_altered_table = 更改的表 (Altered Table)
altered_table_explanation = 当打开 Pack 并且一个或多个单元格的数据发生更改时，此表已被 RPFM 更改。
    为什么会这样？从 4.6.0 开始，RPFM 将某些游戏仅期望数字的字符串字段视为数字字段。
    如果您的包在其中一个字段中有数字以外的任何数据，RPFM 会将该数据更改为 <b><i>87654321</i></b>，您将收到此错误。
    该怎么做？打开出现此错误的表，搜索所有值为 <b><i>87654321</i></b> 的位置，并赋予它们您选择的正确数值。
    注意：仅在修复了所有有问题的单元格并重新打开包后，此错误才会消失。
    注意 2：如果在选择错误游戏的情况下打开了 pack，这也可能发生，从而导致某些表使用了不正确的定义，进而导致某些字段被视为数字（而它们不应该）。
    在这种情况下，请勿保存该 Pack。将选定的游戏切换到该 pack 对应的游戏，然后再次打开该 pack，错误应该会消失。

rigid_model_editor_detailed_view_title = LOD 数据
rigid_model_editor_mesh_block_title = 网格块数据 (Mesh Block Data)
rigid_model_editor_material_data_title = 材质数据 (Material Data)
rigid_model_editor_mesh_data_title = 网格数据 (Mesh Data)
rigid_model_editor_visibility = 可见距离:
rigid_model_editor_lod_number = LOD 编号:
rigid_model_editor_quality_level = 质量等级:

rigid_model_skeleton_id = 骨骼 ID
rigid_model_mesh_name = 网格名称
rigid_model_mesh_mat_name = 材质名称
rigid_model_texture_directory = 纹理目录
rigid_model_filters = 过滤器
rigid_model_att_point_name = 附着点名称
rigid_model_text_path = 纹理路径

rigid_model_editor_version = 版本:
rigid_model_editor_mesh_name = 网格名称:
rigid_model_editor_texture_folder = 纹理文件夹:
rigid_model_editor_shader_name = 着色器名称:
rigid_model_editor_rmv_title = RigidModel 数据
rigid_model_editor_texture_list_title = 纹理列表

rigid_model_editor_export_to_gltf = 导出为 GLTF
extract_gltf = 导出 GLTF 文件
settings_use_debug_view_unit_variant = 为单位变体 (Unit Variants) 使用调试视图:
//---------------------------------------------------------------------------//
// Copyright (c) 2017-2026 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! Module with all the code related to the `WelcomePageUI`.

use qt_widgets::QLabel;
use qt_widgets::QPushButton;
use qt_widgets::QScrollArea;
use qt_widgets::QWidget;

use qt_gui::QIcon;

use qt_core::AlignmentFlag;
use qt_core::QBox;
use qt_core::QFlags;
use qt_core::QString;

use getset::Getters;

use std::path::PathBuf;
use std::rc::Rc;

use rpfm_lib::games::supported_games::*;

use rpfm_ui_common::ASSETS_PATH;
use rpfm_ui_common::clone;
use rpfm_ui_common::utils::create_grid_layout;

use crate::app_ui::AppUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::dependencies_ui::DependenciesUI;
use crate::GAME_SELECTED;
use crate::GAME_SELECTED_ICONS;
use crate::global_search_ui::GlobalSearchUI;
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::settings_ui::backend::*;
use crate::utils::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access the widgets in the Welcome Page.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct WelcomePageUI {
    welcome_widget: QBox<QWidget>,
    logo_label: QBox<QLabel>,
    recent_files_widget: QBox<QWidget>,
    new_pack_button: QBox<QPushButton>,
    open_pack_button: QBox<QPushButton>,
    new_mymod_button: QBox<QPushButton>,
    settings_button: QBox<QPushButton>,
    github_button: QBox<QPushButton>,
    manual_button: QBox<QPushButton>,
    discord_button: QBox<QPushButton>,
    patreon_button: QBox<QPushButton>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

impl WelcomePageUI {

    /// This function creates the entire `WelcomePageUI`.
    pub unsafe fn new(parent: &QBox<QWidget>) -> Self {
        let welcome_widget = QWidget::new_1a(parent);
        let welcome_layout = create_grid_layout(welcome_widget.static_upcast());

        // Use a scroll area so the content is never clipped on small windows.
        let scroll_area = QScrollArea::new_1a(&welcome_widget);
        scroll_area.set_widget_resizable(true);
        scroll_area.set_frame_shape(qt_widgets::q_frame::Shape::NoFrame);
        welcome_layout.add_widget_5a(&scroll_area, 0, 0, 1, 1);

        let scroll_content = QWidget::new_0a();
        let outer_layout = create_grid_layout(scroll_content.static_upcast());
        outer_layout.set_column_stretch(0, 1);
        outer_layout.set_column_stretch(2, 1);
        outer_layout.set_row_stretch(0, 1);
        outer_layout.set_row_stretch(2, 1);
        scroll_area.set_widget(&scroll_content);

        // Center column container.
        let center_widget = QWidget::new_1a(&scroll_content);
        let center_layout = create_grid_layout(center_widget.static_upcast());
        center_layout.set_spacing(10);
        outer_layout.add_widget_5a(&center_widget, 1, 1, 1, 1);

        // Two-column layout: left = actions, right = recent files.
        let columns_widget = QWidget::new_1a(&center_widget);
        let columns_layout = create_grid_layout(columns_widget.static_upcast());
        columns_layout.set_spacing(30);

        // ---- Header: logo + title + version ----
        // The logo pixmap is set dynamically by build_logo() to show a composite
        // of the RPFM icon and the currently selected game icon.
        let logo_label = QLabel::from_q_string_q_widget(&QString::new(), &center_widget);
        logo_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        center_layout.add_widget_5a(&logo_label, 0, 0, 1, 1);

        let title_label = QLabel::from_q_string_q_widget(
            &QString::from_std_str(format!(
                "<h2>Rusted PackFile Manager</h2><p style='color: gray;'>v{}</p>",
                env!("CARGO_PKG_VERSION")
            )),
            &center_widget,
        );
        title_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        center_layout.add_widget_5a(&title_label, 1, 0, 1, 1);

        // ---- Columns area ----
        center_layout.add_widget_5a(&columns_widget, 2, 0, 1, 1);

        // ==== Left column: Quick Actions ====
        let left_widget = QWidget::new_1a(&columns_widget);
        let left_layout = create_grid_layout(left_widget.static_upcast());
        left_layout.set_spacing(6);
        columns_layout.add_widget_5a(&left_widget, 0, 0, 1, 1);

        let actions_label = QLabel::from_q_string_q_widget(
            &qtr("welcome_quick_actions"),
            &left_widget,
        );
        actions_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        left_layout.add_widget_5a(&actions_label, 0, 0, 1, 1);

        let new_pack_button = QPushButton::from_q_string_q_widget(&qtr("welcome_new_pack"), &left_widget);
        new_pack_button.set_minimum_height(32);
        left_layout.add_widget_5a(&new_pack_button, 1, 0, 1, 1);

        let open_pack_button = QPushButton::from_q_string_q_widget(&qtr("welcome_open_pack"), &left_widget);
        open_pack_button.set_minimum_height(32);
        left_layout.add_widget_5a(&open_pack_button, 2, 0, 1, 1);

        let new_mymod_button = QPushButton::from_q_string_q_widget(&qtr("welcome_new_mymod"), &left_widget);
        new_mymod_button.set_minimum_height(32);
        left_layout.add_widget_5a(&new_mymod_button, 3, 0, 1, 1);

        let settings_button = QPushButton::from_q_string_q_widget(&qtr("welcome_settings"), &left_widget);
        settings_button.set_minimum_height(32);
        left_layout.add_widget_5a(&settings_button, 4, 0, 1, 1);

        // Push buttons to top.
        left_layout.set_row_stretch(5, 1);

        // ==== Right column: Recent Files (populated later) ====
        let right_widget = QWidget::new_1a(&columns_widget);
        let right_layout = create_grid_layout(right_widget.static_upcast());
        right_layout.set_spacing(6);
        columns_layout.add_widget_5a(&right_widget, 0, 1, 1, 1);

        let recent_label = QLabel::from_q_string_q_widget(
            &qtr("welcome_recent_files"),
            &right_widget,
        );
        recent_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        right_layout.add_widget_5a(&recent_label, 0, 0, 1, 1);

        // Container that will be cleared and rebuilt with recent file buttons.
        let recent_files_widget = QWidget::new_1a(&right_widget);
        create_grid_layout(recent_files_widget.static_upcast());
        right_layout.add_widget_5a(&recent_files_widget, 1, 0, 1, 1);

        // Placeholder shown when there are no recent files.
        let no_recent_label = QLabel::from_q_string_q_widget(
            &qtr("welcome_no_recent_files"),
            &recent_files_widget,
        );
        no_recent_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        no_recent_label.set_enabled(false);
        recent_files_widget.layout().add_widget(&no_recent_label);

        // Push to top.
        right_layout.set_row_stretch(2, 1);

        // ---- Command palette tip ----
        let tip_label = QLabel::from_q_string_q_widget(
            &qtr("welcome_command_palette_tip"),
            &center_widget,
        );
        tip_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
        tip_label.set_word_wrap(true);
        tip_label.set_enabled(false);
        center_layout.add_widget_5a(&tip_label, 3, 0, 1, 1);

        // ---- Links row at the bottom ----
        let links_widget = QWidget::new_1a(&center_widget);
        let links_layout = create_grid_layout(links_widget.static_upcast());
        links_layout.set_spacing(8);
        links_layout.set_column_stretch(0, 1);
        links_layout.set_column_stretch(5, 1);
        center_layout.add_widget_5a(&links_widget, 4, 0, 1, 1);

        let github_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("GitHub"), &links_widget);
        github_button.set_flat(true);
        github_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/github.svg", ASSETS_PATH.to_string_lossy()))));
        links_layout.add_widget_5a(&github_button, 0, 1, 1, 1);

        let manual_button = QPushButton::from_q_string_q_widget(&qtr("welcome_manual"), &links_widget);
        manual_button.set_flat(true);
        manual_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/manual_icon.png", ASSETS_PATH.to_string_lossy()))));
        links_layout.add_widget_5a(&manual_button, 0, 2, 1, 1);

        let discord_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Discord"), &links_widget);
        discord_button.set_flat(true);
        discord_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/discord.svg", ASSETS_PATH.to_string_lossy()))));
        links_layout.add_widget_5a(&discord_button, 0, 3, 1, 1);

        let patreon_button = QPushButton::from_q_string_q_widget(&QString::from_std_str("Patreon"), &links_widget);
        patreon_button.set_flat(true);
        patreon_button.set_icon(&QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/patreon.png", ASSETS_PATH.to_string_lossy()))));
        links_layout.add_widget_5a(&patreon_button, 0, 4, 1, 1);

        Self {
            welcome_widget,
            logo_label,
            recent_files_widget,
            new_pack_button,
            open_pack_button,
            new_mymod_button,
            settings_button,
            github_button,
            manual_button,
            discord_button,
            patreon_button,
        }
    }

    /// This function builds a composite icon: left half is the RPFM icon, right half is the
    /// game-selected icon, split diagonally.
    pub unsafe fn build_logo(&self) {
        let size = 96;
        let rpfm_path = format!("{}/icons/rpfm.png", ASSETS_PATH.to_string_lossy());

        let (_, game_icon_path) = match GAME_SELECTED.read().unwrap().key() {
            KEY_PHARAOH_DYNASTIES => &GAME_SELECTED_ICONS.pharaoh_dynasties,
            KEY_PHARAOH => &GAME_SELECTED_ICONS.pharaoh,
            KEY_WARHAMMER_3 => &GAME_SELECTED_ICONS.warhammer_3,
            KEY_TROY => &GAME_SELECTED_ICONS.troy,
            KEY_THREE_KINGDOMS => &GAME_SELECTED_ICONS.three_kingdoms,
            KEY_WARHAMMER_2 => &GAME_SELECTED_ICONS.warhammer_2,
            KEY_WARHAMMER => &GAME_SELECTED_ICONS.warhammer,
            KEY_THRONES_OF_BRITANNIA => &GAME_SELECTED_ICONS.thrones_of_britannia,
            KEY_ATTILA => &GAME_SELECTED_ICONS.attila,
            KEY_ROME_2 => &GAME_SELECTED_ICONS.rome_2,
            KEY_SHOGUN_2 => &GAME_SELECTED_ICONS.shogun_2,
            KEY_NAPOLEON => &GAME_SELECTED_ICONS.napoleon,
            KEY_EMPIRE => &GAME_SELECTED_ICONS.empire,
            KEY_ARENA => &GAME_SELECTED_ICONS.arena,
            _ => return,
        };

        // Load both icons and scale them.
        let rpfm_pixmap = qt_gui::QPixmap::new();
        rpfm_pixmap.load_1a(&QString::from_std_str(&rpfm_path));
        let rpfm_scaled = rpfm_pixmap.scaled_2_int(size, size);

        let game_pixmap = qt_gui::QPixmap::new();
        let game_icon_path_fixed = if cfg!(target_os = "windows") { game_icon_path.replace('\\', "/") } else { game_icon_path.clone() };
        game_pixmap.load_1a(&QString::from_std_str(&game_icon_path_fixed));
        let game_scaled = game_pixmap.scaled_2_int(size, size);

        // Create the composite pixmap.
        let result = qt_gui::QPixmap::from_2_int(size, size);
        result.fill_1a(&qt_gui::QColor::from_4_int(0, 0, 0, 0));

        let painter = qt_gui::QPainter::new_1a(&result);
        painter.set_render_hint_1a(qt_gui::q_painter::RenderHint::Antialiasing);
        painter.set_render_hint_1a(qt_gui::q_painter::RenderHint::SmoothPixmapTransform);

        // Left half: RPFM icon, clipped by a diagonal polygon (top-left triangle + a bit more).
        // The diagonal goes from ~60% across the top to ~40% across the bottom.
        let left_clip = qt_gui::QPainterPath::new_0a();
        let s = size as f64;
        left_clip.move_to_2a(0.0, 0.0);
        left_clip.line_to_2a(s * 0.6, 0.0);
        left_clip.line_to_2a(s * 0.4, s);
        left_clip.line_to_2a(0.0, s);
        left_clip.close_subpath();

        painter.save();
        painter.set_clip_path_1a(&left_clip);
        painter.draw_pixmap_q_point_q_pixmap(&qt_core::QPoint::new_2a(0, 0), &rpfm_scaled);
        painter.restore();

        // Right half: game icon, clipped by the complementary polygon.
        let right_clip = qt_gui::QPainterPath::new_0a();
        right_clip.move_to_2a(s * 0.6, 0.0);
        right_clip.line_to_2a(s, 0.0);
        right_clip.line_to_2a(s, s);
        right_clip.line_to_2a(s * 0.4, s);
        right_clip.close_subpath();

        painter.save();
        painter.set_clip_path_1a(&right_clip);
        painter.draw_pixmap_q_point_q_pixmap(&qt_core::QPoint::new_2a(0, 0), &game_scaled);
        painter.restore();

        // Draw a thin diagonal separator line.
        let pen = qt_gui::QPen::from_q_color(&qt_gui::QColor::from_4_int(128, 128, 128, 160));
        pen.set_width(2);
        painter.set_pen_q_pen(&pen);
        painter.draw_line_4_int((s * 0.6) as i32, 0, (s * 0.4) as i32, size);

        drop(painter);

        self.logo_label.set_pixmap(&result);
    }

    /// This function rebuilds the recent files list in the welcome widget.
    pub unsafe fn build_recent_files(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
    ) {
        let container = &app_ui.welcome_page_ui().recent_files_widget;
        let layout = container.layout();

        // Clear all children.
        while layout.count() > 0 {
            let item = layout.take_at(0);
            if !item.is_null() {
                let widget = item.widget();
                if !widget.is_null() {
                    widget.delete_later();
                }
            }
        }

        let recent_file_paths = settings_vec_string("recentFileList");
        if recent_file_paths.is_empty() {
            let no_recent_label = QLabel::from_q_string_q_widget(
                &qtr("welcome_no_recent_files"),
                container,
            );
            no_recent_label.set_alignment(QFlags::from(AlignmentFlag::AlignCenter));
            no_recent_label.set_enabled(false);
            layout.add_widget(&no_recent_label);
        } else {
            for path_str in recent_file_paths.iter().take(8) {
                let path = PathBuf::from(path_str);
                if path.is_file() {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let btn = QPushButton::from_q_string_q_widget(
                        &QString::from_std_str(&file_name),
                        container,
                    );
                    btn.set_minimum_height(28);
                    btn.set_flat(true);
                    btn.set_tool_tip(&QString::from_std_str(path_str));
                    layout.add_widget(&btn);

                    let slot = qt_core::SlotOfBool::new(&btn, clone!(
                        app_ui,
                        pack_file_contents_ui,
                        dependencies_ui,
                        global_search_ui,
                        diagnostics_ui,
                        path => move |_| {
                        if AppUI::are_you_sure(&app_ui, false, false) {
                            if let Err(error) = AppUI::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &dependencies_ui, &[path.to_path_buf()], "", false) {
                                return show_dialog(app_ui.main_window(), error, false);
                            }

                            if settings_bool("diagnostics_trigger_on_open") {
                                app_ui.menu_bar_packfile().set_enabled(false);
                                DiagnosticsUI::check(&app_ui, &diagnostics_ui);
                            }
                        }
                    }));
                    btn.clicked().connect(&slot);
                }
            }
        }
    }

    /// This function toggles visibility between the welcome widget and the tab widget.
    pub unsafe fn toggle_visibility(&self, tab_bar: &QBox<qt_widgets::QTabWidget>) {
        if tab_bar.count() == 0 {
            tab_bar.hide();
            self.build_logo();
            self.welcome_widget.show();
        } else {
            self.welcome_widget.hide();
            tab_bar.show();
        }
    }
}

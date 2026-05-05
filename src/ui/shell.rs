use super::absences::render_absences;
use super::shared::tab_span;
use super::theme::{self, BRAND, DIM_GRAY};
use super::timetable::{render_timetable, render_timetable_search_popup};
use crate::app::state::AppState;
use crate::shortcuts::{TabId, get_shortcut_sections};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use tui_components::ui::settings::{SettingsItemView, SettingsModal, SettingsSectionView};
use tui_components::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellClickTarget {
    Tab(TabId),
}

pub(super) fn render_main(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let help_text = if state.is_demo_mode() {
        "Demo Mode · Press ? for settings"
    } else {
        "Press ? for settings"
    };
    let tabs_width = (" Timetable ".len() + " Absences ".len()) as u16;
    let help_width = help_text.len() as u16;
    let header_area = Rect {
        height: 1,
        ..layout[0]
    };

    if state.main.active_tab == TabId::Timetable {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                state.timetable_target_label(),
                Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            header_area,
        );
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            tab_span(" Timetable ", state.main.active_tab == TabId::Timetable),
            tab_span(" Absences ", state.main.active_tab == TabId::Absences),
        ])),
        Rect {
            width: tabs_width,
            ..header_area
        },
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            help_text,
            Style::default().fg(DIM_GRAY),
        )))
        .alignment(Alignment::Right),
        Rect {
            x: header_area.x + header_area.width.saturating_sub(help_width),
            width: help_width,
            ..header_area
        },
    );

    match state.main.active_tab {
        TabId::Timetable => render_timetable(frame, state, layout[1]),
        TabId::Absences => render_absences(frame, state, layout[1]),
    }

    if state.main.settings_open {
        render_shortcuts_modal(frame, state, area);
    }
    if state.main.timetable.search_open {
        render_timetable_search_popup(frame, state, area);
    }
}

pub(crate) fn hit_test_shell_click(column: u16, row: u16) -> Option<ShellClickTarget> {
    if row != 0 {
        return None;
    }

    let timetable_width = " Timetable ".len() as u16;
    let absences_width = " Absences ".len() as u16;
    let tabs_width = timetable_width + absences_width;

    if column >= tabs_width {
        return None;
    }

    if column < timetable_width {
        Some(ShellClickTarget::Tab(TabId::Timetable))
    } else {
        Some(ShellClickTarget::Tab(TabId::Absences))
    }
}

fn render_shortcuts_modal(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let sections = get_shortcut_sections(state.main.active_tab)
        .into_iter()
        .map(|section| SettingsSectionView {
            title: section.title.to_owned(),
            items: section
                .items
                .into_iter()
                .map(|item| SettingsItemView {
                    keys: item.keys.to_owned(),
                    action: item.action.to_owned(),
                })
                .collect(),
        })
        .collect();
    let mut modal = SettingsModal::new("Keyboard shortcuts", sections);
    modal.scroll = state.main.settings_scroll;
    modal.key_width = 18;
    modal.render(frame, area, app_theme());
}

fn app_theme() -> Theme {
    Theme {
        brand: theme::BRAND,
        warning: theme::WARNING,
        error: theme::ERROR,
        success: theme::INFO,
        neutral_white: theme::BRIGHT_WHITE,
        neutral_black: theme::BLACK,
        neutral_gray: theme::DIM_GRAY,
        neutral_bright_black: theme::BORDER_GRAY,
        panel_header: theme::HEADER_BG,
        panel_selected: theme::SELECT_BG,
        panel_alternate: theme::ALT_BG,
    }
}

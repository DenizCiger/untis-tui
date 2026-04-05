use super::absences::render_absences;
use super::shared::{centered_rect, fit_text, tab_span};
use super::theme::BRAND;
use super::timetable::{render_timetable, render_timetable_search_popup};
use crate::app::state::AppState;
use crate::shortcuts::{TabId, get_shortcut_sections};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub(super) fn render_main(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header = Line::from(vec![
        tab_span(" Timetable ", state.main.active_tab == TabId::Timetable),
        Span::raw(" "),
        tab_span(" Absences ", state.main.active_tab == TabId::Absences),
        Span::raw("   "),
        Span::styled(
            if state.main.active_tab == TabId::Timetable {
                state.timetable_target_label()
            } else {
                "Press ? for settings".to_owned()
            },
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(vec![header]), layout[0]);

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

fn render_shortcuts_modal(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let popup = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup);
    let inner = Block::default()
        .title("Settings")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRAND))
        .inner(popup);
    frame.render_widget(
        Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND)),
        popup,
    );
    let sections = get_shortcut_sections(state.main.active_tab);
    let mut lines = Vec::new();
    for section in sections {
        lines.push(Line::from(Span::styled(
            section.title,
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for item in section.items {
            lines.push(Line::from(format!(
                "{} - {}",
                fit_text(item.keys, 18),
                item.action
            )));
        }
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

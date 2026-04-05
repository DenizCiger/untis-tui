mod absences;
mod login;
mod shared;
mod shell;
#[cfg(test)]
mod tests;
mod theme;
mod timetable;

use crate::app::state::{AppState, Screen};
use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

#[cfg(test)]
pub(crate) use absences::absence_layout_geometry;
pub(crate) use absences::hit_test_absence_history_click;
pub(crate) use shell::{ShellClickTarget, hit_test_shell_click};
pub(crate) use timetable::{TimetableTitleClickTarget, hit_test_timetable_title_click};

pub fn render(frame: &mut Frame, state: &AppState) {
    match state.screen {
        Screen::Loading => render_loading(frame),
        Screen::Login => login::render_login(frame, state),
        Screen::MainShell => shell::render_main(frame, state),
    }
}

fn render_loading(frame: &mut Frame) {
    let area = frame.area();
    let paragraph = Paragraph::new("Loading...")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, shared::centered_rect(40, 20, area));
}

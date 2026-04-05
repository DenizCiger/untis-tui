use super::render;
use crate::app::state::{AppState, Screen};
use crate::models::{Config, ParsedAbsence, SavedConfig};
use crate::shortcuts::TabId;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

fn sample_absence() -> ParsedAbsence {
    ParsedAbsence {
        id: 1,
        student_name: "User".into(),
        reason: "Ill".into(),
        text: "Doctor".into(),
        excuse_status: "Excused".into(),
        is_excused: true,
        start_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
        start_time: "08:00".into(),
        end_time: "08:50".into(),
    }
}

#[test]
fn render_login_screen_shows_title() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::Login;
    state.saved_config = Some(SavedConfig {
        school: "school".into(),
        username: "user".into(),
        server: "mese.webuntis.com".into(),
    });
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("WebUntis TUI - Login"));
}

#[test]
fn render_main_shell_shows_tabs() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.config = Some(Config {
        school: "school".into(),
        username: "user".into(),
        password: "secret".into(),
        server: "mese.webuntis.com".into(),
    });
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("Timetable"));
    assert!(output.contains("Absences"));
}

#[test]
fn render_timetable_uses_shared_header_without_extra_timetable_block() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert_eq!(output.matches("Timetable").count(), 1);
}

#[test]
fn render_absences_uses_shared_header_without_extra_absences_block() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert_eq!(output.matches("Absences").count(), 1);
    assert!(output.contains("Absence Timeline"));
}

#[test]
fn render_absences_shows_loading_state() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.main.absences.loading_initial = true;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("Loading absences..."));
}

#[test]
fn render_absences_shows_backend_error_when_history_is_empty() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.main.absences.loading_initial = false;
    state.main.absences.error = "Failed to fetch absences".into();
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("Failed to fetch absences"));
}

#[test]
fn render_absences_shows_neutral_empty_message_without_filters() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.main.absences.loading_initial = false;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("No absences found in loaded history."));
}

#[test]
fn render_absences_shows_filter_empty_message_when_records_are_filtered_out() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.main.absences.loading_initial = false;
    state.main.absences.search_query = "zzz".into();
    state.main.absences.absences.push(sample_absence());
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("No absences match current filters."));
}

#[test]
fn render_absences_shows_history_and_details_when_records_exist() {
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.main.absences.loading_initial = false;
    state.main.absences.absences.push(sample_absence());
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("History"));
    assert!(output.contains("1/1"));
    assert!(output.contains("Summary"));
    assert!(output.contains("Details"));
    assert!(output.contains("When"));
    assert!(output.contains("Reason"));
    assert!(output.contains("Excuse status"));
    assert!(output.contains("Mar 26, 2026 08:00-08:50"));
}

#[test]
fn render_absences_wide_layout_shows_summary_and_profile_header() {
    let backend = TestBackend::new(140, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.config = Some(Config {
        school: "school".into(),
        username: "user".into(),
        password: "secret".into(),
        server: "mese.webuntis.com".into(),
    });
    state.main.absences.absences.push(sample_absence());
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("Absence Timeline"));
    assert!(output.contains("user@school"));
    assert!(output.contains("Newest first | Showing 1"));
    assert!(output.contains("Summary"));
    assert!(output.contains("Loaded range: Mar 26, 2026 -> Mar 26, 2026"));
}

#[test]
fn render_absences_narrow_layout_stacks_and_uses_compact_filter_header() {
    let backend = TestBackend::new(90, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.active_tab = TabId::Absences;
    state.main.absences.absences.push(sample_absence());
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("[f:All] [w:All time] [/:none] [c]"));
    assert!(output.contains("26.03"));
    assert!(output.contains("Summary"));
    assert!(output.contains("Details"));
}

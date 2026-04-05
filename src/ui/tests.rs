use super::render;
use crate::app::state::{AppState, Screen};
use crate::models::{
    Config, DayTimetable, ParsedAbsence, ParsedLesson, SavedConfig, TimeUnit, WeekTimetable,
};
use crate::shortcuts::TabId;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

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

fn sample_lesson(
    instance_id: &str,
    subject: &str,
    start_time: &str,
    end_time: &str,
) -> ParsedLesson {
    ParsedLesson {
        instance_id: instance_id.into(),
        subject: subject.into(),
        subject_long_name: format!("{subject} long"),
        lesson_text: "Bring notes".into(),
        cell_state: String::new(),
        teacher: "T".into(),
        teacher_long_name: "Teacher".into(),
        all_teachers: vec!["T".into()],
        all_teacher_long_names: vec!["Teacher".into()],
        room: "A1".into(),
        room_long_name: "Room A1".into(),
        all_classes: vec!["1A".into()],
        start_time: start_time.into(),
        end_time: end_time.into(),
        cancelled: false,
        substitution: false,
        remarks: "Remark".into(),
    }
}

fn sample_timetable(period_count: usize, overlapping: bool) -> WeekTimetable {
    let monday = chrono::NaiveDate::from_ymd_opt(2026, 4, 6).unwrap();
    let timegrid = (0..period_count)
        .map(|index| {
            let start_minutes = 8 * 60 + index as i32 * 50;
            let end_minutes = start_minutes + 50;
            TimeUnit {
                name: (index + 1).to_string(),
                start_time: format!("{:02}:{:02}", start_minutes / 60, start_minutes % 60),
                end_time: format!("{:02}:{:02}", end_minutes / 60, end_minutes % 60),
            }
        })
        .collect::<Vec<_>>();
    let mut monday_lessons = timegrid
        .iter()
        .enumerate()
        .map(|(index, period)| {
            sample_lesson(
                &format!("lesson-{index}"),
                &format!("S{index}"),
                &period.start_time,
                &period.end_time,
            )
        })
        .collect::<Vec<_>>();

    if overlapping {
        monday_lessons.push(sample_lesson("overlap-a", "M", "08:00", "09:40"));
        monday_lessons.push(sample_lesson("overlap-b", "E", "08:00", "08:50"));
        monday_lessons.push(sample_lesson("overlap-c", "B", "08:50", "09:40"));
    }

    WeekTimetable {
        days: vec![
            DayTimetable {
                date: monday,
                day_name: "Monday".into(),
                lessons: monday_lessons,
            },
            DayTimetable {
                date: monday.succ_opt().unwrap(),
                day_name: "Tuesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: monday.succ_opt().unwrap().succ_opt().unwrap(),
                day_name: "Wednesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: monday
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap(),
                day_name: "Thursday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: monday
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap(),
                day_name: "Friday".into(),
                lessons: Vec::new(),
            },
        ],
        timegrid,
    }
}

fn overlap_timetable() -> WeekTimetable {
    let monday = chrono::NaiveDate::from_ymd_opt(2026, 4, 6).unwrap();
    WeekTimetable {
        days: vec![
            DayTimetable {
                date: monday,
                day_name: "Monday".into(),
                lessons: vec![
                    sample_lesson("overlap-a", "M", "08:00", "09:40"),
                    sample_lesson("overlap-b", "E", "08:00", "08:50"),
                    sample_lesson("overlap-c", "B", "08:50", "09:40"),
                ],
            },
            DayTimetable {
                date: monday.succ_opt().unwrap(),
                day_name: "Tuesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: monday.succ_opt().unwrap().succ_opt().unwrap(),
                day_name: "Wednesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: monday
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap(),
                day_name: "Thursday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: monday
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap()
                    .succ_opt()
                    .unwrap(),
                day_name: "Friday".into(),
                lessons: Vec::new(),
            },
        ],
        timegrid: vec![
            TimeUnit {
                name: "1".into(),
                start_time: "08:00".into(),
                end_time: "08:50".into(),
            },
            TimeUnit {
                name: "2".into(),
                start_time: "08:50".into(),
                end_time: "09:40".into(),
            },
            TimeUnit {
                name: "3".into(),
                start_time: "09:40".into(),
                end_time: "10:30".into(),
            },
            TimeUnit {
                name: "4".into(),
                start_time: "10:30".into(),
                end_time: "11:20".into(),
            },
        ],
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

#[test]
fn render_timetable_wide_layout_shows_split_cells_and_details() {
    let backend = TestBackend::new(140, 32);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.timetable.data = Some(overlap_timetable());
    state.main.timetable.selected_day_idx = 0;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("▍M"));
    assert!(output.contains("▍E"));
    assert!(output.contains("Details"));
    assert!(output.contains("Teachers:"));
}

#[test]
fn render_timetable_narrow_layout_uses_overlap_preview() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.timetable.data = Some(overlap_timetable());
    state.main.timetable.selected_day_idx = 0;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("E +1"));
    assert!(output.contains("Details"));
}

#[test]
fn render_timetable_shows_scroll_hints_when_grid_is_scrolled() {
    let backend = TestBackend::new(120, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.timetable.data = Some(sample_timetable(8, false));
    state.main.timetable.selected_period_idx = 4;
    state.main.timetable.scroll_offset = 2;
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let output = buffer_text(terminal.backend().buffer());
    assert!(output.contains("▲ 2 more ▲"));
    assert!(output.contains("▼ 3 more ▼"));
}

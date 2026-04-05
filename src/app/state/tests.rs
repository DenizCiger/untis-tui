use super::*;
use crate::models::{
    Config, DayTimetable, ParsedAbsence, ParsedLesson, TimeUnit, TimetableTarget, WeekTimetable,
    add_days, today_local,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn sample_config() -> Config {
    Config {
        school: "school".into(),
        username: "user".into(),
        password: "secret".into(),
        server: "mese.webuntis.com".into(),
    }
}

fn sample_absence(id: i64, start_date: chrono::NaiveDate) -> ParsedAbsence {
    ParsedAbsence {
        id,
        student_name: "user".into(),
        reason: "Reason".into(),
        text: String::new(),
        excuse_status: String::new(),
        is_excused: false,
        start_date,
        end_date: start_date,
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
        subject_long_name: subject.into(),
        lesson_text: String::new(),
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
        remarks: String::new(),
    }
}

fn sample_week_timetable(period_count: usize) -> WeekTimetable {
    let monday = today_local();
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
    let monday_lessons = timegrid
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

    WeekTimetable {
        days: vec![
            DayTimetable {
                date: monday,
                day_name: "Monday".into(),
                lessons: monday_lessons,
            },
            DayTimetable {
                date: add_days(monday, 1),
                day_name: "Tuesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: add_days(monday, 2),
                day_name: "Wednesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: add_days(monday, 3),
                day_name: "Thursday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: add_days(monday, 4),
                day_name: "Friday".into(),
                lessons: Vec::new(),
            },
        ],
        timegrid,
    }
}

fn overlap_week_timetable() -> WeekTimetable {
    WeekTimetable {
        days: vec![
            DayTimetable {
                date: today_local(),
                day_name: "Monday".into(),
                lessons: vec![
                    sample_lesson("long", "M", "08:00", "09:40"),
                    sample_lesson("short", "E", "08:00", "08:50"),
                    sample_lesson("next", "B", "08:50", "09:40"),
                ],
            },
            DayTimetable {
                date: add_days(today_local(), 1),
                day_name: "Tuesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: add_days(today_local(), 2),
                day_name: "Wednesday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: add_days(today_local(), 3),
                day_name: "Thursday".into(),
                lessons: Vec::new(),
            },
            DayTimetable {
                date: add_days(today_local(), 4),
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
        ],
    }
}

#[test]
fn bootstrap_with_saved_password_enters_main_shell() {
    let mut state = AppState::new();
    let commands = state.handle_worker_event(WorkerEvent::BootstrapLoaded(BootstrapPayload {
        saved_config: Some(sample_config().saved()),
        saved_password: Some("secret".into()),
        secure_storage_notice: String::new(),
    }));

    assert_eq!(state.screen, Screen::MainShell);
    assert!(state.config.is_some());
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, AppCommand::LoadTimetableNetwork { .. }))
    );
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, AppCommand::LoadAbsenceChunk { .. }))
    );
}

#[test]
fn settings_modal_blocks_navigation_shortcuts() {
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.config = Some(sample_config());
    state.main.settings_open = true;
    let commands = state.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
    assert!(commands.is_empty());
    assert_eq!(state.main.timetable.week_offset, 0);
}

#[test]
fn absence_chunk_progress_stops_after_empty_streak() {
    let mut chunk_index = 0;
    let mut empty_chunk_streak = 0;
    let mut has_more = true;
    for _ in 0..4 {
        let next = update_absence_chunk_progress(chunk_index, empty_chunk_streak, 0);
        chunk_index = next.0;
        empty_chunk_streak = next.1;
        has_more = next.2;
    }
    assert!(!has_more);
}

#[test]
fn initial_empty_absence_chunk_triggers_background_prefetch() {
    let mut state = AppState::new();
    state.config = Some(sample_config());
    state.main.absences.generation = 1;
    state.main.absences.loading_initial = true;
    state.main.absences.has_more = true;

    let commands = state.handle_worker_event(WorkerEvent::AbsencesLoaded {
        generation: 1,
        is_initial: true,
        result: Ok(AbsenceChunkPayload {
            items: Vec::new(),
            next_chunk_index: 1,
            empty_chunk_streak: 1,
            has_more: true,
            days_loaded: 45,
        }),
    });

    assert!(commands.iter().any(|command| matches!(
        command,
        AppCommand::LoadAbsenceChunk {
            generation: 1,
            chunk_index: 1,
            is_initial: false,
            ..
        }
    )));
    assert!(state.main.absences.loading_more);
    assert!(!state.main.absences.loading_initial);
}

#[test]
fn loading_more_absences_keeps_initial_newest_chunk() {
    let mut state = AppState::new();
    state.config = Some(sample_config());
    state.main.absences.generation = 1;
    state.main.absences.loading_initial = true;
    state.main.absences.has_more = true;

    let newest_date = today_local();
    let next_date = add_days(newest_date, -1);
    let older_date = add_days(newest_date, -45);
    let oldest_date = add_days(newest_date, -46);

    let commands = state.handle_worker_event(WorkerEvent::AbsencesLoaded {
        generation: 1,
        is_initial: true,
        result: Ok(AbsenceChunkPayload {
            items: vec![
                sample_absence(100, newest_date),
                sample_absence(99, next_date),
            ],
            next_chunk_index: 1,
            empty_chunk_streak: 0,
            has_more: true,
            days_loaded: 45,
        }),
    });

    assert!(commands.iter().any(|command| matches!(
        command,
        AppCommand::LoadAbsenceChunk {
            generation: 1,
            chunk_index: 1,
            is_initial: false,
            ..
        }
    )));
    assert_eq!(state.main.absences.absences.len(), 2);
    assert_eq!(state.main.absences.absences[0].id, 100);
    assert_eq!(state.main.absences.absences[1].id, 99);

    let follow_up = state.handle_worker_event(WorkerEvent::AbsencesLoaded {
        generation: 1,
        is_initial: false,
        result: Ok(AbsenceChunkPayload {
            items: vec![
                sample_absence(80, older_date),
                sample_absence(79, oldest_date),
            ],
            next_chunk_index: 4,
            empty_chunk_streak: 0,
            has_more: false,
            days_loaded: 180,
        }),
    });

    assert!(follow_up.is_empty());
    assert_eq!(state.main.absences.absences.len(), 4);
    assert_eq!(state.main.absences.absences[0].id, 100);
    assert_eq!(state.main.absences.absences[1].id, 99);
    assert_eq!(state.main.absences.absences[2].id, 80);
    assert_eq!(state.main.absences.absences[3].id, 79);
}

#[test]
fn timetable_period_index_repeats_multi_period_lessons() {
    let mut state = AppState::new();
    state.main.timetable.data = Some(crate::models::WeekTimetable {
        days: vec![
            crate::models::DayTimetable {
                date: today_local(),
                day_name: "Monday".into(),
                lessons: vec![crate::models::ParsedLesson {
                    instance_id: "x".into(),
                    subject: "Math".into(),
                    subject_long_name: "Mathematics".into(),
                    lesson_text: String::new(),
                    cell_state: String::new(),
                    teacher: "M".into(),
                    teacher_long_name: "Mr M".into(),
                    all_teachers: vec!["M".into()],
                    all_teacher_long_names: vec!["Mr M".into()],
                    room: "A1".into(),
                    room_long_name: "Room A1".into(),
                    all_classes: vec!["1A".into()],
                    start_time: "08:00".into(),
                    end_time: "09:40".into(),
                    cancelled: false,
                    substitution: false,
                    remarks: String::new(),
                }],
            },
            crate::models::DayTimetable {
                date: today_local(),
                day_name: "Tuesday".into(),
                lessons: Vec::new(),
            },
            crate::models::DayTimetable {
                date: today_local(),
                day_name: "Wednesday".into(),
                lessons: Vec::new(),
            },
            crate::models::DayTimetable {
                date: today_local(),
                day_name: "Thursday".into(),
                lessons: Vec::new(),
            },
            crate::models::DayTimetable {
                date: today_local(),
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
        ],
    });
    assert_eq!(state.timetable_lessons_for(0, 0).len(), 1);
    assert_eq!(state.timetable_lessons_for(0, 1).len(), 1);
}

#[test]
fn absence_filter_helper_tracks_non_default_filters() {
    let mut state = AppState::new();
    assert!(!state.has_active_absence_filters());

    state.main.absences.search_query = "math".into();
    assert!(state.has_active_absence_filters());

    state.main.absences.search_query.clear();
    state.main.absences.window_filter = WindowFilter::D30;
    assert!(state.has_active_absence_filters());
}

#[test]
fn timetable_page_down_uses_three_line_grid_budget() {
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.terminal_height = 24;
    state.main.timetable.data = Some(sample_week_timetable(8));

    let commands = state.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));

    assert!(commands.is_empty());
    assert_eq!(state.timetable_rows_per_page(), 3);
    assert_eq!(state.main.timetable.selected_period_idx, 2);
    assert_eq!(state.main.timetable.scroll_offset, 0);
}

#[test]
fn timetable_scroll_tracks_selected_period_after_vertical_navigation() {
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.terminal_height = 24;
    state.main.timetable.data = Some(sample_week_timetable(8));

    for _ in 0..4 {
        let _ = state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
    }

    assert_eq!(state.main.timetable.selected_period_idx, 4);
    assert_eq!(state.main.timetable.scroll_offset, 2);
}

#[test]
fn timetable_loaded_clamps_selection_and_scroll_to_new_dataset() {
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.config = Some(sample_config());
    state.terminal_height = 24;
    state.main.timetable.data = Some(sample_week_timetable(8));
    state.main.timetable.selected_period_idx = 6;
    state.main.timetable.scroll_offset = 5;
    state.main.timetable.request_id = 42;

    let commands = state.handle_worker_event(WorkerEvent::TimetableLoaded {
        request_id: 42,
        week_date: today_local(),
        target: TimetableTarget::Own,
        result: Ok(sample_week_timetable(2)),
    });

    assert!(commands.is_empty());
    assert_eq!(state.main.timetable.selected_period_idx, 1);
    assert_eq!(state.main.timetable.scroll_offset, 0);
}

#[test]
fn timetable_overlap_cycle_uses_visible_lane_order() {
    let mut state = AppState::new();
    state.screen = Screen::MainShell;
    state.main.timetable.data = Some(overlap_week_timetable());
    state.main.timetable.selected_day_idx = 0;

    let _ = state.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert_eq!(state.main.timetable.selected_lesson_idx, 1);

    let _ = state.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
    assert_eq!(state.main.timetable.selected_period_idx, 1);
    assert_eq!(state.selected_timetable_lesson().unwrap().subject, "M");
}

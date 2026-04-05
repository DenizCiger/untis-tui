use super::*;
use crate::models::{Config, TimeUnit, today_local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn sample_config() -> Config {
    Config {
        school: "school".into(),
        username: "user".into(),
        password: "secret".into(),
        server: "mese.webuntis.com".into(),
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

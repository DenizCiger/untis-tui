use super::absences::{AbsencesPayload, RawAbsence, map_absence_payload};
use super::search::search_timetable_targets;
use crate::models::{
    Config, ParsedLesson, TimeUnit, TimetableSearchItem, TimetableSearchTargetType,
    parse_time_to_minutes,
};

fn item(
    id: i64,
    target_type: TimetableSearchTargetType,
    name: &str,
    long_name: &str,
    search_text: Option<&str>,
) -> TimetableSearchItem {
    TimetableSearchItem {
        r#type: target_type,
        id,
        name: name.to_owned(),
        long_name: long_name.to_owned(),
        search_text: search_text
            .unwrap_or(&format!("{name} {long_name}"))
            .to_lowercase(),
    }
}

#[test]
fn timetable_search_ranking_matches_contains_case_insensitively() {
    let results = search_timetable_targets(
        &[
            item(1, TimetableSearchTargetType::Teacher, "MrMiller", "Miller", None),
            item(
                2,
                TimetableSearchTargetType::Room,
                "Room A12",
                "Science Room",
                None,
            ),
        ],
        "MILL",
        Some(10),
    );
    assert_eq!(results.iter().map(|entry| entry.id).collect::<Vec<_>>(), vec![1]);
}

#[test]
fn timetable_search_ranking_prioritizes_starts_with_over_contains_matches() {
    let results = search_timetable_targets(
        &[
            item(1, TimetableSearchTargetType::Teacher, "Tina", "Teacher Tina", None),
            item(
                2,
                TimetableSearchTargetType::Teacher,
                "Math",
                "Advanced Tina Group",
                None,
            ),
            item(3, TimetableSearchTargetType::Teacher, "Bio", "Tina Biology", None),
        ],
        "ti",
        Some(10),
    );
    assert_eq!(results.iter().map(|entry| entry.id).collect::<Vec<_>>(), vec![1, 3, 2]);
}

#[test]
fn timetable_search_ranking_keeps_mixed_type_ordering_stable_for_equal_rank() {
    let results = search_timetable_targets(
        &[
            item(2, TimetableSearchTargetType::Teacher, "A-Name", "A-Name", None),
            item(1, TimetableSearchTargetType::Class, "A-Name", "A-Name", None),
            item(3, TimetableSearchTargetType::Room, "A-Name", "A-Name", None),
        ],
        "a-",
        Some(10),
    );
    assert_eq!(
        results
            .iter()
            .map(|entry| format!("{:?}:{}", entry.r#type, entry.id).to_lowercase())
            .collect::<Vec<_>>(),
        vec!["class:1", "room:3", "teacher:2"]
    );
}

#[test]
fn timetable_search_ranking_matches_multi_token_queries_across_name_fields() {
    let results = search_timetable_targets(
        &[
            item(
                1,
                TimetableSearchTargetType::Teacher,
                "Max Mustermann",
                "MMAX",
                Some("max mustermann mmax"),
            ),
            item(
                2,
                TimetableSearchTargetType::Teacher,
                "Max Muster",
                "MMUS",
                Some("max muster mmus"),
            ),
        ],
        "max mmax",
        None,
    );
    assert_eq!(results.iter().map(|entry| entry.id).collect::<Vec<_>>(), vec![1]);
}

#[test]
fn timetable_search_ranking_returns_all_matches_when_no_limit_is_provided() {
    let results = search_timetable_targets(
        &[
            item(1, TimetableSearchTargetType::Teacher, "AA", "AA", None),
            item(2, TimetableSearchTargetType::Teacher, "AB", "AB", None),
            item(3, TimetableSearchTargetType::Teacher, "AC", "AC", None),
        ],
        "a",
        None,
    );
    assert_eq!(results.len(), 3);
}

#[test]
fn repeated_rows_logic_repeats_multi_period_lessons() {
    let lesson = ParsedLesson {
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
    };
    let periods = vec![
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
    ];
    let hits = periods
        .iter()
        .filter(|period| {
            let lesson_start = parse_time_to_minutes(&lesson.start_time);
            let lesson_end = parse_time_to_minutes(&lesson.end_time);
            let period_start = parse_time_to_minutes(&period.start_time);
            let period_end = parse_time_to_minutes(&period.end_time);
            lesson_start < period_end && lesson_end > period_start
        })
        .count();
    assert_eq!(hits, 2);
}

#[test]
fn absence_mapping_uses_bun_compatible_fields_and_sorting() {
    let config = Config {
        school: "school".into(),
        username: "user".into(),
        password: "secret".into(),
        server: "mese.webuntis.com".into(),
    };
    let payload = AbsencesPayload {
        absences: vec![
            RawAbsence {
                id: 1,
                start_date: 20260115,
                end_date: 20260115,
                start_time: 815,
                end_time: 900,
                student_name: String::new(),
                reason: "Ill".into(),
                text: String::new(),
                excuse_status: "Open".into(),
                is_excused: false,
            },
            RawAbsence {
                id: 2,
                start_date: 20260120,
                end_date: 20260120,
                start_time: 700,
                end_time: 745,
                student_name: "Student".into(),
                reason: String::new(),
                text: "Doctor".into(),
                excuse_status: "Excused".into(),
                is_excused: true,
            },
        ],
    };

    let mapped = map_absence_payload(&config, payload);

    assert_eq!(mapped.len(), 2);
    assert_eq!(mapped[0].id, 2);
    assert_eq!(mapped[0].student_name, "Student");
    assert_eq!(mapped[1].id, 1);
    assert_eq!(mapped[1].student_name, "user");
    assert_eq!(mapped[1].start_time, "08:15");
}

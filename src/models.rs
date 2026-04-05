use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub school: String,
    pub username: String,
    pub password: String,
    pub server: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedConfig {
    pub school: String,
    pub username: String,
    pub server: String,
}

impl Config {
    pub fn saved(&self) -> SavedConfig {
        SavedConfig {
            school: self.school.clone(),
            username: self.username.clone(),
            server: self.server.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLesson {
    #[serde(default)]
    pub instance_id: String,
    pub subject: String,
    pub subject_long_name: String,
    #[serde(default)]
    pub lesson_text: String,
    #[serde(default)]
    pub cell_state: String,
    #[serde(default)]
    pub teacher: String,
    #[serde(default)]
    pub teacher_long_name: String,
    #[serde(default)]
    pub all_teachers: Vec<String>,
    #[serde(default)]
    pub all_teacher_long_names: Vec<String>,
    #[serde(default)]
    pub room: String,
    #[serde(default)]
    pub room_long_name: String,
    #[serde(default)]
    pub all_classes: Vec<String>,
    pub start_time: String,
    pub end_time: String,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(default)]
    pub substitution: bool,
    #[serde(default)]
    pub remarks: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAbsence {
    pub id: i64,
    pub student_name: String,
    pub reason: String,
    pub text: String,
    pub excuse_status: String,
    pub is_excused: bool,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeUnit {
    pub name: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayTimetable {
    #[serde(with = "date_serde")]
    pub date: NaiveDate,
    pub day_name: String,
    pub lessons: Vec<ParsedLesson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekTimetable {
    pub days: Vec<DayTimetable>,
    pub timegrid: Vec<TimeUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimetableSearchTargetType {
    Class,
    Room,
    Teacher,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimetableElementType {
    Class = 1,
    Teacher = 2,
    Subject = 3,
    Room = 4,
    Student = 5,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimetableTarget {
    Own,
    Class {
        id: i64,
        name: String,
        long_name: String,
    },
    Room {
        id: i64,
        name: String,
        long_name: String,
    },
    Teacher {
        id: i64,
        name: String,
        long_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimetableSearchItem {
    pub r#type: TimetableSearchTargetType,
    pub id: i64,
    pub name: String,
    pub long_name: String,
    pub search_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimetableRequestTarget {
    Own,
    Target {
        id: i64,
        element_type: TimetableElementType,
    },
}

pub fn get_default_timetable_target() -> TimetableTarget {
    TimetableTarget::Own
}

pub fn target_to_cache_key(target: Option<&TimetableTarget>) -> String {
    match target.unwrap_or(&TimetableTarget::Own) {
        TimetableTarget::Own => "own".to_owned(),
        TimetableTarget::Class { id, .. } => format!("class:{id}"),
        TimetableTarget::Room { id, .. } => format!("room:{id}"),
        TimetableTarget::Teacher { id, .. } => format!("teacher:{id}"),
    }
}

pub fn format_timetable_target_label(target: Option<&TimetableTarget>) -> String {
    match target.unwrap_or(&TimetableTarget::Own) {
        TimetableTarget::Own => "My timetable".to_owned(),
        TimetableTarget::Class { name, .. } => format!("Class: {name}"),
        TimetableTarget::Room { name, .. } => format!("Room: {name}"),
        TimetableTarget::Teacher { name, .. } => format!("Teacher: {name}"),
    }
}

pub fn resolve_timetable_request(target: &TimetableTarget) -> TimetableRequestTarget {
    match target {
        TimetableTarget::Own => TimetableRequestTarget::Own,
        TimetableTarget::Class { id, .. } => TimetableRequestTarget::Target {
            id: *id,
            element_type: TimetableElementType::Class,
        },
        TimetableTarget::Room { id, .. } => TimetableRequestTarget::Target {
            id: *id,
            element_type: TimetableElementType::Room,
        },
        TimetableTarget::Teacher { id, .. } => TimetableRequestTarget::Target {
            id: *id,
            element_type: TimetableElementType::Teacher,
        },
    }
}

pub fn build_profile_key(saved: &SavedConfig) -> String {
    format!("{}|{}|{}", saved.server, saved.school, saved.username)
}

pub fn today_local() -> NaiveDate {
    Local::now().date_naive()
}

pub fn add_days(date: NaiveDate, days: i64) -> NaiveDate {
    date + Duration::days(days)
}

pub fn get_monday(date: NaiveDate) -> NaiveDate {
    let weekday = i64::from(date.weekday().num_days_from_monday());
    date - Duration::days(weekday)
}

pub fn current_week_range(week_offset: i32) -> (NaiveDate, NaiveDate) {
    let monday = get_monday(add_days(today_local(), i64::from(week_offset) * 7));
    (monday, add_days(monday, 4))
}

pub fn parse_untis_date(value: i32) -> Option<NaiveDate> {
    let year = value / 10_000;
    let month = ((value / 100) % 100) as u32;
    let day = (value % 100) as u32;
    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn format_untis_date(date: NaiveDate) -> String {
    format!("{:04}{:02}{:02}", date.year(), date.month(), date.day())
}

pub fn format_web_date(date: NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

pub fn format_untis_time(value: i32) -> String {
    let hours = value.div_euclid(100);
    let minutes = value.rem_euclid(100);
    format!("{hours:02}:{minutes:02}")
}

pub fn parse_time_to_minutes(value: &str) -> i32 {
    let mut parts = value.split(':');
    let hours = parts
        .next()
        .and_then(|part| part.parse::<i32>().ok())
        .unwrap_or(0);
    let minutes = parts
        .next()
        .and_then(|part| part.parse::<i32>().ok())
        .unwrap_or(0);
    hours * 60 + minutes
}

pub fn format_date(date: NaiveDate) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month = MONTHS.get(date.month0() as usize).copied().unwrap_or("???");
    format!("{month} {}, {}", date.day(), date.year())
}

pub fn get_weekday_name(date: NaiveDate) -> String {
    match date.weekday().number_from_monday() {
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Sunday",
    }
    .to_owned()
}

pub fn compare_absence_newest_first(
    left: &ParsedAbsence,
    right: &ParsedAbsence,
) -> std::cmp::Ordering {
    right
        .start_date
        .cmp(&left.start_date)
        .then_with(|| right.start_time.cmp(&left.start_time))
        .then_with(|| right.id.cmp(&left.id))
}

pub fn merge_absences(
    previous: &[ParsedAbsence],
    incoming: &[ParsedAbsence],
) -> Vec<ParsedAbsence> {
    let mut by_id = HashMap::<i64, ParsedAbsence>::new();
    for absence in previous.iter().chain(incoming.iter()) {
        by_id.insert(absence.id, absence.clone());
    }

    let mut merged: Vec<_> = by_id.into_values().collect();
    merged.sort_by(compare_absence_newest_first);
    merged
}

pub mod date_serde {
    use super::*;

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format_web_date(*date))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            return Ok(date);
        }
        if let Ok(date_time) = chrono::DateTime::parse_from_rfc3339(&value) {
            return Ok(date_time.date_naive());
        }
        Err(serde::de::Error::custom("invalid date string"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timetable_target_helpers_resolve_own_request() {
        assert_eq!(
            resolve_timetable_request(&TimetableTarget::Own),
            TimetableRequestTarget::Own
        );
    }

    #[test]
    fn timetable_target_helpers_map_targets_to_element_types() {
        assert_eq!(
            resolve_timetable_request(&TimetableTarget::Class {
                id: 12,
                name: "1A".into(),
                long_name: "1A Class".into(),
            }),
            TimetableRequestTarget::Target {
                id: 12,
                element_type: TimetableElementType::Class,
            }
        );
        assert_eq!(
            resolve_timetable_request(&TimetableTarget::Room {
                id: 13,
                name: "A12".into(),
                long_name: "Room A12".into(),
            }),
            TimetableRequestTarget::Target {
                id: 13,
                element_type: TimetableElementType::Room,
            }
        );
        assert_eq!(
            resolve_timetable_request(&TimetableTarget::Teacher {
                id: 14,
                name: "MILL".into(),
                long_name: "Miller".into(),
            }),
            TimetableRequestTarget::Target {
                id: 14,
                element_type: TimetableElementType::Teacher,
            }
        );
    }

    #[test]
    fn timetable_target_helpers_build_cache_keys() {
        assert_eq!(target_to_cache_key(Some(&TimetableTarget::Own)), "own");
        assert_eq!(
            target_to_cache_key(Some(&TimetableTarget::Class {
                id: 42,
                name: "4AHIF".into(),
                long_name: "4AHIF Class".into(),
            })),
            "class:42"
        );
    }

    #[test]
    fn timetable_target_helpers_format_labels() {
        assert_eq!(
            format_timetable_target_label(Some(&TimetableTarget::Own)),
            "My timetable"
        );
        assert_eq!(
            format_timetable_target_label(Some(&TimetableTarget::Teacher {
                id: 99,
                name: "DELL".into(),
                long_name: "Mr Dell".into(),
            })),
            "Teacher: DELL"
        );
    }
}

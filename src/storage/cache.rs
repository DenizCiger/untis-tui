use crate::models::{DayTimetable, ParsedLesson, WeekTimetable};
use crate::storage::{StorageError, config_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_CACHED_WEEKS: usize = 12;
const CACHE_TTL_MS: u64 = 1000 * 60 * 60 * 24 * 21;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CacheData {
    #[serde(default)]
    weeks: HashMap<String, CachedWeekEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedWeekEntry {
    data: WeekTimetable,
    timestamp: u64,
}

pub fn cache_file() -> Result<PathBuf, StorageError> {
    Ok(config_dir()?.join("cache.json"))
}

pub fn build_week_cache_key(monday: &str, target_key: &str) -> String {
    let normalized_target_key = if target_key.trim().is_empty() {
        "own"
    } else {
        target_key.trim()
    };
    format!("{normalized_target_key}:{monday}")
}

pub fn get_week_lookup_keys(monday: &str, target_key: &str) -> Vec<String> {
    let normalized_target_key = if target_key.trim().is_empty() {
        "own"
    } else {
        target_key.trim()
    };

    if normalized_target_key == "own" {
        vec![
            build_week_cache_key(monday, normalized_target_key),
            monday.to_owned(),
        ]
    } else {
        vec![build_week_cache_key(monday, normalized_target_key)]
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn load_cache() -> CacheData {
    let path = match cache_file() {
        Ok(path) => path,
        Err(_) => return CacheData::default(),
    };
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return CacheData::default(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_cache(cache: &CacheData) -> Result<(), StorageError> {
    fs::create_dir_all(config_dir()?)?;
    fs::write(cache_file()?, serde_json::to_vec_pretty(cache)?)?;
    Ok(())
}

fn ensure_lesson_instance_id(
    lesson: &ParsedLesson,
    day_date: chrono::NaiveDate,
    index_in_day: usize,
) -> ParsedLesson {
    let date_part = day_date.format("%Y-%m-%d").to_string();
    ParsedLesson {
        instance_id: if lesson.instance_id.is_empty() {
            format!(
                "{date_part}-{}-{}-{}-{}-{}-{index_in_day}",
                lesson.start_time, lesson.end_time, lesson.subject, lesson.teacher, lesson.room
            )
        } else {
            lesson.instance_id.clone()
        },
        lesson_text: lesson.lesson_text.clone(),
        cell_state: lesson.cell_state.clone(),
        all_teachers: if lesson.all_teachers.is_empty() && !lesson.teacher.is_empty() {
            vec![lesson.teacher.clone()]
        } else {
            lesson.all_teachers.clone()
        },
        all_teacher_long_names: if lesson.all_teacher_long_names.is_empty()
            && !lesson.teacher_long_name.is_empty()
        {
            vec![lesson.teacher_long_name.clone()]
        } else {
            lesson.all_teacher_long_names.clone()
        },
        all_classes: lesson.all_classes.clone(),
        ..lesson.clone()
    }
}

pub fn get_cached_week(monday: &str, target_key: &str) -> Option<WeekTimetable> {
    let mut cache = load_cache();
    let lookup_keys = get_week_lookup_keys(monday, target_key);

    let mut storage_key = None;
    let mut found = None;
    for lookup_key in lookup_keys {
        if let Some(entry) = cache.weeks.get(&lookup_key) {
            storage_key = Some(lookup_key);
            found = Some(entry.clone());
            break;
        }
    }

    let week = found?;
    if now_ms().saturating_sub(week.timestamp) > CACHE_TTL_MS {
        if let Some(storage_key) = storage_key {
            cache.weeks.remove(&storage_key);
            let _ = save_cache(&cache);
        }
        return None;
    }

    Some(WeekTimetable {
        days: week
            .data
            .days
            .into_iter()
            .map(|day| {
                let date = day.date;
                DayTimetable {
                    lessons: day
                        .lessons
                        .iter()
                        .enumerate()
                        .map(|(index, lesson)| ensure_lesson_instance_id(lesson, date, index))
                        .collect(),
                    ..day
                }
            })
            .collect(),
        timegrid: week.data.timegrid,
    })
}

pub fn save_week_to_cache(
    monday: &str,
    data: &WeekTimetable,
    target_key: &str,
) -> Result<(), StorageError> {
    let mut cache = load_cache();
    cache.weeks.insert(
        build_week_cache_key(monday, target_key),
        CachedWeekEntry {
            data: data.clone(),
            timestamp: now_ms(),
        },
    );

    let mut entries: Vec<_> = cache.weeks.into_iter().collect();
    entries.sort_by(|left, right| right.1.timestamp.cmp(&left.1.timestamp));
    entries.truncate(MAX_CACHED_WEEKS);
    cache.weeks = entries.into_iter().collect();

    save_cache(&cache)
}

pub fn clear_cache() -> Result<(), StorageError> {
    if let Ok(path) = cache_file() {
        fs::write(path, serde_json::to_vec_pretty(&CacheData::default())?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_cache_keys_create_target_scoped_keys() {
        assert_eq!(
            build_week_cache_key("2026-01-05", "class:42"),
            "class:42:2026-01-05"
        );
    }

    #[test]
    fn week_cache_keys_include_legacy_own_key_fallback() {
        assert_eq!(
            get_week_lookup_keys("2026-01-05", "own"),
            vec!["own:2026-01-05".to_owned(), "2026-01-05".to_owned()]
        );
    }

    #[test]
    fn week_cache_keys_do_not_include_legacy_fallback_for_non_own_keys() {
        assert_eq!(
            get_week_lookup_keys("2026-01-05", "room:12"),
            vec!["room:12:2026-01-05".to_owned()]
        );
    }
}

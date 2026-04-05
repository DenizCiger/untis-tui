mod absences;
mod input;
mod requests;
mod selection;
#[cfg(test)]
mod tests;
mod text_input;
mod types;
mod worker;

use crate::models::{Config, WeekTimetable, add_days, format_timetable_target_label};
use crate::storage::config::load_config;
use crate::storage::secret::{get_secure_storage_diagnostic, load_password};
use chrono::NaiveDate;

pub use types::{
    AbsenceChunkPayload, AppCommand, AppState, BootstrapPayload, LoginField, LoginState,
    MainState, Screen, StatusFilter, TextInputState, TimetableState, WindowFilter, WorkerEvent,
};

const CHUNK_DAYS: usize = 45;
const MAX_HISTORY_DAYS: usize = 365 * 5;
const MAX_EMPTY_CHUNK_STREAK: usize = 4;
const LOAD_MORE_BURST_CHUNKS: usize = 3;

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initial_commands(&self) -> Vec<AppCommand> {
        vec![AppCommand::Bootstrap]
    }

    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    pub fn saved_login_config(&self) -> Option<Config> {
        Some(Config {
            school: self.saved_config.as_ref()?.school.clone(),
            username: self.saved_config.as_ref()?.username.clone(),
            password: self.saved_password.clone()?,
            server: self.saved_config.as_ref()?.server.clone(),
        })
    }

    pub fn timetable_target_label(&self) -> String {
        format_timetable_target_label(Some(&self.main.timetable.active_target))
    }

    pub(super) fn hydrate_login_form(&mut self) {
        self.login.server.value = self
            .saved_config
            .as_ref()
            .map(|config| config.server.clone())
            .unwrap_or_default();
        self.login.school.value = self
            .saved_config
            .as_ref()
            .map(|config| config.school.clone())
            .unwrap_or_default();
        self.login.username.value = self
            .saved_config
            .as_ref()
            .map(|config| config.username.clone())
            .unwrap_or_default();
        self.login.password.value.clear();
        self.login.server.cursor = self.login.server.value.len();
        self.login.school.cursor = self.login.school.value.len();
        self.login.username.cursor = self.login.username.value.len();
        self.login.password.cursor = 0;
    }
}

pub fn build_bootstrap_payload() -> BootstrapPayload {
    let diagnostic = get_secure_storage_diagnostic();
    let saved_config = load_config();
    let saved_password = saved_config
        .as_ref()
        .and_then(|config| load_password(config).ok().flatten());
    BootstrapPayload {
        saved_config,
        saved_password,
        secure_storage_notice: if diagnostic.available {
            String::new()
        } else {
            diagnostic.message
        },
    }
}

pub fn build_absence_chunk_request(
    base_date: NaiveDate,
    chunk_index: usize,
    is_initial: bool,
) -> Vec<(NaiveDate, NaiveDate)> {
    let chunks = if is_initial {
        1
    } else {
        LOAD_MORE_BURST_CHUNKS
    };
    (0..chunks)
        .map(|offset| chunk_range(base_date, chunk_index + offset))
        .collect()
}

pub fn chunk_range(base_date: NaiveDate, chunk_index: usize) -> (NaiveDate, NaiveDate) {
    let range_end = add_days(base_date, -((chunk_index * CHUNK_DAYS) as i64));
    let range_start = add_days(range_end, -((CHUNK_DAYS - 1) as i64));
    (range_start, range_end)
}

pub fn update_absence_chunk_progress(
    chunk_index: usize,
    empty_chunk_streak: usize,
    records_loaded: usize,
) -> (usize, usize, bool, usize) {
    let next_chunk_index = chunk_index + 1;
    let next_empty_chunk_streak = if records_loaded == 0 {
        empty_chunk_streak + 1
    } else {
        0
    };
    let reached_max_history = next_chunk_index * CHUNK_DAYS >= MAX_HISTORY_DAYS;
    let reached_empty_streak = next_empty_chunk_streak >= MAX_EMPTY_CHUNK_STREAK;
    let has_more = !reached_max_history && !reached_empty_streak;
    let days_loaded = next_chunk_index * CHUNK_DAYS;
    (
        next_chunk_index,
        next_empty_chunk_streak,
        has_more,
        days_loaded,
    )
}

fn lessons_for_period(
    data: Option<&WeekTimetable>,
    day_idx: usize,
    period_idx: usize,
) -> Vec<crate::models::ParsedLesson> {
    let Some(data) = data else {
        return Vec::new();
    };
    let Some(day) = data.days.get(day_idx) else {
        return Vec::new();
    };
    let Some(period) = data.timegrid.get(period_idx) else {
        return Vec::new();
    };

    let period_start = crate::models::parse_time_to_minutes(&period.start_time);
    let period_end = crate::models::parse_time_to_minutes(&period.end_time);
    let mut lessons = day
        .lessons
        .iter()
        .filter(|lesson| {
            let lesson_start = crate::models::parse_time_to_minutes(&lesson.start_time);
            let lesson_end = crate::models::parse_time_to_minutes(&lesson.end_time);
            lesson_start < period_end && lesson_end > period_start
        })
        .cloned()
        .collect::<Vec<_>>();
    lessons.sort_by(|left, right| {
        left.start_time
            .cmp(&right.start_time)
            .then_with(|| left.end_time.cmp(&right.end_time))
            .then_with(|| left.subject.cmp(&right.subject))
            .then_with(|| left.teacher.cmp(&right.teacher))
            .then_with(|| left.room.cmp(&right.room))
            .then_with(|| left.instance_id.cmp(&right.instance_id))
    });
    lessons
}

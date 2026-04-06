use super::{AppCommand, AppState};
use crate::models::{
    Config, add_days, build_profile_key, get_default_timetable_target, target_to_cache_key,
    today_local,
};
use crate::storage::cache::{clear_cache, get_cached_week};
use crate::timetable_model::{timetable_body_height_from_terminal, timetable_rows_per_page};
use std::collections::HashSet;

impl AppState {
    pub(super) fn submit_login(&mut self) -> Vec<AppCommand> {
        let config = Config {
            school: self.login.school.value.trim().to_owned(),
            username: self.login.username.value.trim().to_owned(),
            password: self.login.password.value.clone(),
            server: self.login.server.value.trim().to_owned(),
        };

        if config.server.is_empty()
            || config.school.is_empty()
            || config.username.is_empty()
            || config.password.is_empty()
        {
            self.login.error = "All fields are required".to_owned();
            return Vec::new();
        }

        self.login.loading = true;
        self.login.error.clear();
        vec![AppCommand::ValidateLogin(config)]
    }

    pub(super) fn enter_main_shell(&mut self) -> Vec<AppCommand> {
        let saved = match self.config.as_ref() {
            Some(config) => config.saved(),
            None => return Vec::new(),
        };
        let profile_key = build_profile_key(&saved);
        let session = self.profile_sessions.get(&profile_key).cloned().unwrap_or(
            super::types::ProfileSessionState {
                active_target: get_default_timetable_target(),
                search_index: Vec::new(),
            },
        );

        self.main = super::MainState::default();
        self.main.timetable.active_target = session.active_target;
        self.main.timetable.search_index = session.search_index;
        if self.is_demo_mode() {
            self.main.timetable.selected_day_idx = 0;
            self.main.timetable.selected_period_idx = 0;
            self.main.timetable.selected_lesson_idx = 1;
            if self.main.timetable.search_index.is_empty() {
                self.main.timetable.search_index = crate::demo::demo_search_index();
            }
        }
        self.main.absences.base_date = today_local();
        self.sync_timetable_scroll();
        let mut commands = self.request_timetable(false);
        commands.extend(self.request_absences_refresh());
        commands
    }

    pub(super) fn request_search_index(&mut self) -> Vec<AppCommand> {
        if self.is_demo_mode() {
            let profile_key = self
                .saved_config
                .as_ref()
                .map(build_profile_key)
                .unwrap_or_default();
            self.main.timetable.search_index_loading = true;
            self.main.timetable.search_index_error.clear();
            return self.handle_worker_event(super::WorkerEvent::SearchIndexLoaded {
                profile_key,
                result: Ok(crate::demo::demo_search_index()),
            });
        }

        let config = match self.config.clone() {
            Some(config) => config,
            None => return Vec::new(),
        };
        let profile_key = build_profile_key(&config.saved());
        self.main.timetable.search_index_loading = true;
        self.main.timetable.search_index_error.clear();
        vec![AppCommand::LoadSearchIndex {
            profile_key,
            config,
        }]
    }

    pub(super) fn request_timetable(&mut self, force_refresh: bool) -> Vec<AppCommand> {
        if self.is_demo_mode() {
            let week_date = add_days(
                today_local(),
                i64::from(self.main.timetable.week_offset) * 7,
            );
            let target = self.main.timetable.active_target.clone();
            let request_id = self.next_request_id;
            self.next_request_id += 1;
            self.main.timetable.request_id = request_id;
            self.main.timetable.loading = true;
            self.main.timetable.is_from_cache = false;
            self.main.timetable.error.clear();
            if force_refresh {
                self.main.timetable.data = None;
            }
            return self.handle_worker_event(super::WorkerEvent::TimetableLoaded {
                request_id,
                week_date,
                target: target.clone(),
                result: Ok(crate::demo::demo_week_timetable(week_date, &target)),
            });
        }

        let config = match self.config.clone() {
            Some(config) => config,
            None => return Vec::new(),
        };

        let week_date = add_days(
            today_local(),
            i64::from(self.main.timetable.week_offset) * 7,
        );
        let target = self.main.timetable.active_target.clone();
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.main.timetable.request_id = request_id;

        let monday = crate::models::get_monday(week_date);
        let monday_key = crate::models::format_web_date(monday);
        if !force_refresh {
            if let Some(cached) = get_cached_week(&monday_key, &target_to_cache_key(Some(&target)))
            {
                self.main.timetable.data = Some(cached);
                self.main.timetable.loading = false;
                self.main.timetable.is_from_cache = true;
                self.main.timetable.error.clear();
                self.sync_timetable_scroll();
                return vec![AppCommand::LoadTimetableNetwork {
                    request_id,
                    config,
                    week_date,
                    target,
                }];
            }
            self.main.timetable.data = None;
        }

        self.main.timetable.loading = true;
        self.main.timetable.is_from_cache = false;
        self.main.timetable.error.clear();
        vec![AppCommand::LoadTimetableNetwork {
            request_id,
            config,
            week_date,
            target,
        }]
    }

    pub(super) fn request_absences_refresh(&mut self) -> Vec<AppCommand> {
        if self.is_demo_mode() {
            self.main.absences.generation += 1;
            self.main.absences.chunk_index = 0;
            self.main.absences.empty_chunk_streak = 0;
            self.main.absences.absences.clear();
            self.main.absences.error.clear();
            self.main.absences.has_more = true;
            self.main.absences.days_loaded = 0;
            self.main.absences.loading_initial = true;
            self.main.absences.loading_more = false;
            self.main.absences.base_date = today_local();
            let generation = self.main.absences.generation;
            return self.handle_worker_event(super::WorkerEvent::AbsencesLoaded {
                generation,
                is_initial: true,
                result: Ok(self.build_demo_absence_payload(0, true)),
            });
        }

        let config = match self.config.clone() {
            Some(config) => config,
            None => return Vec::new(),
        };
        self.main.absences.generation += 1;
        self.main.absences.chunk_index = 0;
        self.main.absences.empty_chunk_streak = 0;
        self.main.absences.absences.clear();
        self.main.absences.error.clear();
        self.main.absences.has_more = true;
        self.main.absences.days_loaded = 0;
        self.main.absences.loading_initial = true;
        self.main.absences.loading_more = false;
        self.main.absences.base_date = today_local();
        vec![AppCommand::LoadAbsenceChunk {
            generation: self.main.absences.generation,
            config,
            base_date: self.main.absences.base_date,
            chunk_index: 0,
            is_initial: true,
        }]
    }

    pub(super) fn request_absences_more(&mut self) -> Vec<AppCommand> {
        if self.main.absences.loading_initial
            || self.main.absences.loading_more
            || !self.main.absences.has_more
        {
            return Vec::new();
        }

        if self.is_demo_mode() {
            self.main.absences.loading_more = true;
            let generation = self.main.absences.generation;
            return self.handle_worker_event(super::WorkerEvent::AbsencesLoaded {
                generation,
                is_initial: false,
                result: Ok(self.build_demo_absence_payload(self.main.absences.chunk_index, false)),
            });
        }

        let config = match self.config.clone() {
            Some(config) => config,
            None => return Vec::new(),
        };
        self.main.absences.loading_more = true;
        vec![AppCommand::LoadAbsenceChunk {
            generation: self.main.absences.generation,
            config,
            base_date: self.main.absences.base_date,
            chunk_index: self.main.absences.chunk_index,
            is_initial: false,
        }]
    }

    pub(super) fn maybe_request_more_absences(&mut self) -> Vec<AppCommand> {
        let filtered_len = self.filtered_absences().len();
        let prefetch_threshold = self.absences_page_jump().max(6);
        let maintain_prefetch = self.main.absences.status_filter == super::StatusFilter::All
            && self.main.absences.window_filter == super::WindowFilter::All
            && self.main.absences.search_query.trim().is_empty();

        if filtered_len == 0 {
            return self.request_absences_more();
        }

        if self.main.absences.selected_idx >= filtered_len.saturating_sub(prefetch_threshold) {
            return self.request_absences_more();
        }

        if maintain_prefetch
            && filtered_len
                <= self.main.absences.selected_idx + self.absences_page_jump() + prefetch_threshold
        {
            return self.request_absences_more();
        }

        Vec::new()
    }

    pub(super) fn persist_profile_session(&mut self) {
        let saved = self
            .config
            .as_ref()
            .map(Config::saved)
            .or_else(|| self.saved_config.clone());
        let Some(saved) = saved else {
            return;
        };
        self.profile_sessions.insert(
            build_profile_key(&saved),
            super::types::ProfileSessionState {
                active_target: self.main.timetable.active_target.clone(),
                search_index: self.main.timetable.search_index.clone(),
            },
        );
    }

    pub(super) fn perform_logout(&mut self) {
        if self.is_demo_mode() {
            let width = self.terminal_width;
            let height = self.terminal_height;
            *self = AppState::new_demo();
            self.update_terminal_size(width, height);
            let _ = self.enter_main_shell();
            return;
        }

        self.persist_profile_session();
        if let Some(config) = &self.config {
            self.saved_password = Some(config.password.clone());
        }
        let _ = clear_cache();
        self.config = None;
        self.main.settings_open = false;
        self.main.timetable.search_open = false;
        self.main.absences.search_open = false;
        self.login.loading = false;
        self.login.error.clear();
        self.screen = super::Screen::Login;
        self.hydrate_login_form();
    }

    pub(super) fn timetable_rows_per_page(&self) -> usize {
        timetable_rows_per_page(timetable_body_height_from_terminal(self.terminal_height))
    }

    pub(super) fn absences_page_jump(&self) -> usize {
        usize::from(self.terminal_height.saturating_sub(11)).max(4) / 2
    }

    fn build_demo_absence_payload(
        &self,
        chunk_index: usize,
        is_initial: bool,
    ) -> super::AbsenceChunkPayload {
        let demo_absences = crate::demo::demo_absences();
        let mut items = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut next_chunk_index = chunk_index;
        let mut empty_chunk_streak = 0;
        let mut has_more = true;
        let mut days_loaded = chunk_index * 45;

        for (index, (range_start, range_end)) in super::build_absence_chunk_request(
            self.main.absences.base_date,
            chunk_index,
            is_initial,
        )
        .into_iter()
        .enumerate()
        {
            let range_items = demo_absences
                .iter()
                .filter(|absence| {
                    absence.end_date >= range_start && absence.start_date <= range_end
                })
                .filter(|absence| seen_ids.insert(absence.id))
                .cloned()
                .collect::<Vec<_>>();

            let (updated_chunk_index, updated_empty_streak, updated_has_more, updated_days_loaded) =
                super::update_absence_chunk_progress(
                    chunk_index + index,
                    empty_chunk_streak,
                    range_items.len(),
                );

            next_chunk_index = updated_chunk_index;
            empty_chunk_streak = updated_empty_streak;
            has_more = updated_has_more;
            days_loaded = updated_days_loaded;
            items.extend(range_items);

            if is_initial || items.len() >= 12 || !has_more {
                break;
            }
        }

        super::AbsenceChunkPayload {
            items,
            next_chunk_index,
            empty_chunk_streak,
            has_more,
            days_loaded,
        }
    }
}

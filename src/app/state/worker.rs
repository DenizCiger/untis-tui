use super::{AbsenceChunkPayload, AppCommand, AppState, BootstrapPayload, WorkerEvent};
use crate::models::{
    Config, TimetableSearchItem, TimetableTarget, WeekTimetable, build_profile_key,
    target_to_cache_key,
};
use crate::storage::cache::save_week_to_cache;
use crate::storage::config::save_config;
use crate::storage::secret::save_password;
use chrono::NaiveDate;

impl AppState {
    pub fn handle_worker_event(&mut self, event: WorkerEvent) -> Vec<AppCommand> {
        match event {
            WorkerEvent::BootstrapLoaded(payload) => self.handle_bootstrap_loaded(payload),
            WorkerEvent::LoginValidated(result) => self.handle_login_validated(result),
            WorkerEvent::TimetableLoaded {
                request_id,
                week_date,
                target,
                result,
            } => self.handle_timetable_loaded(request_id, week_date, target, result),
            WorkerEvent::SearchIndexLoaded {
                profile_key,
                result,
            } => self.handle_search_index_loaded(&profile_key, result),
            WorkerEvent::AbsencesLoaded {
                generation,
                is_initial,
                result,
            } => self.handle_absences_loaded(generation, is_initial, result),
        }
    }

    fn handle_bootstrap_loaded(&mut self, payload: BootstrapPayload) -> Vec<AppCommand> {
        self.saved_config = payload.saved_config.clone();
        self.saved_password = payload.saved_password.clone();
        self.secure_storage_notice = payload.secure_storage_notice;
        self.hydrate_login_form();

        if let (Some(saved_config), Some(saved_password)) =
            (payload.saved_config, payload.saved_password)
        {
            self.config = Some(Config {
                school: saved_config.school.clone(),
                username: saved_config.username.clone(),
                password: saved_password,
                server: saved_config.server.clone(),
            });
            self.screen = super::Screen::MainShell;
            return self.enter_main_shell();
        }

        self.screen = super::Screen::Login;
        Vec::new()
    }

    fn handle_login_validated(&mut self, result: Result<Config, String>) -> Vec<AppCommand> {
        self.login.loading = false;
        match result {
            Ok(config) => {
                self.login.error.clear();
                self.saved_config = Some(config.saved());
                self.saved_password = Some(config.password.clone());
                self.config = Some(config.clone());
                self.screen = super::Screen::MainShell;

                if self.is_demo_mode() {
                    self.app_error.clear();
                } else {
                    if let Err(error) = save_config(&config) {
                        self.app_error = format!(
                            "Login succeeded, but profile settings could not be saved to disk: {error}"
                        );
                    } else {
                        self.app_error.clear();
                    }

                    if let Err(error) = save_password(&config.saved(), &config.password) {
                        self.app_error =
                            format!("Login succeeded, but secure password storage failed: {error}");
                    }
                }

                self.enter_main_shell()
            }
            Err(error) => {
                self.login.error = error;
                Vec::new()
            }
        }
    }

    fn handle_timetable_loaded(
        &mut self,
        request_id: u64,
        week_date: NaiveDate,
        target: TimetableTarget,
        result: Result<WeekTimetable, String>,
    ) -> Vec<AppCommand> {
        if request_id != self.main.timetable.request_id {
            return Vec::new();
        }

        self.main.timetable.loading = false;
        match result {
            Ok(data) => {
                self.main.timetable.data = Some(data.clone());
                self.main.timetable.is_from_cache = false;
                self.main.timetable.error.clear();
                self.sync_timetable_scroll();
                if !self.is_demo_mode() {
                    let monday = crate::models::get_monday(week_date);
                    let _ = save_week_to_cache(
                        &crate::models::format_web_date(monday),
                        &data,
                        &target_to_cache_key(Some(&target)),
                    );
                }
            }
            Err(error) => {
                self.main.timetable.error = error;
                self.sync_timetable_scroll();
            }
        }
        Vec::new()
    }

    fn handle_search_index_loaded(
        &mut self,
        profile_key: &str,
        result: Result<Vec<TimetableSearchItem>, String>,
    ) -> Vec<AppCommand> {
        if Some(profile_key) != self.saved_config.as_ref().map(build_profile_key).as_deref() {
            return Vec::new();
        }

        self.main.timetable.search_index_loading = false;
        match result {
            Ok(items) => {
                self.main.timetable.search_index = items.clone();
                self.main.timetable.search_index_error.clear();
                if let Some(session) = self.profile_sessions.get_mut(profile_key) {
                    session.search_index = items;
                }
            }
            Err(error) => self.main.timetable.search_index_error = error,
        }
        Vec::new()
    }

    fn handle_absences_loaded(
        &mut self,
        generation: u64,
        is_initial: bool,
        result: Result<AbsenceChunkPayload, String>,
    ) -> Vec<AppCommand> {
        if generation != self.main.absences.generation {
            return Vec::new();
        }
        if is_initial {
            self.main.absences.loading_initial = false;
        }
        self.main.absences.loading_more = false;

        match result {
            Ok(payload) => {
                self.main.absences.absences =
                    crate::models::merge_absences(&self.main.absences.absences, &payload.items);
                self.main.absences.chunk_index = payload.next_chunk_index;
                self.main.absences.empty_chunk_streak = payload.empty_chunk_streak;
                self.main.absences.has_more = payload.has_more;
                self.main.absences.days_loaded = payload.days_loaded;
                self.main.absences.error.clear();
                self.main.absences.selected_idx = self
                    .main
                    .absences
                    .selected_idx
                    .min(self.main.absences.absences.len().saturating_sub(1));
                return self.maybe_request_more_absences();
            }
            Err(error) => self.main.absences.error = error,
        }
        Vec::new()
    }
}

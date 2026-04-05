use crate::models::{
    Config, ParsedAbsence, SavedConfig, TimetableSearchItem, TimetableTarget, WeekTimetable,
    add_days, build_profile_key, format_timetable_target_label, get_default_timetable_target,
    target_to_cache_key, today_local,
};
use crate::shortcuts::{TabId, is_shortcut_pressed};
use crate::storage::cache::{clear_cache, get_cached_week, save_week_to_cache};
use crate::storage::config::{load_config, save_config};
use crate::storage::secret::{get_secure_storage_diagnostic, load_password, save_password};
use chrono::{Datelike, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

const CHUNK_DAYS: usize = 45;
const MAX_HISTORY_DAYS: usize = 365 * 5;
const MAX_EMPTY_CHUNK_STREAK: usize = 4;
const LOAD_MORE_BURST_CHUNKS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Loading,
    Login,
    MainShell,
}

#[derive(Debug, Clone)]
pub enum AppCommand {
    Bootstrap,
    ValidateLogin(Config),
    LoadTimetableNetwork {
        request_id: u64,
        config: Config,
        week_date: NaiveDate,
        target: TimetableTarget,
    },
    LoadSearchIndex {
        profile_key: String,
        config: Config,
    },
    LoadAbsenceChunk {
        generation: u64,
        config: Config,
        base_date: NaiveDate,
        chunk_index: usize,
        is_initial: bool,
    },
    Quit,
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    BootstrapLoaded(BootstrapPayload),
    LoginValidated(Result<Config, String>),
    TimetableLoaded {
        request_id: u64,
        week_date: NaiveDate,
        target: TimetableTarget,
        result: Result<WeekTimetable, String>,
    },
    SearchIndexLoaded {
        profile_key: String,
        result: Result<Vec<TimetableSearchItem>, String>,
    },
    AbsencesLoaded {
        generation: u64,
        is_initial: bool,
        result: Result<AbsenceChunkPayload, String>,
    },
}

#[derive(Debug, Clone)]
pub struct BootstrapPayload {
    pub saved_config: Option<SavedConfig>,
    pub saved_password: Option<String>,
    pub secure_storage_notice: String,
}

#[derive(Debug, Clone)]
pub struct AbsenceChunkPayload {
    pub items: Vec<ParsedAbsence>,
    pub next_chunk_index: usize,
    pub empty_chunk_streak: usize,
    pub has_more: bool,
    pub days_loaded: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    pub value: String,
    pub cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    Server,
    School,
    Username,
    Password,
}

#[derive(Debug, Clone)]
pub struct LoginState {
    pub server: TextInputState,
    pub school: TextInputState,
    pub username: TextInputState,
    pub password: TextInputState,
    pub active_field: LoginField,
    pub loading: bool,
    pub error: String,
    pub show_password: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    All,
    Excused,
    Unexcused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowFilter {
    All,
    D30,
    D90,
    D180,
    D365,
}

#[derive(Debug, Clone)]
pub struct TimetableState {
    pub week_offset: i32,
    pub data: Option<WeekTimetable>,
    pub loading: bool,
    pub is_from_cache: bool,
    pub error: String,
    pub active_target: TimetableTarget,
    pub selected_day_idx: usize,
    pub selected_period_idx: usize,
    pub selected_lesson_idx: usize,
    pub search_index: Vec<TimetableSearchItem>,
    pub search_index_loading: bool,
    pub search_index_error: String,
    pub search_open: bool,
    pub search_input: TextInputState,
    pub search_selected_idx: usize,
    pub request_id: u64,
}

#[derive(Debug, Clone)]
pub struct AbsencesState {
    pub absences: Vec<ParsedAbsence>,
    pub loading_initial: bool,
    pub loading_more: bool,
    pub error: String,
    pub has_more: bool,
    pub days_loaded: usize,
    pub selected_idx: usize,
    pub status_filter: StatusFilter,
    pub window_filter: WindowFilter,
    pub search_query: String,
    pub search_input: TextInputState,
    pub search_open: bool,
    pub chunk_index: usize,
    pub empty_chunk_streak: usize,
    pub generation: u64,
    pub base_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct MainState {
    pub active_tab: TabId,
    pub settings_open: bool,
    pub timetable: TimetableState,
    pub absences: AbsencesState,
}

#[derive(Debug, Clone)]
struct ProfileSessionState {
    active_target: TimetableTarget,
    search_index: Vec<TimetableSearchItem>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub saved_config: Option<SavedConfig>,
    pub saved_password: Option<String>,
    pub config: Option<Config>,
    pub app_error: String,
    pub secure_storage_notice: String,
    pub login: LoginState,
    pub main: MainState,
    pub terminal_width: u16,
    pub terminal_height: u16,
    profile_sessions: HashMap<String, ProfileSessionState>,
    next_request_id: u64,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            server: TextInputState::default(),
            school: TextInputState::default(),
            username: TextInputState::default(),
            password: TextInputState::default(),
            active_field: LoginField::Server,
            loading: false,
            error: String::new(),
            show_password: false,
        }
    }
}

impl Default for TimetableState {
    fn default() -> Self {
        let weekday = today_local().weekday().number_from_monday();
        Self {
            week_offset: 0,
            data: None,
            loading: false,
            is_from_cache: false,
            error: String::new(),
            active_target: get_default_timetable_target(),
            selected_day_idx: (weekday.saturating_sub(1) as usize).min(4),
            selected_period_idx: 0,
            selected_lesson_idx: 0,
            search_index: Vec::new(),
            search_index_loading: false,
            search_index_error: String::new(),
            search_open: false,
            search_input: TextInputState::default(),
            search_selected_idx: 0,
            request_id: 0,
        }
    }
}

impl Default for AbsencesState {
    fn default() -> Self {
        Self {
            absences: Vec::new(),
            loading_initial: false,
            loading_more: false,
            error: String::new(),
            has_more: true,
            days_loaded: 0,
            selected_idx: 0,
            status_filter: StatusFilter::All,
            window_filter: WindowFilter::All,
            search_query: String::new(),
            search_input: TextInputState::default(),
            search_open: false,
            chunk_index: 0,
            empty_chunk_streak: 0,
            generation: 0,
            base_date: today_local(),
        }
    }
}

impl Default for MainState {
    fn default() -> Self {
        Self {
            active_tab: TabId::Timetable,
            settings_open: false,
            timetable: TimetableState::default(),
            absences: AbsencesState::default(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Loading,
            saved_config: None,
            saved_password: None,
            config: None,
            app_error: String::new(),
            secure_storage_notice: String::new(),
            login: LoginState::default(),
            main: MainState::default(),
            terminal_width: 120,
            terminal_height: 24,
            profile_sessions: HashMap::new(),
            next_request_id: 1,
        }
    }
}

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
            self.screen = Screen::MainShell;
            return self.enter_main_shell();
        }

        self.screen = Screen::Login;
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
                self.screen = Screen::MainShell;

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
                let monday = crate::models::get_monday(week_date);
                let _ = save_week_to_cache(
                    &crate::models::format_web_date(monday),
                    &data,
                    &target_to_cache_key(Some(&target)),
                );
            }
            Err(error) => self.main.timetable.error = error,
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

    fn hydrate_login_form(&mut self) {
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

impl AppState {
    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match self.screen {
            Screen::Loading => {
                if is_shortcut_pressed("quit", key) {
                    vec![AppCommand::Quit]
                } else {
                    Vec::new()
                }
            }
            Screen::Login => self.handle_login_key(key),
            Screen::MainShell => self.handle_main_key(key),
        }
    }

    fn handle_login_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if self.login.loading {
            return Vec::new();
        }

        if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT) {
            self.login.active_field = match self.login.active_field {
                LoginField::Server => LoginField::Server,
                LoginField::School => LoginField::Server,
                LoginField::Username => LoginField::School,
                LoginField::Password => LoginField::Username,
            };
            return Vec::new();
        }

        if key.code == KeyCode::Tab || key.code == KeyCode::Down {
            self.login.active_field = match self.login.active_field {
                LoginField::Server => LoginField::School,
                LoginField::School => LoginField::Username,
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Password,
            };
            return Vec::new();
        }

        if key.code == KeyCode::Up {
            self.login.active_field = match self.login.active_field {
                LoginField::Server => LoginField::Server,
                LoginField::School => LoginField::Server,
                LoginField::Username => LoginField::School,
                LoginField::Password => LoginField::Username,
            };
            return Vec::new();
        }

        if is_shortcut_pressed("login-toggle-password", key) {
            self.login.show_password = !self.login.show_password;
            return Vec::new();
        }

        if is_shortcut_pressed("login-saved", key) {
            if let Some(config) = self.saved_login_config() {
                self.login.loading = true;
                self.login.error.clear();
                return vec![AppCommand::ValidateLogin(config)];
            }
            return Vec::new();
        }

        if key.code == KeyCode::Enter {
            if self.login.active_field != LoginField::Password {
                self.login.active_field = match self.login.active_field {
                    LoginField::Server => LoginField::School,
                    LoginField::School => LoginField::Username,
                    LoginField::Username => LoginField::Password,
                    LoginField::Password => LoginField::Password,
                };
                return Vec::new();
            }
            return self.submit_login();
        }

        self.current_login_input_mut().handle_key(key);
        Vec::new()
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if self.main.settings_open {
            if is_shortcut_pressed("settings-close", key) {
                self.main.settings_open = false;
            }
            return Vec::new();
        }

        if self.main.timetable.search_open {
            return self.handle_timetable_search_key(key);
        }

        if self.main.absences.search_open {
            return self.handle_absence_search_key(key);
        }

        if is_shortcut_pressed("settings-open", key) {
            self.main.settings_open = true;
            return Vec::new();
        }

        if is_shortcut_pressed("tab-prev", key) {
            self.main.active_tab = match self.main.active_tab {
                TabId::Timetable => TabId::Absences,
                TabId::Absences => TabId::Timetable,
            };
            return Vec::new();
        }

        if is_shortcut_pressed("tab-next", key) {
            self.main.active_tab = match self.main.active_tab {
                TabId::Timetable => TabId::Absences,
                TabId::Absences => TabId::Timetable,
            };
            return Vec::new();
        }

        if is_shortcut_pressed("tab-timetable", key) {
            self.main.active_tab = TabId::Timetable;
            return Vec::new();
        }

        if is_shortcut_pressed("tab-absences", key) {
            self.main.active_tab = TabId::Absences;
            return Vec::new();
        }

        if is_shortcut_pressed("quit", key) {
            return vec![AppCommand::Quit];
        }

        if is_shortcut_pressed("logout", key) {
            self.perform_logout();
            return Vec::new();
        }

        match self.main.active_tab {
            TabId::Timetable => self.handle_timetable_key(key),
            TabId::Absences => self.handle_absences_key(key),
        }
    }

    fn handle_timetable_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if is_shortcut_pressed("timetable-search", key) {
            self.main.timetable.search_open = true;
            if self.main.timetable.search_index.is_empty()
                && !self.main.timetable.search_index_loading
            {
                return self.request_search_index();
            }
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-target-clear", key) {
            if self.main.timetable.active_target != TimetableTarget::Own {
                self.main.timetable.active_target = TimetableTarget::Own;
                self.persist_profile_session();
                return self.request_timetable(false);
            }
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-week-prev", key) {
            self.main.timetable.week_offset -= 1;
            self.main.timetable.selected_period_idx = 0;
            self.main.timetable.selected_lesson_idx = 0;
            return self.request_timetable(false);
        }

        if is_shortcut_pressed("timetable-week-next", key) {
            self.main.timetable.week_offset += 1;
            self.main.timetable.selected_period_idx = 0;
            self.main.timetable.selected_lesson_idx = 0;
            return self.request_timetable(false);
        }

        if is_shortcut_pressed("timetable-day-prev", key) {
            self.main.timetable.selected_day_idx =
                self.main.timetable.selected_day_idx.saturating_sub(1);
            self.ensure_timetable_selection_bounds();
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-day-next", key) {
            self.main.timetable.selected_day_idx =
                (self.main.timetable.selected_day_idx + 1).min(4);
            self.ensure_timetable_selection_bounds();
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-up", key) || is_shortcut_pressed("timetable-up-step", key)
        {
            self.move_timetable_selection(-1, !is_shortcut_pressed("timetable-up-step", key));
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-down", key)
            || is_shortcut_pressed("timetable-down-step", key)
        {
            self.move_timetable_selection(1, !is_shortcut_pressed("timetable-down-step", key));
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-page-up", key) {
            let jump = self.timetable_rows_per_page().max(1) as isize - 1;
            self.move_timetable_selection(-jump, true);
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-page-down", key) {
            let jump = self.timetable_rows_per_page().max(1) as isize - 1;
            self.move_timetable_selection(jump, true);
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-home", key) {
            self.main.timetable.selected_period_idx = self.find_timetable_edge_period(true);
            self.ensure_timetable_selection_bounds();
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-end", key) {
            self.main.timetable.selected_period_idx = self.find_timetable_edge_period(false);
            self.ensure_timetable_selection_bounds();
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-cycle-overlap", key) {
            let count = self.current_timetable_period_lessons().len();
            if count > 1 {
                self.main.timetable.selected_lesson_idx =
                    (self.main.timetable.selected_lesson_idx + 1) % count;
            }
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-today", key) {
            self.main.timetable.week_offset = 0;
            self.main.timetable.selected_day_idx = (today_local()
                .weekday()
                .number_from_monday()
                .saturating_sub(1) as usize)
                .min(4);
            self.main.timetable.selected_lesson_idx = 0;
            return self.request_timetable(false);
        }

        if is_shortcut_pressed("timetable-refresh", key) {
            return self.request_timetable(true);
        }

        Vec::new()
    }

    fn handle_timetable_search_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.code == KeyCode::Esc {
            self.main.timetable.search_open = false;
            return Vec::new();
        }
        if key.code == KeyCode::Up {
            self.main.timetable.search_selected_idx =
                self.main.timetable.search_selected_idx.saturating_sub(1);
            return Vec::new();
        }
        if key.code == KeyCode::Down {
            let max_index = self.timetable_search_results().len().saturating_sub(1);
            self.main.timetable.search_selected_idx =
                (self.main.timetable.search_selected_idx + 1).min(max_index);
            return Vec::new();
        }
        if key.code == KeyCode::Enter {
            let results = self.timetable_search_results();
            if let Some(selected) = results.get(self.main.timetable.search_selected_idx) {
                self.main.timetable.active_target = match selected.r#type {
                    crate::models::TimetableSearchTargetType::Class => TimetableTarget::Class {
                        id: selected.id,
                        name: selected.name.clone(),
                        long_name: selected.long_name.clone(),
                    },
                    crate::models::TimetableSearchTargetType::Room => TimetableTarget::Room {
                        id: selected.id,
                        name: selected.name.clone(),
                        long_name: selected.long_name.clone(),
                    },
                    crate::models::TimetableSearchTargetType::Teacher => TimetableTarget::Teacher {
                        id: selected.id,
                        name: selected.name.clone(),
                        long_name: selected.long_name.clone(),
                    },
                };
                self.main.timetable.search_open = false;
                self.persist_profile_session();
                return self.request_timetable(false);
            }
            self.main.timetable.search_open = false;
            return Vec::new();
        }

        self.main.timetable.search_input.handle_key(key);
        self.main.timetable.search_selected_idx = 0;
        Vec::new()
    }

    fn handle_absences_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if is_shortcut_pressed("absences-refresh", key) {
            self.main.absences.selected_idx = 0;
            return self.request_absences_refresh();
        }
        if is_shortcut_pressed("absences-load-more", key) {
            return self.request_absences_more();
        }
        if is_shortcut_pressed("absences-status", key) {
            self.main.absences.status_filter = match self.main.absences.status_filter {
                StatusFilter::All => StatusFilter::Excused,
                StatusFilter::Excused => StatusFilter::Unexcused,
                StatusFilter::Unexcused => StatusFilter::All,
            };
            self.main.absences.selected_idx = 0;
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-window", key) {
            self.main.absences.window_filter = match self.main.absences.window_filter {
                WindowFilter::All => WindowFilter::D30,
                WindowFilter::D30 => WindowFilter::D90,
                WindowFilter::D90 => WindowFilter::D180,
                WindowFilter::D180 => WindowFilter::D365,
                WindowFilter::D365 => WindowFilter::All,
            };
            self.main.absences.selected_idx = 0;
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-clear", key) {
            self.main.absences.status_filter = StatusFilter::All;
            self.main.absences.window_filter = WindowFilter::All;
            self.main.absences.search_query.clear();
            self.main.absences.search_input.value.clear();
            self.main.absences.search_input.cursor = 0;
            self.main.absences.selected_idx = 0;
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-search", key) {
            self.main.absences.search_open = true;
            self.main.absences.search_input =
                TextInputState::from(self.main.absences.search_query.clone());
            return Vec::new();
        }
        if is_shortcut_pressed("absences-up", key) {
            self.main.absences.selected_idx = self.main.absences.selected_idx.saturating_sub(1);
            return Vec::new();
        }
        if is_shortcut_pressed("absences-down", key) {
            let max = self.filtered_absences().len().saturating_sub(1);
            self.main.absences.selected_idx = (self.main.absences.selected_idx + 1).min(max);
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-page-up", key) {
            self.main.absences.selected_idx = self
                .main
                .absences
                .selected_idx
                .saturating_sub(self.absences_page_jump());
            return Vec::new();
        }
        if is_shortcut_pressed("absences-page-down", key) {
            let max = self.filtered_absences().len().saturating_sub(1);
            self.main.absences.selected_idx =
                (self.main.absences.selected_idx + self.absences_page_jump()).min(max);
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-home", key) {
            self.main.absences.selected_idx = 0;
            return Vec::new();
        }
        if is_shortcut_pressed("absences-end", key) {
            self.main.absences.selected_idx = self.filtered_absences().len().saturating_sub(1);
            return self.request_absences_more();
        }
        Vec::new()
    }

    fn handle_absence_search_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.code == KeyCode::Esc {
            self.main.absences.search_open = false;
            self.main.absences.search_input =
                TextInputState::from(self.main.absences.search_query.clone());
            return Vec::new();
        }
        if key.code == KeyCode::Enter {
            self.main.absences.search_query =
                self.main.absences.search_input.value.trim().to_owned();
            self.main.absences.search_open = false;
            self.main.absences.selected_idx = 0;
            return self.maybe_request_more_absences();
        }
        self.main.absences.search_input.handle_key(key);
        Vec::new()
    }

    fn current_login_input_mut(&mut self) -> &mut TextInputState {
        match self.login.active_field {
            LoginField::Server => &mut self.login.server,
            LoginField::School => &mut self.login.school,
            LoginField::Username => &mut self.login.username,
            LoginField::Password => &mut self.login.password,
        }
    }

    fn submit_login(&mut self) -> Vec<AppCommand> {
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

    fn enter_main_shell(&mut self) -> Vec<AppCommand> {
        let saved = match self.config.as_ref() {
            Some(config) => config.saved(),
            None => return Vec::new(),
        };
        let profile_key = build_profile_key(&saved);
        let session =
            self.profile_sessions
                .get(&profile_key)
                .cloned()
                .unwrap_or(ProfileSessionState {
                    active_target: get_default_timetable_target(),
                    search_index: Vec::new(),
                });

        self.main = MainState::default();
        self.main.timetable.active_target = session.active_target;
        self.main.timetable.search_index = session.search_index;
        self.main.absences.base_date = today_local();
        let mut commands = self.request_timetable(false);
        commands.extend(self.request_absences_refresh());
        commands
    }

    fn request_search_index(&mut self) -> Vec<AppCommand> {
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

    fn request_timetable(&mut self, force_refresh: bool) -> Vec<AppCommand> {
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

    fn request_absences_refresh(&mut self) -> Vec<AppCommand> {
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

    fn request_absences_more(&mut self) -> Vec<AppCommand> {
        if self.main.absences.loading_initial
            || self.main.absences.loading_more
            || !self.main.absences.has_more
        {
            return Vec::new();
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

    fn maybe_request_more_absences(&mut self) -> Vec<AppCommand> {
        let filtered_len = self.filtered_absences().len();
        let prefetch_threshold = self.absences_page_jump().max(6);
        let maintain_prefetch = self.main.absences.status_filter == StatusFilter::All
            && self.main.absences.window_filter == WindowFilter::All
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

    fn persist_profile_session(&mut self) {
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
            ProfileSessionState {
                active_target: self.main.timetable.active_target.clone(),
                search_index: self.main.timetable.search_index.clone(),
            },
        );
    }

    fn perform_logout(&mut self) {
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
        self.screen = Screen::Login;
        self.hydrate_login_form();
    }

    fn timetable_rows_per_page(&self) -> usize {
        let height = self.terminal_height.saturating_sub(10);
        usize::from(height.max(1))
    }

    fn absences_page_jump(&self) -> usize {
        usize::from(self.terminal_height.saturating_sub(11)).max(4) / 2
    }

    fn current_timetable_period_lessons(&self) -> Vec<crate::models::ParsedLesson> {
        lessons_for_period(
            self.main.timetable.data.as_ref(),
            self.main.timetable.selected_day_idx,
            self.main.timetable.selected_period_idx,
        )
    }

    fn ensure_timetable_selection_bounds(&mut self) {
        let count = self.current_timetable_period_lessons().len();
        self.main.timetable.selected_lesson_idx = self
            .main
            .timetable
            .selected_lesson_idx
            .min(count.saturating_sub(1));
    }

    fn move_timetable_selection(&mut self, delta: isize, jump_to_lesson: bool) {
        let max_period = self
            .main
            .timetable
            .data
            .as_ref()
            .map(|data| data.timegrid.len().saturating_sub(1))
            .unwrap_or(0);
        let mut next = self.main.timetable.selected_period_idx as isize + delta;
        next = next.clamp(0, max_period as isize);
        let direction = if delta.is_negative() { -1 } else { 1 };

        if jump_to_lesson {
            while next >= 0 && next <= max_period as isize {
                if !lessons_for_period(
                    self.main.timetable.data.as_ref(),
                    self.main.timetable.selected_day_idx,
                    next as usize,
                )
                .is_empty()
                {
                    break;
                }
                next += direction;
            }
            next = next.clamp(0, max_period as isize);
        }

        self.main.timetable.selected_period_idx = next as usize;
        self.ensure_timetable_selection_bounds();
    }

    fn find_timetable_edge_period(&self, from_start: bool) -> usize {
        let Some(data) = &self.main.timetable.data else {
            return 0;
        };
        if from_start {
            for index in 0..data.timegrid.len() {
                if !lessons_for_period(Some(data), self.main.timetable.selected_day_idx, index)
                    .is_empty()
                {
                    return index;
                }
            }
            0
        } else {
            for index in (0..data.timegrid.len()).rev() {
                if !lessons_for_period(Some(data), self.main.timetable.selected_day_idx, index)
                    .is_empty()
                {
                    return index;
                }
            }
            data.timegrid.len().saturating_sub(1)
        }
    }

    pub fn filtered_absences(&self) -> Vec<ParsedAbsence> {
        let cutoff = self.main.absences.window_filter.cutoff_date();
        let query = self.main.absences.search_query.trim().to_lowercase();

        self.main
            .absences
            .absences
            .iter()
            .filter(|absence| match self.main.absences.status_filter {
                StatusFilter::All => true,
                StatusFilter::Excused => absence.is_excused,
                StatusFilter::Unexcused => !absence.is_excused,
            })
            .filter(|absence| cutoff.map(|date| absence.end_date >= date).unwrap_or(true))
            .filter(|absence| {
                if query.is_empty() {
                    return true;
                }
                format!(
                    "{} {} {} {} {} {}",
                    absence.student_name,
                    absence.reason,
                    absence.text,
                    absence.excuse_status,
                    crate::models::format_date(absence.start_date),
                    crate::models::format_date(absence.end_date)
                )
                .to_lowercase()
                .contains(&query)
            })
            .cloned()
            .collect()
    }

    pub fn has_active_absence_filters(&self) -> bool {
        self.main.absences.status_filter != StatusFilter::All
            || self.main.absences.window_filter != WindowFilter::All
            || !self.main.absences.search_query.trim().is_empty()
    }

    pub fn timetable_search_results(&self) -> Vec<TimetableSearchItem> {
        crate::webuntis::search_timetable_targets(
            &self.main.timetable.search_index,
            &self.main.timetable.search_input.value,
            None,
        )
    }

    pub fn current_timetable_lessons(&self) -> Vec<crate::models::ParsedLesson> {
        self.current_timetable_period_lessons()
    }

    pub fn timetable_lessons_for(
        &self,
        day_idx: usize,
        period_idx: usize,
    ) -> Vec<crate::models::ParsedLesson> {
        lessons_for_period(self.main.timetable.data.as_ref(), day_idx, period_idx)
    }

    pub fn selected_timetable_lesson(&self) -> Option<crate::models::ParsedLesson> {
        self.current_timetable_period_lessons()
            .get(self.main.timetable.selected_lesson_idx)
            .cloned()
    }

    pub fn selected_absence(&self) -> Option<ParsedAbsence> {
        self.filtered_absences()
            .get(self.main.absences.selected_idx)
            .cloned()
    }

    pub fn visible_absences(&self, rows: usize) -> (usize, Vec<ParsedAbsence>) {
        let filtered = self.filtered_absences();
        let visible_start = std::cmp::min(
            self.main
                .absences
                .selected_idx
                .saturating_sub(rows.saturating_div(2)),
            filtered.len().saturating_sub(rows),
        );
        (
            visible_start,
            filtered
                .into_iter()
                .skip(visible_start)
                .take(rows)
                .collect(),
        )
    }
}

impl TextInputState {
    pub fn from(value: String) -> Self {
        Self {
            cursor: value.len(),
            value,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.value.len()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.value.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Char(character)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.value.insert(self.cursor, character);
                self.cursor += character.len_utf8();
            }
            _ => {}
        }
    }
}

impl WindowFilter {
    pub fn cutoff_date(self) -> Option<NaiveDate> {
        let days = match self {
            WindowFilter::All => return None,
            WindowFilter::D30 => 30,
            WindowFilter::D90 => 90,
            WindowFilter::D180 => 180,
            WindowFilter::D365 => 365,
        };
        Some(add_days(today_local(), -(days as i64) + 1))
    }
}

impl StatusFilter {
    pub fn label(self) -> &'static str {
        match self {
            StatusFilter::All => "All",
            StatusFilter::Excused => "Excused",
            StatusFilter::Unexcused => "Unexcused",
        }
    }
}

impl WindowFilter {
    pub fn label(self) -> &'static str {
        match self {
            WindowFilter::All => "All time",
            WindowFilter::D30 => "30 days",
            WindowFilter::D90 => "90 days",
            WindowFilter::D180 => "180 days",
            WindowFilter::D365 => "365 days",
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimeUnit;

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
        state.main.timetable.data = Some(WeekTimetable {
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
}

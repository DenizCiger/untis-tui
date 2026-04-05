use crate::models::{
    Config, ParsedAbsence, SavedConfig, TimetableSearchItem, TimetableTarget, WeekTimetable,
    get_default_timetable_target, today_local,
};
use crate::shortcuts::TabId;
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

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
pub(crate) struct ProfileSessionState {
    pub active_target: TimetableTarget,
    pub search_index: Vec<TimetableSearchItem>,
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
    pub(crate) profile_sessions: HashMap<String, ProfileSessionState>,
    pub(crate) next_request_id: u64,
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

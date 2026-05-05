use super::{AppCommand, AppState, LoginField, TextInputState};
use crate::models::{TimetableTarget, today_local};
use crate::shortcuts::{TabId, get_shortcut_sections, is_shortcut_pressed};
use crate::timetable_model::{find_next_lesson_period_index, hit_test_timetable_click};
use crate::ui::{
    ShellClickTarget, TimetableTitleClickTarget, hit_test_absence_history_click,
    hit_test_shell_click, hit_test_timetable_title_click,
};
use chrono::Datelike;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use tui_components::input::login::{handle_login_key as handle_shared_login_key, LoginKeyBindings, LoginKeyOutcome};

fn next_login_field(field: LoginField) -> LoginField {
    match field {
        LoginField::Server => LoginField::School,
        LoginField::School => LoginField::Username,
        LoginField::Username => LoginField::Password,
        LoginField::Password => LoginField::Submit,
        LoginField::Submit => LoginField::Server,
    }
}

fn prev_login_field(field: LoginField) -> LoginField {
    match field {
        LoginField::Server => LoginField::Submit,
        LoginField::School => LoginField::Server,
        LoginField::Username => LoginField::School,
        LoginField::Password => LoginField::Username,
        LoginField::Submit => LoginField::Password,
    }
}

fn settings_total_lines(active_tab: TabId) -> u16 {
    get_shortcut_sections(active_tab)
        .into_iter()
        .map(|section| 1 + section.items.len() as u16 + 1)
        .sum()
}

impl AppState {
    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match self.screen {
            super::Screen::Loading => {
                if is_shortcut_pressed("quit", key) {
                    vec![AppCommand::Quit]
                } else {
                    Vec::new()
                }
            }
            super::Screen::Login => self.handle_login_key(key),
            super::Screen::MainShell => self.handle_main_key(key),
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Vec<AppCommand> {
        match self.screen {
            super::Screen::MainShell => self.handle_main_mouse(mouse),
            _ => Vec::new(),
        }
    }

    fn handle_login_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if self.login.loading {
            return Vec::new();
        }

        let mut focus = self.login.active_field;
        let outcome = {
            let input = match focus {
                LoginField::Server => Some(&mut self.login.server),
                LoginField::School => Some(&mut self.login.school),
                LoginField::Username => Some(&mut self.login.username),
                LoginField::Password => Some(&mut self.login.password),
                LoginField::Submit => None,
            };
            handle_shared_login_key(
                key,
                &mut focus,
                LoginField::Password,
                LoginField::Submit,
                input,
                next_login_field,
                prev_login_field,
                LoginKeyBindings::default(),
            )
        };
        self.login.active_field = focus;

        match outcome {
            LoginKeyOutcome::Submit => self.submit_login(),
            LoginKeyOutcome::SavedLogin => {
                if let Some(config) = self.saved_login_config() {
                    self.login.loading = true;
                    self.login.error.clear();
                    vec![AppCommand::ValidateLogin(config)]
                } else {
                    Vec::new()
                }
            }
            LoginKeyOutcome::TogglePassword => {
                self.login.show_password = !self.login.show_password;
                Vec::new()
            }
            LoginKeyOutcome::Quit => vec![AppCommand::Quit],
            _ => Vec::new(),
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if self.main.settings_open {
            if is_shortcut_pressed("settings-close", key) {
                self.main.settings_open = false;
                self.main.settings_scroll = 0;
                return Vec::new();
            }
            let total = settings_total_lines(self.main.active_tab);
            let viewport = ((self.terminal_height as f32) * 0.8) as u16;
            let viewport = viewport.saturating_sub(3);
            let max_scroll = total.saturating_sub(viewport);
            match key.code {
                KeyCode::Up => {
                    self.main.settings_scroll = self.main.settings_scroll.min(max_scroll).saturating_sub(1);
                }
                KeyCode::Down => {
                    self.main.settings_scroll = (self.main.settings_scroll + 1).min(max_scroll);
                }
                KeyCode::PageUp => {
                    self.main.settings_scroll = self.main.settings_scroll.min(max_scroll).saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.main.settings_scroll = (self.main.settings_scroll + 10).min(max_scroll);
                }
                KeyCode::Home => self.main.settings_scroll = 0,
                KeyCode::End => self.main.settings_scroll = max_scroll,
                _ => {}
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
            self.main.settings_scroll = 0;
            return Vec::new();
        }

        if is_shortcut_pressed("tab-prev", key) || is_shortcut_pressed("tab-next", key) {
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

    fn handle_main_mouse(&mut self, mouse: MouseEvent) -> Vec<AppCommand> {
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return Vec::new();
        }

        if self.main.settings_open
            || self.main.timetable.search_open
            || self.main.absences.search_open
        {
            return Vec::new();
        }

        if let Some(ShellClickTarget::Tab(tab)) = hit_test_shell_click(mouse.column, mouse.row) {
            self.main.active_tab = tab;
            return Vec::new();
        }

        match self.main.active_tab {
            TabId::Timetable => {
                if let Some(target) = hit_test_timetable_title_click(
                    self.terminal_width,
                    mouse.column,
                    mouse.row,
                    self.main.timetable.week_offset,
                ) {
                    match target {
                        TimetableTitleClickTarget::PrevWeek => {
                            self.main.timetable.week_offset -= 1;
                        }
                        TimetableTitleClickTarget::NextWeek => {
                            self.main.timetable.week_offset += 1;
                        }
                    }
                    self.main.timetable.selected_period_idx = 0;
                    self.main.timetable.selected_lesson_idx = 0;
                    self.main.timetable.scroll_offset = 0;
                    return self.request_timetable(false);
                }

                let Some(data) = self.main.timetable.data.as_ref() else {
                    return Vec::new();
                };
                let Some(model) = self.timetable_render_model() else {
                    return Vec::new();
                };

                let Some(target) = hit_test_timetable_click(
                    data,
                    &model,
                    self.terminal_width,
                    self.terminal_height,
                    self.main.timetable.scroll_offset,
                    mouse.column,
                    mouse.row,
                ) else {
                    return Vec::new();
                };

                self.main.timetable.selected_day_idx = target.day_idx;
                self.main.timetable.selected_period_idx = target.period_idx;
                self.main.timetable.selected_lesson_idx = target.lesson_idx;
                self.sync_timetable_scroll();
                Vec::new()
            }
            TabId::Absences => {
                let filtered_len = self.filtered_absences().len();
                let Some(target) = hit_test_absence_history_click(
                    self.terminal_width,
                    self.terminal_height,
                    filtered_len,
                    self.main.absences.selected_idx,
                    mouse.column,
                    mouse.row,
                ) else {
                    return Vec::new();
                };

                self.main.absences.selected_idx = target.selected_idx;
                self.maybe_request_more_absences()
            }
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
            self.main.timetable.scroll_offset = 0;
            return self.request_timetable(false);
        }

        if is_shortcut_pressed("timetable-week-next", key) {
            self.main.timetable.week_offset += 1;
            self.main.timetable.selected_period_idx = 0;
            self.main.timetable.selected_lesson_idx = 0;
            self.main.timetable.scroll_offset = 0;
            return self.request_timetable(false);
        }

        if is_shortcut_pressed("timetable-day-prev", key) {
            self.main.timetable.selected_day_idx =
                self.main.timetable.selected_day_idx.saturating_sub(1);
            self.ensure_timetable_selection_bounds();
            self.sync_timetable_scroll();
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-day-next", key) {
            self.main.timetable.selected_day_idx =
                (self.main.timetable.selected_day_idx + 1).min(4);
            self.ensure_timetable_selection_bounds();
            self.sync_timetable_scroll();
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
            let Some(data) = self.main.timetable.data.as_ref() else {
                return Vec::new();
            };
            let Some(model) = self.timetable_render_model() else {
                return Vec::new();
            };
            let target_period_idx = self
                .main
                .timetable
                .selected_period_idx
                .saturating_sub(self.timetable_rows_per_page().max(1).saturating_sub(1));
            let next_period_idx = find_next_lesson_period_index(
                &model,
                data,
                self.main.timetable.selected_day_idx,
                target_period_idx.saturating_add(1),
                -1,
            )
            .unwrap_or(target_period_idx);
            self.align_timetable_selection_to_period(next_period_idx);
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-page-down", key) {
            let Some(data) = self.main.timetable.data.as_ref() else {
                return Vec::new();
            };
            let Some(model) = self.timetable_render_model() else {
                return Vec::new();
            };
            let max_period = data.timegrid.len().saturating_sub(1);
            let target_period_idx = (self.main.timetable.selected_period_idx
                + self.timetable_rows_per_page().max(1).saturating_sub(1))
            .min(max_period);
            let search_from = target_period_idx.saturating_sub(1);
            let next_period_idx = find_next_lesson_period_index(
                &model,
                data,
                self.main.timetable.selected_day_idx,
                search_from,
                1,
            )
            .unwrap_or(target_period_idx);
            self.align_timetable_selection_to_period(next_period_idx);
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-home", key) {
            let next_period_idx = self.find_timetable_edge_period(true);
            self.align_timetable_selection_to_period(next_period_idx);
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-end", key) {
            let next_period_idx = self.find_timetable_edge_period(false);
            self.align_timetable_selection_to_period(next_period_idx);
            return Vec::new();
        }

        if is_shortcut_pressed("timetable-cycle-overlap", key) {
            self.cycle_timetable_overlap();
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
            self.main.timetable.scroll_offset = 0;
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
                super::StatusFilter::All => super::StatusFilter::Excused,
                super::StatusFilter::Excused => super::StatusFilter::Unexcused,
                super::StatusFilter::Unexcused => super::StatusFilter::All,
            };
            self.main.absences.selected_idx = 0;
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-window", key) {
            self.main.absences.window_filter = match self.main.absences.window_filter {
                super::WindowFilter::All => super::WindowFilter::D30,
                super::WindowFilter::D30 => super::WindowFilter::D90,
                super::WindowFilter::D90 => super::WindowFilter::D180,
                super::WindowFilter::D180 => super::WindowFilter::D365,
                super::WindowFilter::D365 => super::WindowFilter::All,
            };
            self.main.absences.selected_idx = 0;
            return self.maybe_request_more_absences();
        }
        if is_shortcut_pressed("absences-clear", key) {
            self.main.absences.status_filter = super::StatusFilter::All;
            self.main.absences.window_filter = super::WindowFilter::All;
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

}

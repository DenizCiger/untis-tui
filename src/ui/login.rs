use super::theme;
use crate::app::state::{AppState, LoginField};
use ratatui::Frame;
use ratatui::style::Color;
use tui_components::ui::login::{LoginFieldView, LoginModal};
use tui_components::ui::theme::Theme;

pub(super) fn render_login(frame: &mut Frame, state: &AppState) {
    LoginModal {
        title: "WebUntis TUI - Login",
        help_lines: Vec::new(),
        fields: vec![
            LoginFieldView {
                label: "Server",
                value: &state.login.server.value,
                placeholder: "e.g. mese.webuntis.com",
                focused: state.login.active_field == LoginField::Server,
                masked: false,
            },
            LoginFieldView {
                label: "School",
                value: &state.login.school.value,
                placeholder: "School from the URL",
                focused: state.login.active_field == LoginField::School,
                masked: false,
            },
            LoginFieldView {
                label: "Username",
                value: &state.login.username.value,
                placeholder: "WebUntis username",
                focused: state.login.active_field == LoginField::Username,
                masked: false,
            },
            LoginFieldView {
                label: "Password",
                value: &state.login.password.value,
                placeholder: "WebUntis password",
                focused: state.login.active_field == LoginField::Password,
                masked: !state.login.show_password,
            },
        ],
        submit_focused: state.login.active_field == LoginField::Submit,
        saved_account: state
            .saved_login_config()
            .map(|saved| format!("{}@{} ({})", saved.username, saved.school, saved.server)),
        error: if !state.app_error.is_empty() {
            Some(state.app_error.as_str())
        } else if !state.login.error.is_empty() {
            Some(state.login.error.as_str())
        } else {
            None
        },
        warning: if state.secure_storage_notice.is_empty() {
            None
        } else {
            Some(state.secure_storage_notice.as_str())
        },
        busy: state.login.loading,
        busy_label: "Logging in...",
        submit_label: "Submit",
        footer: "Tab/Shift+Tab or ↑/↓ fields · Enter submit
Alt+V show password · Ctrl+L saved login · Esc quit",
        width: 72,
        min_height: 18,
    }
    .render(frame, frame.area(), app_theme());
}

fn app_theme() -> Theme {
    Theme {
        brand: theme::BRAND,
        warning: theme::WARNING,
        error: theme::ERROR,
        success: Color::Indexed(84),
        neutral_white: theme::BRIGHT_WHITE,
        neutral_black: theme::BLACK,
        neutral_gray: theme::DIM_GRAY,
        neutral_bright_black: theme::BORDER_GRAY,
        panel_header: theme::HEADER_BG,
        panel_selected: theme::SELECT_BG,
        panel_alternate: theme::ALT_BG,
    }
}

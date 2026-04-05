use super::shared::login_field_line;
use super::theme::{BRAND, ERROR, WARNING};
use crate::app::state::{AppState, LoginField};
use ratatui::Frame;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub(super) fn render_login(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let block = Block::default()
        .title("WebUntis TUI - Login")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRAND));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from("Enter your WebUntis credentials. Use arrows or Tab to change focus."),
        Line::from("Password is stored securely via your OS credentials store."),
        Line::from(""),
        login_field_line(
            "Server",
            &state.login.server.value,
            "e.g. mese.webuntis.com",
            state.login.active_field == LoginField::Server,
            false,
        ),
        login_field_line(
            "School",
            &state.login.school.value,
            "School from the URL",
            state.login.active_field == LoginField::School,
            false,
        ),
        login_field_line(
            "Username",
            &state.login.username.value,
            "WebUntis username",
            state.login.active_field == LoginField::Username,
            false,
        ),
        login_field_line(
            "Password",
            &state.login.password.value,
            "WebUntis password",
            state.login.active_field == LoginField::Password,
            !state.login.show_password,
        ),
    ];

    if let Some(saved) = state.saved_login_config() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Saved account: ", Style::default().fg(BRAND)),
            Span::raw(format!("{}@{} ({})", saved.username, saved.school, saved.server)),
            Span::raw(" | Ctrl+l login"),
        ]));
    }

    if !state.app_error.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            state.app_error.clone(),
            Style::default().fg(ERROR),
        )));
    }
    if !state.login.error.is_empty() {
        lines.push(Line::from(Span::styled(
            state.login.error.clone(),
            Style::default().fg(ERROR),
        )));
    }
    if !state.secure_storage_notice.is_empty() {
        lines.push(Line::from(Span::styled(
            state.secure_storage_notice.clone(),
            Style::default().fg(WARNING),
        )));
    }
    if state.login.loading {
        lines.push(Line::from(Span::styled(
            "Authenticating...",
            Style::default().fg(WARNING),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Enter next/submit | Tab move focus | Ctrl+v toggle password visibility | Ctrl+l login saved",
        ));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

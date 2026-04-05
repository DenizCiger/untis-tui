use crate::app::state::{AppState, LoginField, Screen};
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::webuntis::format_timetable_search_type_label;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

const BRAND: Color = Color::Cyan;
const WARNING: Color = Color::Yellow;
const ERROR: Color = Color::Red;
const SELECT_BG: Color = Color::Blue;
const ALT_BG: Color = Color::DarkGray;

pub fn render(frame: &mut Frame, state: &AppState) {
    match state.screen {
        Screen::Loading => render_loading(frame),
        Screen::Login => render_login(frame, state),
        Screen::MainShell => render_main(frame, state),
    }
}

fn render_loading(frame: &mut Frame) {
    let area = frame.area();
    let paragraph = Paragraph::new("Loading...")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, centered_rect(40, 20, area));
}

fn render_login(frame: &mut Frame, state: &AppState) {
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
            Span::raw(format!(
                "{}@{} ({})",
                saved.username, saved.school, saved.server
            )),
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

fn render_main(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header = Line::from(vec![
        tab_span(" Timetable ", state.main.active_tab == TabId::Timetable),
        Span::raw(" "),
        tab_span(" Absences ", state.main.active_tab == TabId::Absences),
        Span::raw("   "),
        Span::styled(
            if state.main.active_tab == TabId::Timetable {
                state.timetable_target_label()
            } else {
                "Press ? for settings".to_owned()
            },
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(vec![header]), layout[0]);

    match state.main.active_tab {
        TabId::Timetable => render_timetable(frame, state, layout[1]),
        TabId::Absences => render_absences(frame, state, layout[1]),
    }

    if state.main.settings_open {
        render_shortcuts_modal(frame, state, area);
    }
    if state.main.timetable.search_open {
        render_timetable_search_popup(frame, state, area);
    }
}

fn render_timetable(frame: &mut Frame, state: &AppState, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(6),
            Constraint::Length(6),
        ])
        .split(area);

    let (monday, friday) = crate::models::current_week_range(state.main.timetable.week_offset);
    let title = format!(
        "WebUntis TUI {}{}",
        crate::models::format_date(monday),
        if monday != friday {
            format!(" - {}", crate::models::format_date(friday))
        } else {
            String::new()
        }
    );
    let mut title_lines = vec![Line::from(vec![
        Span::styled(
            title,
            Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            if state.main.timetable.is_from_cache {
                "(cached)"
            } else {
                ""
            },
            Style::default().fg(WARNING),
        ),
    ])];
    if let Some(config) = &state.config {
        title_lines.push(Line::from(format!("{}@{}", config.username, config.school)));
    }
    frame.render_widget(Paragraph::new(title_lines), layout[0]);

    frame.render_widget(
        Paragraph::new(build_timetable_lines(state))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(if state.main.timetable.loading {
                        "Loading timetable..."
                    } else {
                        "Grid"
                    }),
            )
            .wrap(Wrap { trim: false }),
        layout[1],
    );

    let details = build_timetable_details(state);
    frame.render_widget(
        Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false }),
        layout[2],
    );
}

fn render_absences(frame: &mut Frame, state: &AppState, area: Rect) {
    let split_horizontal = area.width >= 118;
    let header_height = if state.main.absences.error.is_empty() {
        3
    } else {
        4
    };
    let header = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(header_height), Constraint::Min(0)])
        .split(area);

    let filter_line = format!(
        "Status: {} | Window: {} | Search: {}",
        state.main.absences.status_filter.label(),
        state.main.absences.window_filter.label(),
        if state.main.absences.search_query.is_empty() {
            "none"
        } else {
            state.main.absences.search_query.as_str()
        }
    );
    let mut header_lines = vec![
        Line::from(Span::styled(
            "Absence Timeline",
            Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
        )),
        Line::from(filter_line),
        Line::from(if state.main.absences.search_open {
            format!(
                "Search: {}",
                render_input_text(
                    &state.main.absences.search_input.value,
                    state.main.absences.search_input.cursor,
                    false
                )
            )
        } else if state.main.absences.loading_initial {
            "Loading absences...".to_owned()
        } else if state.main.absences.loading_more {
            "Loading older records...".to_owned()
        } else {
            format!("{} days loaded", state.main.absences.days_loaded)
        }),
    ];
    if !state.main.absences.error.is_empty() {
        header_lines.push(Line::from(Span::styled(
            state.main.absences.error.clone(),
            Style::default().fg(ERROR),
        )));
    }
    frame.render_widget(Paragraph::new(header_lines), header[0]);

    let body = if split_horizontal {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(header[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(header[1])
    };

    let list_rows = usize::from(body[0].height.saturating_sub(2)).max(3);
    let (visible_start, visible) = state.visible_absences(list_rows);
    let filtered = state.filtered_absences();
    let has_active_filters = state.has_active_absence_filters();
    let has_loaded_absences = !state.main.absences.absences.is_empty();
    let list_lines = visible
        .iter()
        .enumerate()
        .map(|(offset, absence)| {
            let actual_index = visible_start + offset;
            let selected = actual_index == state.main.absences.selected_idx;
            let bg = if selected {
                Some(SELECT_BG)
            } else if actual_index % 2 == 1 {
                Some(ALT_BG)
            } else {
                None
            };
            let status = if absence.is_excused {
                "EXCUSED"
            } else {
                "UNEXCUSED"
            };
            Line::from(vec![
                styled_cell(
                    if selected { "> " } else { "  " },
                    bg,
                    Some(if selected { BRAND } else { Color::DarkGray }),
                ),
                styled_cell(
                    &fit_text(
                        &format!(
                            "{} {}-{}",
                            crate::models::format_date(absence.start_date),
                            absence.start_time,
                            absence.end_time
                        ),
                        22,
                    ),
                    bg,
                    Some(Color::Gray),
                ),
                styled_cell(" ", bg, None),
                styled_cell(
                    &fit_text(
                        if absence.text.is_empty() {
                            &absence.reason
                        } else {
                            &absence.text
                        },
                        28,
                    ),
                    bg,
                    Some(Color::White),
                ),
                styled_cell(" ", bg, None),
                styled_cell(
                    status,
                    Some(if absence.is_excused {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Some(Color::White),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let selected_count = if filtered.is_empty() {
        0
    } else {
        filtered.len().min(state.main.absences.selected_idx + 1)
    };
    let history_lines = if state.main.absences.loading_initial {
        vec![Line::from(Span::styled(
            "Loading absences...",
            Style::default().fg(WARNING),
        ))]
    } else if !state.main.absences.error.is_empty() && !has_loaded_absences {
        vec![Line::from(Span::styled(
            state.main.absences.error.clone(),
            Style::default().fg(ERROR),
        ))]
    } else if filtered.is_empty() && !has_loaded_absences {
        vec![Line::from("No absences found in loaded history.")]
    } else if filtered.is_empty() {
        vec![Line::from(if has_active_filters {
            "No absences match current filters."
        } else {
            "No absences found in loaded history."
        })]
    } else {
        list_lines
    };
    frame.render_widget(
        Paragraph::new(history_lines).block(Block::default().borders(Borders::ALL).title(format!(
            "History {}/{}",
            selected_count,
            filtered.len()
        ))),
        body[0],
    );

    let selected = state.selected_absence();
    let detail_lines = if let Some(absence) = selected {
        vec![
            Line::from(format!(
                "When: {} {} - {} {}",
                crate::models::format_date(absence.start_date),
                absence.start_time,
                crate::models::format_date(absence.end_date),
                absence.end_time
            )),
            Line::from(format!(
                "Reason: {}",
                if absence.reason.is_empty() {
                    "No reason"
                } else {
                    &absence.reason
                }
            )),
            Line::from(format!(
                "Status: {}",
                if absence.is_excused {
                    "Excused"
                } else {
                    "Unexcused"
                }
            )),
            Line::from(format!(
                "Notes: {}",
                if absence.text.is_empty() {
                    "No additional notes"
                } else {
                    &absence.text
                }
            )),
        ]
    } else {
        vec![Line::from("Select a record from the history list.")]
    };
    frame.render_widget(
        Paragraph::new(detail_lines)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false }),
        body[1],
    );
}

fn render_shortcuts_modal(frame: &mut Frame, state: &AppState, area: Rect) {
    let popup = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup);
    let inner = Block::default()
        .title("Settings")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRAND))
        .inner(popup);
    frame.render_widget(
        Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND)),
        popup,
    );
    let sections = get_shortcut_sections(state.main.active_tab);
    let mut lines = Vec::new();
    for section in sections {
        lines.push(Line::from(Span::styled(
            section.title,
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for item in section.items {
            lines.push(Line::from(format!(
                "{} - {}",
                fit_text(item.keys, 18),
                item.action
            )));
        }
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_timetable_search_popup(frame: &mut Frame, state: &AppState, area: Rect) {
    let popup = centered_rect(70, 60, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title("Timetable Target Search")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BRAND));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut lines = vec![
        Line::from(format!(
            "> {}",
            render_input_text(
                &state.main.timetable.search_input.value,
                state.main.timetable.search_input.cursor,
                false
            )
        )),
        Line::from(if state.main.timetable.search_index_loading {
            "Loading timetable targets...".to_owned()
        } else if !state.main.timetable.search_index_error.is_empty() {
            format!(
                "Target load failed: {}",
                state.main.timetable.search_index_error
            )
        } else {
            "Use ↑/↓ and Enter apply, Esc cancel.".to_owned()
        }),
        Line::from(""),
    ];

    for (index, result) in state
        .timetable_search_results()
        .into_iter()
        .take(12)
        .enumerate()
    {
        let selected = index == state.main.timetable.search_selected_idx;
        lines.push(Line::from(vec![
            Span::styled(
                if selected { "> " } else { "  " },
                Style::default().fg(if selected { BRAND } else { Color::Gray }),
            ),
            Span::styled(
                format!("[{}] ", format_timetable_search_type_label(result.r#type)),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(format!(
                "{}{}",
                result.name,
                if result.long_name != result.name {
                    format!(" ({})", result.long_name)
                } else {
                    String::new()
                }
            )),
        ]));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn build_timetable_lines(state: &AppState) -> Vec<Line<'static>> {
    let Some(data) = &state.main.timetable.data else {
        return vec![Line::from(if state.main.timetable.error.is_empty() {
            "No timetable data loaded.".to_owned()
        } else {
            state.main.timetable.error.clone()
        })];
    };

    let time_width = 13usize;
    let day_width = (((state.terminal_width as usize).saturating_sub(time_width + 6)) / 5).max(8);
    let mut lines = Vec::new();
    let mut header = vec![Span::styled(
        fit_text("Time", time_width),
        Style::default().fg(Color::Gray),
    )];
    for day in &data.days {
        header.push(Span::raw(" "));
        header.push(Span::styled(
            fit_text(&day.day_name[..day.day_name.len().min(3)], day_width),
            Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(header));

    for (period_idx, period) in data.timegrid.iter().enumerate() {
        let mut spans = vec![Span::styled(
            fit_text(
                &format!("{} {}", period.name, period.start_time),
                time_width,
            ),
            Style::default().fg(if state.main.timetable.selected_period_idx == period_idx {
                BRAND
            } else {
                WARNING
            }),
        )];
        for day_idx in 0..5 {
            let lessons = state.timetable_lessons_for(day_idx, period_idx);
            let selected = state.main.timetable.selected_day_idx == day_idx
                && state.main.timetable.selected_period_idx == period_idx;
            let cell_text = if lessons.is_empty() {
                ".".to_owned()
            } else {
                let lesson = if selected {
                    lessons
                        .get(state.main.timetable.selected_lesson_idx)
                        .cloned()
                        .unwrap_or_else(|| lessons[0].clone())
                } else {
                    lessons[0].clone()
                };
                if lessons.len() > 1 {
                    format!("{} +{}", lesson.subject, lessons.len() - 1)
                } else {
                    lesson.subject
                }
            };
            spans.push(Span::raw(" "));
            spans.push(styled_cell(
                &fit_text(&cell_text, day_width),
                if selected { Some(SELECT_BG) } else { None },
                Some(if selected { Color::White } else { Color::Gray }),
            ));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn build_timetable_details(state: &AppState) -> Vec<Line<'static>> {
    if let Some(lesson) = state.selected_timetable_lesson() {
        let overlaps = state.current_timetable_lessons().len();
        let mut lines = vec![
            Line::from(Span::styled(
                format!("{} ({})", lesson.subject_long_name, lesson.subject),
                Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("Time: {} - {}", lesson.start_time, lesson.end_time)),
            Line::from(format!(
                "Teachers: {}",
                if lesson.all_teachers.is_empty() {
                    "N/A".to_owned()
                } else {
                    lesson.all_teachers.join(", ")
                }
            )),
            Line::from(format!(
                "Room / Classes: {} / {}",
                if lesson.room.is_empty() {
                    "N/A"
                } else {
                    &lesson.room
                },
                if lesson.all_classes.is_empty() {
                    "N/A".to_owned()
                } else {
                    lesson.all_classes.join(", ")
                }
            )),
        ];
        if overlaps > 1 {
            lines.push(Line::from(format!(
                "Overlap: {}/{}",
                state.main.timetable.selected_lesson_idx + 1,
                overlaps
            )));
        }
        if !lesson.lesson_text.is_empty() {
            lines.push(Line::from(format!("Lesson text: {}", lesson.lesson_text)));
        }
        if !lesson.remarks.is_empty() {
            lines.push(Line::from(Span::styled(
                lesson.remarks,
                Style::default().fg(WARNING),
            )));
        }
        return lines;
    }
    vec![Line::from("Select a lesson to see details.")]
}

fn login_field_line(
    label: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    mask: bool,
) -> Line<'static> {
    let rendered = if value.is_empty() {
        placeholder.to_owned()
    } else if mask {
        "*".repeat(value.chars().count())
    } else {
        value.to_owned()
    };
    Line::from(vec![
        Span::styled(
            format!("{}{}: ", if focused { "> " } else { "  " }, label),
            Style::default()
                .fg(if focused { BRAND } else { Color::Gray })
                .add_modifier(if focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Span::raw(rendered),
    ])
}

fn tab_span(label: &str, active: bool) -> Span<'static> {
    Span::styled(
        label.to_owned(),
        Style::default()
            .fg(if active { Color::Black } else { Color::White })
            .bg(if active { BRAND } else { Color::DarkGray })
            .add_modifier(if active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    )
}

fn styled_cell(text: &str, bg: Option<Color>, fg: Option<Color>) -> Span<'static> {
    Span::styled(
        text.to_owned(),
        Style::default()
            .bg(bg.unwrap_or(Color::Reset))
            .fg(fg.unwrap_or(Color::Reset)),
    )
}

fn render_input_text(value: &str, cursor: usize, mask: bool) -> String {
    let value = if mask {
        "*".repeat(value.chars().count())
    } else {
        value.to_owned()
    };
    if value.is_empty() {
        return "_".to_owned();
    }
    if cursor >= value.len() {
        return format!("{value}_");
    }
    value
}

fn fit_text(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if UnicodeWidthStr::width(value) <= width {
        return format!("{value:<width$}");
    }
    let mut result = String::new();
    for character in value.chars() {
        if UnicodeWidthStr::width(result.as_str())
            + UnicodeWidthStr::width(character.encode_utf8(&mut [0; 4]))
            > width.saturating_sub(1)
        {
            break;
        }
        result.push(character);
    }
    result.push('…');
    while UnicodeWidthStr::width(result.as_str()) < width {
        result.push(' ');
    }
    result
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::AppState;
    use crate::models::{Config, SavedConfig};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn render_login_screen_shows_title() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::Login;
        state.saved_config = Some(SavedConfig {
            school: "school".into(),
            username: "user".into(),
            server: "mese.webuntis.com".into(),
        });
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("WebUntis TUI - Login"));
    }

    #[test]
    fn render_main_shell_shows_tabs() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.config = Some(Config {
            school: "school".into(),
            username: "user".into(),
            password: "secret".into(),
            server: "mese.webuntis.com".into(),
        });
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("Timetable"));
        assert!(output.contains("Absences"));
    }

    #[test]
    fn render_timetable_uses_shared_header_without_extra_timetable_block() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert_eq!(output.matches("Timetable").count(), 1);
    }

    #[test]
    fn render_absences_uses_shared_header_without_extra_absences_block() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert_eq!(output.matches("Absences").count(), 1);
        assert!(output.contains("Absence Timeline"));
    }

    #[test]
    fn render_absences_shows_loading_state() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.main.absences.loading_initial = true;
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("Loading absences..."));
    }

    #[test]
    fn render_absences_shows_backend_error_when_history_is_empty() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.main.absences.loading_initial = false;
        state.main.absences.error = "Failed to fetch absences".into();
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("Failed to fetch absences"));
    }

    #[test]
    fn render_absences_shows_neutral_empty_message_without_filters() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.main.absences.loading_initial = false;
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("No absences found in loaded history."));
    }

    #[test]
    fn render_absences_shows_filter_empty_message_when_records_are_filtered_out() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.main.absences.loading_initial = false;
        state.main.absences.search_query = "zzz".into();
        state
            .main
            .absences
            .absences
            .push(crate::models::ParsedAbsence {
                id: 1,
                student_name: "User".into(),
                reason: "Ill".into(),
                text: String::new(),
                excuse_status: "Open".into(),
                is_excused: false,
                start_date: chrono::Local::now().date_naive(),
                end_date: chrono::Local::now().date_naive(),
                start_time: "08:00".into(),
                end_time: "08:50".into(),
            });
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("No absences match current filters."));
    }

    #[test]
    fn render_absences_shows_history_and_details_when_records_exist() {
        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.main.absences.loading_initial = false;
        state
            .main
            .absences
            .absences
            .push(crate::models::ParsedAbsence {
                id: 1,
                student_name: "User".into(),
                reason: "Ill".into(),
                text: "Doctor".into(),
                excuse_status: "Excused".into(),
                is_excused: true,
                start_date: chrono::Local::now().date_naive(),
                end_date: chrono::Local::now().date_naive(),
                start_time: "08:00".into(),
                end_time: "08:50".into(),
            });
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("History 1/1"));
        assert!(output.contains("Reason: Ill"));
    }
}

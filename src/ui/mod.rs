use crate::app::state::{ AppState, LoginField, Screen };
use crate::shortcuts::{ TabId, get_shortcut_sections };
use crate::webuntis::format_timetable_search_type_label;
use chrono::Datelike;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::{ Constraint, Direction, Layout, Rect };
use ratatui::style::{ Color, Modifier, Style };
use ratatui::text::{ Line, Span };
use ratatui::widgets::{ Block, Borders, Clear, Paragraph, Wrap };
use unicode_width::UnicodeWidthStr;

const BRAND: Color = Color::Indexed(45);
const WARNING: Color = Color::Indexed(220);
const ERROR: Color = Color::Indexed(196);
const SELECT_BG: Color = Color::Indexed(24);
const ALT_BG: Color = Color::Indexed(236);
const DIM_GRAY: Color = Color::Indexed(244);
const BORDER_GRAY: Color = Color::Indexed(240);
const HEADER_BG: Color = Color::Indexed(238);
const EXCUSED_BG: Color = Color::Indexed(35);
const UNEXCUSED_BG: Color = Color::Indexed(167);

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
            false
        ),
        login_field_line(
            "School",
            &state.login.school.value,
            "School from the URL",
            state.login.active_field == LoginField::School,
            false
        ),
        login_field_line(
            "Username",
            &state.login.username.value,
            "WebUntis username",
            state.login.active_field == LoginField::Username,
            false
        ),
        login_field_line(
            "Password",
            &state.login.password.value,
            "WebUntis password",
            state.login.active_field == LoginField::Password,
            !state.login.show_password
        )
    ];

    if let Some(saved) = state.saved_login_config() {
        lines.push(Line::from(""));
        lines.push(
            Line::from(
                vec![
                    Span::styled("Saved account: ", Style::default().fg(BRAND)),
                    Span::raw(format!("{}@{} ({})", saved.username, saved.school, saved.server)),
                    Span::raw(" | Ctrl+l login")
                ]
            )
        );
    }

    if !state.app_error.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(state.app_error.clone(), Style::default().fg(ERROR))));
    }
    if !state.login.error.is_empty() {
        lines.push(Line::from(Span::styled(state.login.error.clone(), Style::default().fg(ERROR))));
    }
    if !state.secure_storage_notice.is_empty() {
        lines.push(
            Line::from(
                Span::styled(state.secure_storage_notice.clone(), Style::default().fg(WARNING))
            )
        );
    }
    if state.login.loading {
        lines.push(Line::from(Span::styled("Authenticating...", Style::default().fg(WARNING))));
    } else {
        lines.push(Line::from(""));
        lines.push(
            Line::from(
                "Enter next/submit | Tab move focus | Ctrl+v toggle password visibility | Ctrl+l login saved"
            )
        );
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_main(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header = Line::from(
        vec![
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
                Style::default().fg(Color::Gray)
            )
        ]
    );
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
        .constraints([Constraint::Length(2), Constraint::Min(6), Constraint::Length(6)])
        .split(area);

    let (monday, friday) = crate::models::current_week_range(state.main.timetable.week_offset);
    let title = format!("WebUntis TUI {}{}", crate::models::format_date(monday), if
        monday != friday
    {
        format!(" - {}", crate::models::format_date(friday))
    } else {
        String::new()
    });
    let mut title_lines = vec![
        Line::from(
            vec![
                Span::styled(title, Style::default().fg(BRAND).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(
                    if state.main.timetable.is_from_cache {
                        "(cached)"
                    } else {
                        ""
                    },
                    Style::default().fg(WARNING)
                )
            ]
        )
    ];
    if let Some(config) = &state.config {
        title_lines.push(Line::from(format!("{}@{}", config.username, config.school)));
    }
    frame.render_widget(Paragraph::new(title_lines), layout[0]);

    frame.render_widget(
        Paragraph::new(build_timetable_lines(state))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(
                        if state.main.timetable.loading {
                            "Loading timetable..."
                        } else {
                            "Grid"
                        }
                    )
            )
            .wrap(Wrap { trim: false }),
        layout[1]
    );

    let details = build_timetable_details(state);
    frame.render_widget(
        Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false }),
        layout[2]
    );
}

fn render_absences(frame: &mut Frame, state: &AppState, area: Rect) {
    let split_horizontal = area.width >= 118;
    let compact_header = area.width < 96;
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let filtered = state.filtered_absences();
    let selected = state.selected_absence();
    let selected_count = if filtered.is_empty() {
        0
    } else {
        filtered.len().min(state.main.absences.selected_idx + 1)
    };
    let filtered_excused = filtered
        .iter()
        .filter(|absence| absence.is_excused)
        .count();
    let filtered_unexcused = filtered.len().saturating_sub(filtered_excused);
    let has_active_filters = state.has_active_absence_filters();
    let has_loaded_absences = !state.main.absences.absences.is_empty();
    let list_summary = absence_list_summary(state, &filtered);
    let history_label = absence_history_load_label(state);
    let newest_loaded = state.main.absences.absences
        .first()
        .map(|absence| crate::models::format_date(absence.start_date))
        .unwrap_or_else(|| "-".to_owned());
    let oldest_loaded = state.main.absences.absences
        .last()
        .map(|absence| crate::models::format_date(absence.start_date))
        .unwrap_or_else(|| "-".to_owned());

    frame.render_widget(
        Paragraph::new(
            line_with_right(
                "Absence Timeline",
                &state.config
                    .as_ref()
                    .map(|config| format!("{}@{}", config.username, config.school))
                    .unwrap_or_default(),
                usize::from(layout[0].width),
                Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
                Style::default().fg(DIM_GRAY)
            )
        ),
        layout[0]
    );
    frame.render_widget(
        Paragraph::new(
            line_with_right(
                &format!("Newest first | {list_summary}"),
                &format!("{} days loaded | {history_label}", state.main.absences.days_loaded),
                usize::from(layout[1].width),
                Style::default().fg(DIM_GRAY),
                Style::default().fg(DIM_GRAY)
            )
        ),
        layout[1]
    );

    let filter_line = if compact_header {
        Line::from(
            Span::styled(
                format!(
                    "[f:{}] [w:{}] [/:{}] [c]",
                    state.main.absences.status_filter.label(),
                    state.main.absences.window_filter.label(),
                    if state.main.absences.search_query.is_empty() {
                        "none"
                    } else {
                        state.main.absences.search_query.as_str()
                    }
                ),
                Style::default().fg(DIM_GRAY)
            )
        )
    } else {
        Line::from(
            vec![
                filter_chip(
                    &format!("Status: {}", state.main.absences.status_filter.label()),
                    state.main.absences.status_filter.label() != "All"
                ),
                Span::raw(" "),
                filter_chip(
                    &format!("Window: {}", state.main.absences.window_filter.label()),
                    state.main.absences.window_filter.label() != "All time"
                ),
                Span::raw(" "),
                filter_chip(
                    &format!(
                        "Search: {}",
                        truncate_text(
                            if state.main.absences.search_query.is_empty() {
                                "none"
                            } else {
                                state.main.absences.search_query.as_str()
                            },
                            18
                        )
                    ),
                    !state.main.absences.search_query.is_empty()
                ),
                Span::raw(" "),
                filter_chip("Clear", false)
            ]
        )
    };
    frame.render_widget(Paragraph::new(filter_line), layout[2]);

    let hint_line = if state.main.absences.search_open {
        Line::from(
            vec![
                Span::styled("Search: ", Style::default().fg(BRAND)),
                Span::raw(
                    render_input_text(
                        &state.main.absences.search_input.value,
                        state.main.absences.search_input.cursor,
                        false
                    )
                )
            ]
        )
    } else {
        Line::from(
            Span::styled(
                absence_prefetch_hint(state, filtered.len()),
                Style::default().fg(DIM_GRAY)
            )
        )
    };
    frame.render_widget(Paragraph::new(hint_line), layout[3]);

    let (history_area, summary_area, details_area) = if split_horizontal {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(((f32::from(layout[4].width) * 0.6).floor() as u16).max(56)),
                Constraint::Min(28),
            ])
            .split(layout[4]);
        let right_height = body[1].height.max(9);
        let summary_height = right_height.saturating_sub(5).max(4).min(5);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(summary_height), Constraint::Min(5)])
            .split(body[1]);
        (body[0], right[0], right[1])
    } else {
        let summary_height = layout[4].height.saturating_sub(5).max(4).min(5);
        let stacked = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(58),
                Constraint::Length(summary_height),
                Constraint::Min(5),
            ])
            .split(layout[4]);
        (stacked[0], stacked[1], stacked[2])
    };

    render_absence_history_pane(
        frame,
        state,
        history_area,
        &filtered,
        selected_count,
        has_active_filters,
        has_loaded_absences
    );
    render_absence_summary_pane(
        frame,
        summary_area,
        filtered_excused,
        filtered_unexcused,
        &state.main.absences.window_filter.label(),
        &newest_loaded,
        &oldest_loaded
    );
    render_absence_details_pane(frame, details_area, selected);
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
        popup
    );
    let sections = get_shortcut_sections(state.main.active_tab);
    let mut lines = Vec::new();
    for section in sections {
        lines.push(
            Line::from(Span::styled(section.title, Style::default().add_modifier(Modifier::BOLD)))
        );
        for item in section.items {
            lines.push(Line::from(format!("{} - {}", fit_text(item.keys, 18), item.action)));
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
        Line::from(
            format!(
                "> {}",
                render_input_text(
                    &state.main.timetable.search_input.value,
                    state.main.timetable.search_input.cursor,
                    false
                )
            )
        ),
        Line::from(
            if state.main.timetable.search_index_loading {
                "Loading timetable targets...".to_owned()
            } else if !state.main.timetable.search_index_error.is_empty() {
                format!("Target load failed: {}", state.main.timetable.search_index_error)
            } else {
                "Use ↑/↓ and Enter apply, Esc cancel.".to_owned()
            }
        ),
        Line::from("")
    ];

    for (index, result) in state.timetable_search_results().into_iter().take(12).enumerate() {
        let selected = index == state.main.timetable.search_selected_idx;
        lines.push(
            Line::from(
                vec![
                    Span::styled(
                        if selected {
                            "> "
                        } else {
                            "  "
                        },
                        Style::default().fg(if selected { BRAND } else { Color::Gray })
                    ),
                    Span::styled(
                        format!("[{}] ", format_timetable_search_type_label(result.r#type)),
                        Style::default().fg(Color::Gray)
                    ),
                    Span::raw(
                        format!("{}{}", result.name, if result.long_name != result.name {
                            format!(" ({})", result.long_name)
                        } else {
                            String::new()
                        })
                    )
                ]
            )
        );
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn build_timetable_lines(state: &AppState) -> Vec<Line<'static>> {
    let Some(data) = &state.main.timetable.data else {
        return vec![
            Line::from(
                if state.main.timetable.error.is_empty() {
                    "No timetable data loaded.".to_owned()
                } else {
                    state.main.timetable.error.clone()
                }
            )
        ];
    };

    let time_width = 13usize;
    let day_width = ((state.terminal_width as usize).saturating_sub(time_width + 6) / 5).max(8);
    let mut lines = Vec::new();
    let mut header = vec![
        Span::styled(fit_text("Time", time_width), Style::default().fg(Color::Gray))
    ];
    for day in &data.days {
        header.push(Span::raw(" "));
        header.push(
            Span::styled(
                fit_text(&day.day_name[..day.day_name.len().min(3)], day_width),
                Style::default().fg(BRAND).add_modifier(Modifier::BOLD)
            )
        );
    }
    lines.push(Line::from(header));

    for (period_idx, period) in data.timegrid.iter().enumerate() {
        let mut spans = vec![
            Span::styled(
                fit_text(&format!("{} {}", period.name, period.start_time), time_width),
                Style::default().fg(
                    if state.main.timetable.selected_period_idx == period_idx {
                        BRAND
                    } else {
                        WARNING
                    }
                )
            )
        ];
        for day_idx in 0..5 {
            let lessons = state.timetable_lessons_for(day_idx, period_idx);
            let selected =
                state.main.timetable.selected_day_idx == day_idx &&
                state.main.timetable.selected_period_idx == period_idx;
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
            spans.push(
                styled_cell(
                    &fit_text(&cell_text, day_width),
                    if selected {
                        Some(SELECT_BG)
                    } else {
                        None
                    },
                    Some(if selected { Color::White } else { Color::Gray })
                )
            );
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn build_timetable_details(state: &AppState) -> Vec<Line<'static>> {
    if let Some(lesson) = state.selected_timetable_lesson() {
        let overlaps = state.current_timetable_lessons().len();
        let mut lines = vec![
            Line::from(
                Span::styled(
                    format!("{} ({})", lesson.subject_long_name, lesson.subject),
                    Style::default().fg(BRAND).add_modifier(Modifier::BOLD)
                )
            ),
            Line::from(format!("Time: {} - {}", lesson.start_time, lesson.end_time)),
            Line::from(
                format!("Teachers: {}", if lesson.all_teachers.is_empty() {
                    "N/A".to_owned()
                } else {
                    lesson.all_teachers.join(", ")
                })
            ),
            Line::from(
                format!(
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
                )
            )
        ];
        if overlaps > 1 {
            lines.push(
                Line::from(
                    format!(
                        "Overlap: {}/{}",
                        state.main.timetable.selected_lesson_idx + 1,
                        overlaps
                    )
                )
            );
        }
        if !lesson.lesson_text.is_empty() {
            lines.push(Line::from(format!("Lesson text: {}", lesson.lesson_text)));
        }
        if !lesson.remarks.is_empty() {
            lines.push(Line::from(Span::styled(lesson.remarks, Style::default().fg(WARNING))));
        }
        return lines;
    }
    vec![Line::from("Select a lesson to see details.")]
}

#[derive(Clone, Copy)]
struct AbsenceStatusMeta {
    short_label: &'static str,
    chip_label: &'static str,
    long_label: &'static str,
    chip_bg: Color,
    chip_fg: Color,
}

fn absence_status_meta(is_excused: bool) -> AbsenceStatusMeta {
    if is_excused {
        AbsenceStatusMeta {
            short_label: "EX",
            chip_label: "EXCUSED",
            long_label: "Excused",
            chip_bg: EXCUSED_BG,
            chip_fg: Color::Indexed(15),
        }
    } else {
        AbsenceStatusMeta {
            short_label: "UN",
            chip_label: "UNEXCUSED",
            long_label: "Unexcused",
            chip_bg: UNEXCUSED_BG,
            chip_fg: Color::Indexed(15),
        }
    }
}

fn render_absence_history_pane(
    frame: &mut Frame,
    state: &AppState,
    area: Rect,
    filtered: &[crate::models::ParsedAbsence],
    selected_count: usize,
    has_active_filters: bool,
    has_loaded_absences: bool
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_GRAY))
        .title("History")
        .title(
            Line::from(format!("{selected_count}/{}", filtered.len())).alignment(Alignment::Right)
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let row_content_width = usize::from(inner.width).max(30);
    let status_chip_compact = row_content_width < 56;
    let status_chip_inner_width = if status_chip_compact { 2 } else { 9 };
    let status_chip_width = status_chip_inner_width + 2;
    let date_col_width = 12usize;
    let mut note_col_width = row_content_width
        .saturating_sub(date_col_width)
        .saturating_sub(status_chip_width)
        .saturating_sub(4);
    if note_col_width < 10 {
        note_col_width = 10;
    }
    let list_rows = usize::from(inner.height).saturating_sub(2).max(3);
    let (visible_start, visible) = state.visible_absences(list_rows);

    let mut lines = vec![
        Line::from(
            vec![
                styled_cell("  ", Some(HEADER_BG), Some(DIM_GRAY)),
                styled_cell(&fit_text("When", date_col_width), Some(HEADER_BG), Some(DIM_GRAY)),
                styled_cell(" ", Some(HEADER_BG), Some(DIM_GRAY)),
                styled_cell(&fit_text("Notes", note_col_width), Some(HEADER_BG), Some(DIM_GRAY)),
                styled_cell(" ", Some(HEADER_BG), Some(DIM_GRAY)),
                styled_cell(&fit_text("State", status_chip_width), Some(HEADER_BG), Some(DIM_GRAY))
            ]
        )
    ];

    if state.main.absences.loading_initial {
        lines.extend(
            centered_message_lines(
                "Loading absences...",
                inner.height.saturating_sub(2),
                inner.width,
                Style::default().fg(WARNING)
            )
        );
    } else if !state.main.absences.error.is_empty() && !has_loaded_absences {
        lines.extend(
            centered_message_lines(
                &state.main.absences.error,
                inner.height.saturating_sub(2),
                inner.width,
                Style::default().fg(ERROR)
            )
        );
    } else if filtered.is_empty() && !has_loaded_absences {
        lines.extend(
            centered_message_lines(
                "No absences found in loaded history.",
                inner.height.saturating_sub(2),
                inner.width,
                Style::default().fg(WARNING)
            )
        );
    } else if filtered.is_empty() {
        lines.extend(
            centered_message_lines(
                if has_active_filters {
                    "No absences match current filters."
                } else {
                    "No absences found in loaded history."
                },
                inner.height.saturating_sub(2),
                inner.width,
                Style::default().fg(WARNING)
            )
        );
    } else {
        for (offset, absence) in visible.iter().enumerate() {
            let actual_index = visible_start + offset;
            let is_selected = actual_index == state.main.absences.selected_idx;
            let row_bg = if is_selected {
                Some(SELECT_BG)
            } else if actual_index % 2 == 1 {
                Some(ALT_BG)
            } else {
                None
            };
            let status = absence_status_meta(absence.is_excused);
            let chip_label = if status_chip_compact {
                status.short_label
            } else {
                status.chip_label
            };
            let note = truncate_text(
                &to_single_line(
                    if absence.text.is_empty() {
                        if absence.reason.is_empty() { "No reason" } else { &absence.reason }
                    } else {
                        &absence.text
                    }
                ),
                note_col_width
            );
            lines.push(
                Line::from(
                    vec![
                        styled_cell(
                            if is_selected {
                                "> "
                            } else {
                                "  "
                            },
                            row_bg,
                            Some(if is_selected { BRAND } else { BORDER_GRAY })
                        ),
                        styled_cell(
                            &fit_text(&format_absence_range_compact(absence), date_col_width),
                            row_bg,
                            Some(if is_selected { Color::Indexed(15) } else { DIM_GRAY })
                        ),
                        styled_cell(" ", row_bg, None),
                        styled_cell(
                            &fit_text(&note, note_col_width),
                            row_bg,
                            Some(Color::Indexed(15))
                        ),
                        styled_cell(" ", row_bg, None),
                        Span::styled(
                            format!(
                                " {:>width$} ",
                                fit_text(chip_label, status_chip_inner_width).trim(),
                                width = status_chip_inner_width
                            ),
                            Style::default()
                                .bg(status.chip_bg)
                                .fg(status.chip_fg)
                                .add_modifier(Modifier::BOLD)
                        )
                    ]
                )
            );
        }

        if state.main.absences.loading_more {
            lines.push(
                Line::from(
                    Span::styled(
                        fit_text("Loading older records...", usize::from(inner.width)),
                        Style::default().fg(WARNING)
                    )
                )
            );
        } else if state.main.absences.has_more {
            lines.push(
                Line::from(
                    Span::styled(
                        fit_text(
                            "More records available - press m or keep scrolling",
                            usize::from(inner.width)
                        ),
                        Style::default().fg(DIM_GRAY)
                    )
                )
            );
        } else if !filtered.is_empty() {
            lines.push(
                Line::from(
                    Span::styled(
                        fit_text("End of available history", usize::from(inner.width)),
                        Style::default().fg(DIM_GRAY)
                    )
                )
            );
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_absence_summary_pane(
    frame: &mut Frame,
    area: Rect,
    excused_count: usize,
    unexcused_count: usize,
    window_label: &str,
    newest_loaded: &str,
    oldest_loaded: &str
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_GRAY));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let lines = vec![
        line_with_right(
            "Summary",
            window_label,
            usize::from(inner.width),
            Style::default().fg(Color::Indexed(15)).add_modifier(Modifier::BOLD),
            Style::default().fg(DIM_GRAY)
        ),
        Line::from(format!("{excused_count} excused | {unexcused_count} unexcused")),
        Line::from(
            Span::styled(
                format!("Loaded range: {newest_loaded} -> {oldest_loaded}"),
                Style::default().fg(DIM_GRAY)
            )
        )
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_absence_details_pane(
    frame: &mut Frame,
    area: Rect,
    selected: Option<crate::models::ParsedAbsence>
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_GRAY));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let header = if let Some(absence) = &selected {
        let status = absence_status_meta(absence.is_excused);
        let chip = format!(" {} ", status.chip_label);
        let width = usize::from(inner.width);
        let gap = width.saturating_sub("Details".len() + chip.len());
        Line::from(
            vec![
                Span::styled("Details", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" ".repeat(gap)),
                Span::styled(chip, Style::default().fg(status.chip_fg).bg(status.chip_bg))
            ]
        )
    } else {
        line_with_right(
            "Details",
            "No selection",
            usize::from(inner.width),
            Style::default().fg(Color::Indexed(15)).add_modifier(Modifier::BOLD),
            Style::default().fg(DIM_GRAY)
        )
    };

    let mut lines = vec![header];
    if let Some(absence) = selected {
        lines.extend([
            Line::from(Span::styled("When", Style::default().fg(DIM_GRAY))),
            Line::from(format_absence_range_full(&absence)),
            Line::from(Span::styled("Reason", Style::default().fg(DIM_GRAY))),
            Line::from(
                to_single_line(
                    if absence.reason.is_empty() {
                        "No reason"
                    } else {
                        &absence.reason
                    }
                )
            ),
            Line::from(Span::styled("Excuse status", Style::default().fg(DIM_GRAY))),
            Line::from(
                to_single_line(
                    if absence.excuse_status.is_empty() {
                        absence_status_meta(absence.is_excused).long_label
                    } else {
                        &absence.excuse_status
                    }
                )
            ),
            Line::from(Span::styled("Notes", Style::default().fg(DIM_GRAY))),
            Line::from(
                to_single_line(
                    if absence.text.is_empty() {
                        "No additional notes"
                    } else {
                        &absence.text
                    }
                )
            ),
        ]);
    } else {
        lines.push(
            Line::from(
                Span::styled(
                    "Select a record from the history list.",
                    Style::default().fg(DIM_GRAY)
                )
            )
        );
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn line_with_right(
    left: &str,
    right: &str,
    width: usize,
    left_style: Style,
    right_style: Style
) -> Line<'static> {
    if right.is_empty() {
        return Line::from(Span::styled(truncate_text(left, width), left_style));
    }
    let right_width = UnicodeWidthStr::width(right);
    let left_width = UnicodeWidthStr::width(left);
    if width <= right_width + 1 {
        return Line::from(Span::styled(truncate_text(right, width), right_style));
    }
    let max_left_width = width.saturating_sub(right_width + 1);
    let left_text = if left_width > max_left_width {
        truncate_text(left, max_left_width)
    } else {
        left.to_owned()
    };
    let gap = width.saturating_sub(UnicodeWidthStr::width(left_text.as_str()) + right_width);
    Line::from(
        vec![
            Span::styled(left_text, left_style),
            Span::raw(" ".repeat(gap)),
            Span::styled(right.to_owned(), right_style)
        ]
    )
}

fn filter_chip(label: &str, active: bool) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(if active { Color::Indexed(15) } else { DIM_GRAY })
            .bg(if active { SELECT_BG } else { Color::Reset })
    )
}

fn truncate_text(value: &str, width: usize) -> String {
    fit_text(value, width).trim_end().to_owned()
}

fn to_single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn format_absence_range_compact(absence: &crate::models::ParsedAbsence) -> String {
    let start = format!("{:02}.{:02}", absence.start_date.day(), absence.start_date.month());
    let end = format!("{:02}.{:02}", absence.end_date.day(), absence.end_date.month());
    if absence.start_date == absence.end_date {
        start
    } else {
        format!("{start}->{end}")
    }
}

fn format_absence_range_full(absence: &crate::models::ParsedAbsence) -> String {
    if absence.start_date == absence.end_date {
        format!(
            "{} {}-{}",
            crate::models::format_date(absence.start_date),
            absence.start_time,
            absence.end_time
        )
    } else {
        format!(
            "{} {} -> {} {}",
            crate::models::format_date(absence.start_date),
            absence.start_time,
            crate::models::format_date(absence.end_date),
            absence.end_time
        )
    }
}

fn absence_list_summary(state: &AppState, filtered: &[crate::models::ParsedAbsence]) -> String {
    if filtered.len() == state.main.absences.absences.len() {
        format!("Showing {}", state.main.absences.absences.len())
    } else {
        format!("Showing {} of {}", filtered.len(), state.main.absences.absences.len())
    }
}

fn absence_history_load_label(state: &AppState) -> String {
    if state.main.absences.loading_initial {
        "Initial sync in progress".to_owned()
    } else if state.main.absences.loading_more {
        "Loading older records".to_owned()
    } else if state.main.absences.has_more {
        "Ready to load more".to_owned()
    } else {
        "History fully loaded".to_owned()
    }
}

fn absence_prefetch_hint(state: &AppState, filtered_len: usize) -> String {
    if state.main.absences.loading_more {
        "Loading older records now...".to_owned()
    } else if state.main.absences.has_more {
        let page_jump = usize::from(state.terminal_height.saturating_sub(11)).max(4) / 2;
        let prefetch_threshold = page_jump.max(6);
        if !state.has_active_absence_filters() {
            format!(
                "Auto-load keeps ~{} rows buffered. Press m to fetch now.",
                page_jump + prefetch_threshold
            )
        } else if filtered_len <= state.main.absences.selected_idx + prefetch_threshold {
            format!("Auto-load starts near the bottom ({} rows early). Press m to fetch now.", prefetch_threshold)
        } else {
            format!("Auto-load starts near the bottom ({} rows early). Press m to fetch now.", prefetch_threshold)
        }
    } else {
        "Reached oldest available records in loaded history.".to_owned()
    }
}

fn centered_message_lines(
    message: &str,
    height: u16,
    width: u16,
    style: Style
) -> Vec<Line<'static>> {
    if height == 0 || width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let top_padding = usize::from(height.saturating_sub(1) / 2);
    for _ in 0..top_padding {
        lines.push(Line::from(""));
    }
    let message_width = UnicodeWidthStr::width(message);
    let left_pad = usize::from(width).saturating_sub(message_width) / 2;
    lines.push(
        Line::from(
            Span::styled(
                format!("{}{}", " ".repeat(left_pad), truncate_text(message, usize::from(width))),
                style
            )
        )
    );
    lines
}

fn login_field_line(
    label: &str,
    value: &str,
    placeholder: &str,
    focused: bool,
    mask: bool
) -> Line<'static> {
    let rendered = if value.is_empty() {
        placeholder.to_owned()
    } else if mask {
        "*".repeat(value.chars().count())
    } else {
        value.to_owned()
    };
    Line::from(
        vec![
            Span::styled(
                format!("{}{}: ", if focused { "> " } else { "  " }, label),
                Style::default()
                    .fg(if focused { BRAND } else { Color::Gray })
                    .add_modifier(if focused { Modifier::BOLD } else { Modifier::empty() })
            ),
            Span::raw(rendered)
        ]
    )
}

fn tab_span(label: &str, active: bool) -> Span<'static> {
    Span::styled(
        label.to_owned(),
        Style::default()
            .fg(if active { Color::Black } else { Color::White })
            .bg(if active { BRAND } else { Color::DarkGray })
            .add_modifier(if active { Modifier::BOLD } else { Modifier::empty() })
    )
}

fn styled_cell(text: &str, bg: Option<Color>, fg: Option<Color>) -> Span<'static> {
    Span::styled(
        text.to_owned(),
        Style::default().bg(bg.unwrap_or(Color::Reset)).fg(fg.unwrap_or(Color::Reset))
    )
}

fn render_input_text(value: &str, cursor: usize, mask: bool) -> String {
    let value = if mask { "*".repeat(value.chars().count()) } else { value.to_owned() };
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
        if
            UnicodeWidthStr::width(result.as_str()) +
                UnicodeWidthStr::width(character.encode_utf8(&mut [0; 4])) > width.saturating_sub(1)
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
    use crate::models::{ Config, ParsedAbsence, SavedConfig };
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

    fn sample_absence() -> ParsedAbsence {
        ParsedAbsence {
            id: 1,
            student_name: "User".into(),
            reason: "Ill".into(),
            text: "Doctor".into(),
            excuse_status: "Excused".into(),
            is_excused: true,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            start_time: "08:00".into(),
            end_time: "08:50".into(),
        }
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
        state.main.absences.absences.push(sample_absence());
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
        state.main.absences.absences.push(sample_absence());
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("History"));
        assert!(output.contains("1/1"));
        assert!(output.contains("Summary"));
        assert!(output.contains("Details"));
        assert!(output.contains("When"));
        assert!(output.contains("Reason"));
        assert!(output.contains("Excuse status"));
        assert!(output.contains("Mar 26, 2026 08:00-08:50"));
    }

    #[test]
    fn render_absences_wide_layout_shows_summary_and_profile_header() {
        let backend = TestBackend::new(140, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.config = Some(Config {
            school: "school".into(),
            username: "user".into(),
            password: "secret".into(),
            server: "mese.webuntis.com".into(),
        });
        state.main.absences.absences.push(sample_absence());
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("Absence Timeline"));
        assert!(output.contains("user@school"));
        assert!(output.contains("Newest first | Showing 1"));
        assert!(output.contains("Summary"));
        assert!(output.contains("Loaded range: Mar 26, 2026 -> Mar 26, 2026"));
    }

    #[test]
    fn render_absences_narrow_layout_stacks_and_uses_compact_filter_header() {
        let backend = TestBackend::new(90, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::new();
        state.screen = Screen::MainShell;
        state.main.active_tab = TabId::Absences;
        state.main.absences.absences.push(sample_absence());
        terminal.draw(|frame| render(frame, &state)).unwrap();
        let output = buffer_text(terminal.backend().buffer());
        assert!(output.contains("[f:All] [w:All time] [/:none] [c]"));
        assert!(output.contains("26.03"));
        assert!(output.contains("Summary"));
        assert!(output.contains("Details"));
    }
}

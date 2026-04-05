use super::shared::{centered_rect, fit_text, render_input_text, styled_cell};
use super::theme::{BRAND, SELECT_BG, WARNING};
use crate::app::state::AppState;
use crate::webuntis::format_timetable_search_type_label;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub(super) fn render_timetable(frame: &mut Frame, state: &AppState, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(6), Constraint::Length(6)])
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

    frame.render_widget(
        Paragraph::new(build_timetable_details(state))
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false }),
        layout[2],
    );
}

pub(super) fn render_timetable_search_popup(frame: &mut Frame, state: &AppState, area: Rect) {
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
                false,
            )
        )),
        Line::from(if state.main.timetable.search_index_loading {
            "Loading timetable targets...".to_owned()
        } else if !state.main.timetable.search_index_error.is_empty() {
            format!("Target load failed: {}", state.main.timetable.search_index_error)
        } else {
            "Use ↑/↓ and Enter apply, Esc cancel.".to_owned()
        }),
        Line::from(""),
    ];

    for (index, result) in state.timetable_search_results().into_iter().take(12).enumerate() {
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
    let day_width = ((state.terminal_width as usize).saturating_sub(time_width + 6) / 5).max(8);
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
            fit_text(&format!("{} {}", period.name, period.start_time), time_width),
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

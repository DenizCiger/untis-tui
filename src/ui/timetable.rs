use super::shared::{
    centered_message_lines,
    centered_rect,
    fit_text,
    line_with_right,
    render_input_text,
};
use super::theme::{
    ALT_BG,
    BLACK,
    BRAND,
    BRIGHT_WHITE,
    DIM_GRAY,
    ERROR,
    INFO,
    LESSON_CANCELLED_BG,
    LESSON_CANCELLED_FOCUS_BG,
    LESSON_DEFAULT_BG,
    LESSON_DEFAULT_FOCUS_BG,
    LESSON_EXAM_BG,
    LESSON_EXAM_FOCUS_BG,
    LESSON_SUBSTITUTION_BG,
    LESSON_SUBSTITUTION_FOCUS_BG,
    NEUTRAL_LIGHT,
    NEUTRAL_MID,
    SUBJECT_STRIPE_COLORS,
    WARNING,
};
use crate::app::state::AppState;
use crate::models::{ ParsedLesson, TimeUnit };
use crate::timetable_model::{
    Continuation,
    GRID_ROW_HEIGHT,
    MIN_DETAILS_HEIGHT,
    SPLIT_DAY_COLUMN_MIN_WIDTH,
    TimetableRenderModel,
    build_render_model,
    day_column_width,
    find_current_period_index,
    is_compact,
    lessons_for_period,
    selected_lesson_position,
    time_column_width,
    timetable_rows_per_page,
};
use crate::webuntis::format_timetable_search_type_label;
use ratatui::Frame;
use ratatui::layout::{ Constraint, Direction, Layout, Rect };
use ratatui::style::{ Color, Modifier, Style };
use ratatui::text::{ Line, Span };
use ratatui::widgets::{ Block, Borders, Clear, Paragraph, Wrap };
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

const TITLE_ROWS: u16 = 2;
const DAY_COUNT: usize = 5;

pub(super) fn render_timetable(frame: &mut Frame, state: &AppState, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(TITLE_ROWS), Constraint::Min(0)])
        .split(area);

    frame.render_widget(
        Paragraph::new(build_timetable_title_lines(state, layout[0].width)),
        layout[0]
    );

    let details_lines = build_timetable_details(state);
    let body_area = layout[1];
    let details_min_height = MIN_DETAILS_HEIGHT.min(body_area.height);

    if let Some(data) = state.main.timetable.data.as_ref() {
        let model = build_render_model(data, 2);
        let compact = is_compact(area.width, area.height);
        let time_width = time_column_width(area.width, area.height);
        let day_width = day_column_width(area.width, area.height);
        let rows_per_page = timetable_rows_per_page(area.height).max(1);
        let scroll_offset = state.main.timetable.scroll_offset.min(
            data.timegrid.len().saturating_sub(rows_per_page)
        );
        let visible_periods = data.timegrid
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(rows_per_page)
            .collect::<Vec<_>>();
        let grid_lines = build_timetable_grid_lines(
            state,
            data,
            &model,
            compact,
            time_width,
            day_width,
            scroll_offset,
            &visible_periods
        );
        let grid_height = (grid_lines.len() as u16).min(
            body_area.height.saturating_sub(details_min_height)
        );
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(grid_height), Constraint::Min(details_min_height)])
            .split(body_area);
        frame.render_widget(
            Paragraph::new(grid_lines).wrap(Wrap { trim: false }),
            content_layout[0]
        );
        frame.render_widget(
            Paragraph::new(details_lines)
                .block(Block::default().borders(Borders::ALL).title("Details"))
                .wrap(Wrap { trim: false }),
            content_layout[1]
        );
        return;
    }

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(details_min_height)])
        .split(body_area);
    let empty_message = if state.main.timetable.loading {
        "Loading timetable..."
    } else if state.main.timetable.error.is_empty() {
        "No timetable data loaded."
    } else {
        &state.main.timetable.error
    };
    frame.render_widget(
        Paragraph::new(
            centered_message_lines(
                empty_message,
                content_layout[0].height,
                content_layout[0].width,
                Style::default().fg(
                    if state.main.timetable.error.is_empty() {
                        WARNING
                    } else {
                        ERROR
                    }
                )
            )
        ),
        content_layout[0]
    );
    frame.render_widget(
        Paragraph::new(details_lines)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false }),
        content_layout[1]
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

fn build_timetable_title_lines(state: &AppState, width: u16) -> Vec<Line<'static>> {
    let (monday, friday) = crate::models::current_week_range(state.main.timetable.week_offset);
    let date_range = if monday != friday {
        format!("{} - {}", crate::models::format_date(monday), crate::models::format_date(friday))
    } else {
        crate::models::format_date(monday)
    };
    let cached_marker = if state.main.timetable.is_from_cache { " (cached)" } else { "" };
    let username = state.config
        .as_ref()
        .map(|config| format!("{}@{}", config.username, config.school))
        .unwrap_or_default();
    let title = "WebUntis TUI";
    let header_line = if username.is_empty() {
        Line::from(
            vec![
                Span::styled(title, Style::default().fg(BRAND).add_modifier(Modifier::BOLD)),
                Span::styled(cached_marker, Style::default().fg(Color::Indexed(3)))
            ]
        )
    } else {
        let right_width = UnicodeWidthStr::width(username.as_str());
        if usize::from(width) <= right_width + 1 {
            line_with_right(
                "",
                &username,
                usize::from(width),
                Style::default(),
                Style::default().fg(DIM_GRAY)
            )
        } else {
            let left_width = UnicodeWidthStr::width(title) + UnicodeWidthStr::width(cached_marker);
            let gap = usize::from(width).saturating_sub(left_width + right_width);
            Line::from(
                vec![
                    Span::styled(title, Style::default().fg(BRAND).add_modifier(Modifier::BOLD)),
                    Span::styled(cached_marker, Style::default().fg(Color::Indexed(3))),
                    Span::raw(" ".repeat(gap.max(1))),
                    Span::styled(username, Style::default().fg(DIM_GRAY))
                ]
            )
        }
    };
    let centered_date = format!("‹ {} ›", date_range);
    let left_pad =
        usize::from(width).saturating_sub(UnicodeWidthStr::width(centered_date.as_str())) / 2;
    vec![
        header_line,
        Line::from(
            Span::styled(
                format!("{}{}", " ".repeat(left_pad), centered_date),
                Style::default().fg(DIM_GRAY)
            )
        )
    ]
}

fn build_timetable_grid_lines(
    state: &AppState,
    data: &crate::models::WeekTimetable,
    model: &TimetableRenderModel,
    compact: bool,
    time_width: u16,
    day_width: u16,
    scroll_offset: usize,
    visible_periods: &[(usize, &TimeUnit)]
) -> Vec<Line<'static>> {
    let mut color_map = HashMap::<String, Color>::new();
    let mut lines = Vec::new();
    let today = crate::models::today_local();
    let today_idx = data.days.iter().position(|day| day.date == today);
    let current_period_idx = find_current_period_index(&data.timegrid);

    lines.push(build_day_header_line(data, compact, time_width, day_width, today_idx));
    lines.push(
        Line::from(
            Span::styled(
                build_grid_divider(time_width, day_width, DAY_COUNT, "┼"),
                Style::default().fg(DIM_GRAY)
            )
        )
    );

    if scroll_offset > 0 {
        lines.push(
            build_scroll_hint_line(time_width, day_width, &format!("▲ {scroll_offset} more ▲"))
        );
    }

    for (period_idx, period) in visible_periods {
        let row_lines = build_period_row_lines(
            state,
            data,
            model,
            period,
            *period_idx,
            compact,
            time_width,
            day_width,
            current_period_idx,
            &mut color_map
        );
        lines.extend(row_lines);
    }

    let hidden_below = data.timegrid.len().saturating_sub(scroll_offset + visible_periods.len());
    if hidden_below > 0 {
        lines.push(
            build_scroll_hint_line(time_width, day_width, &format!("▼ {hidden_below} more ▼"))
        );
    }

    lines
}

fn build_day_header_line(
    data: &crate::models::WeekTimetable,
    compact: bool,
    time_width: u16,
    day_width: u16,
    today_idx: Option<usize>
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            pad_with_margin("Time", time_width),
            Style::default().fg(DIM_GRAY).add_modifier(Modifier::BOLD)
        )
    ];

    for (index, day) in data.days.iter().take(DAY_COUNT).enumerate() {
        spans.push(Span::styled("│", Style::default().fg(DIM_GRAY)));
        spans.push(
            Span::styled(
                center_text(
                    if compact {
                        &day.day_name[..day.day_name.len().min(2)]
                    } else {
                        &day.day_name[..day.day_name.len().min(3)]
                    },
                    day_width.saturating_sub(1)
                ),
                Style::default()
                    .fg(if Some(index) == today_idx { BRAND } else { BRIGHT_WHITE })
                    .add_modifier(Modifier::BOLD)
            )
        );
    }

    Line::from(spans)
}

fn build_scroll_hint_line(time_width: u16, day_width: u16, label: &str) -> Line<'static> {
    let mut spans = vec![Span::raw(" ".repeat(usize::from(time_width)))];
    for day_idx in 0..DAY_COUNT {
        spans.push(Span::styled("│", Style::default().fg(DIM_GRAY)));
        spans.push(
            Span::styled(
                if day_idx == 2 {
                    center_text(label, day_width.saturating_sub(1))
                } else {
                    " ".repeat(usize::from(day_width.saturating_sub(1)))
                },
                Style::default().fg(DIM_GRAY)
            )
        );
    }
    Line::from(spans)
}

fn build_period_row_lines(
    state: &AppState,
    data: &crate::models::WeekTimetable,
    model: &TimetableRenderModel,
    period: &TimeUnit,
    period_idx: usize,
    compact: bool,
    time_width: u16,
    day_width: u16,
    current_period_idx: Option<usize>,
    color_map: &mut HashMap<String, Color>
) -> Vec<Line<'static>> {
    let mut row_spans = vec![Vec::<Span>::new(), Vec::<Span>::new(), Vec::<Span>::new()];
    let time_lines = build_time_column_lines(
        period,
        compact,
        time_width,
        period_idx == state.main.timetable.selected_period_idx,
        current_period_idx == Some(period_idx)
    );
    for line_idx in 0..GRID_ROW_HEIGHT as usize {
        row_spans[line_idx].extend(time_lines[line_idx].clone());
    }

    let cell_width = day_width.saturating_sub(1);
    let can_render_split = day_width >= SPLIT_DAY_COLUMN_MIN_WIDTH;

    for day_idx in 0..DAY_COUNT {
        let lessons = lessons_for_period(model, &data.timegrid, day_idx, period_idx);
        let overlay = model.overlay_index_by_day
            .get(day_idx)
            .and_then(|day_overlay| day_overlay.get(&period.start_time));
        let is_anchor_focused =
            day_idx == state.main.timetable.selected_day_idx &&
            period_idx == state.main.timetable.selected_period_idx;
        let selected_entry = if is_anchor_focused {
            lessons.get(state.main.timetable.selected_lesson_idx)
        } else {
            None
        };
        let content_lines = if lessons.is_empty() {
            build_empty_cell_lines(is_anchor_focused, cell_width)
        } else if can_render_split && overlay.map(|value| value.split).unwrap_or(false) {
            build_split_cell_lines(overlay.unwrap(), selected_entry, cell_width, color_map)
        } else if lessons.len() > 1 {
            let label = if
                let Some(entry) = (if is_anchor_focused {
                    selected_entry.or_else(|| lessons.first())
                } else {
                    lessons.first()
                })
            {
                format!("{} +{}", entry.lesson.subject, lessons.len().saturating_sub(1))
            } else {
                format!("{}x", lessons.len())
            };
            build_overlap_preview_lines(label.as_str(), is_anchor_focused, cell_width)
        } else {
            build_lesson_lane_lines(
                lessons.first().unwrap(),
                subject_stripe_color(&lessons[0].lesson.subject, color_map),
                is_anchor_focused,
                cell_width,
                None
            )
        };

        for line_idx in 0..GRID_ROW_HEIGHT as usize {
            row_spans[line_idx].push(Span::styled("│", Style::default().fg(DIM_GRAY)));
            row_spans[line_idx].extend(content_lines[line_idx].clone());
        }
    }

    row_spans.into_iter().map(Line::from).collect()
}

fn build_time_column_lines(
    period: &TimeUnit,
    compact: bool,
    time_width: u16,
    is_focused: bool,
    is_current: bool
) -> [Vec<Span<'static>>; 3] {
    let label_width = usize::from(time_width.saturating_sub(2));
    let period_label = truncate_text(&period.name, if compact { 8 } else { 12 });
    let time_label = format!(
        "{} - {}",
        truncate_text(&period.start_time, 5),
        truncate_text(&period.end_time, 5)
    );

    [
        vec![
            Span::styled(
                pad_with_margin(&period_label, time_width),
                Style::default()
                    .fg(if is_current { BRAND } else { WARNING })
                    .add_modifier(Modifier::BOLD)
            )
        ],
        vec![
            Span::styled(
                format!(" {} ", fit_text(&time_label, label_width)),
                Style::default()
                    .bg(if is_focused { ALT_BG } else { Color::Reset })
                    .fg(if is_focused { BRIGHT_WHITE } else { Color::Reset })
            )
        ],
        vec![Span::raw(" ".repeat(usize::from(time_width)))],
    ]
}

fn build_empty_cell_lines(is_focused: bool, width: u16) -> [Vec<Span<'static>>; 3] {
    let content_width = usize::from(width);
    if is_focused {
        return [
            vec![Span::styled(" ".repeat(content_width), Style::default().bg(ALT_BG).fg(BLACK))],
            vec![Span::styled(" ".repeat(content_width), Style::default().bg(ALT_BG).fg(BLACK))],
            vec![Span::raw(" ".repeat(content_width))],
        ];
    }

    [
        vec![Span::raw(" ".repeat(content_width))],
        vec![Span::styled(center_text(".", width), Style::default().fg(DIM_GRAY))],
        vec![Span::raw(" ".repeat(content_width))],
    ]
}

fn build_overlap_preview_lines(
    label: &str,
    is_focused: bool,
    width: u16
) -> [Vec<Span<'static>>; 3] {
    [
        vec![Span::raw(" ".repeat(usize::from(width)))],
        vec![
            Span::styled(
                center_text(label, width),
                Style::default().fg(if is_focused { WARNING } else { BRIGHT_WHITE })
            )
        ],
        vec![Span::raw(" ".repeat(usize::from(width)))],
    ]
}

fn build_split_cell_lines(
    overlay: &crate::timetable_model::OverlayPeriod,
    selected_entry: Option<&crate::timetable_model::RenderLesson>,
    width: u16,
    color_map: &mut HashMap<String, Color>
) -> [Vec<Span<'static>>; 3] {
    let split_gap_width = 1u16;
    let left_lane_width = width.saturating_sub(split_gap_width) / 2;
    let right_lane_width = width.saturating_sub(split_gap_width + left_lane_width);

    let left_entry = overlay.lanes.first().and_then(Option::as_ref);
    let right_entry = overlay.lanes.get(1).and_then(Option::as_ref);
    let left_focused = left_entry
        .zip(selected_entry)
        .map(|(left, selected)| left.lesson_instance_id == selected.lesson_instance_id)
        .unwrap_or(false);
    let right_focused = right_entry
        .zip(selected_entry)
        .map(|(right, selected)| right.lesson_instance_id == selected.lesson_instance_id)
        .unwrap_or(false);
    let left_suffix = if overlay.hidden_count > 0 {
        Some(format!("+{}", overlay.hidden_count))
    } else {
        None
    };

    let left_lines = if let Some(entry) = left_entry {
        build_lesson_lane_lines(
            entry,
            subject_stripe_color(&entry.lesson.subject, color_map),
            left_focused,
            left_lane_width,
            left_suffix.as_deref()
        )
    } else {
        build_blank_lane_lines(left_lane_width)
    };
    let right_lines = if let Some(entry) = right_entry {
        build_lesson_lane_lines(
            entry,
            subject_stripe_color(&entry.lesson.subject, color_map),
            right_focused,
            right_lane_width,
            if left_entry.is_none() {
                left_suffix.as_deref()
            } else {
                None
            }
        )
    } else {
        build_blank_lane_lines(right_lane_width)
    };

    let gap = vec![Span::raw(" ".repeat(usize::from(split_gap_width)))];
    let mut lines = [Vec::new(), Vec::new(), Vec::new()];
    for line_idx in 0..GRID_ROW_HEIGHT as usize {
        lines[line_idx].extend(left_lines[line_idx].clone());
        lines[line_idx].extend(gap.clone());
        lines[line_idx].extend(right_lines[line_idx].clone());
    }
    lines
}

fn build_blank_lane_lines(width: u16) -> [Vec<Span<'static>>; 3] {
    [
        vec![Span::raw(" ".repeat(usize::from(width)))],
        vec![Span::raw(" ".repeat(usize::from(width)))],
        vec![Span::raw(" ".repeat(usize::from(width)))],
    ]
}

fn build_lesson_lane_lines(
    entry: &crate::timetable_model::RenderLesson,
    stripe_color: Color,
    is_focused: bool,
    width: u16,
    title_suffix: Option<&str>
) -> [Vec<Span<'static>>; 3] {
    let starts_here = matches!(entry.continuation, Continuation::Single | Continuation::Start);
    let continues_down = matches!(entry.continuation, Continuation::Start | Continuation::Middle);
    let lesson = &entry.lesson;
    let title = if starts_here {
        match title_suffix {
            Some(suffix) => format!("{} {}", lesson.subject, suffix),
            None => lesson.subject.clone(),
        }
    } else {
        String::new()
    };
    let meta = if starts_here {
        format!(
            "{}{}",
            if lesson.room.is_empty() {
                "?"
            } else {
                &lesson.room
            },
            if lesson.teacher.is_empty() {
                String::new()
            } else {
                format!(" {}", lesson.teacher)
            }
        )
    } else {
        String::new()
    };

    let colors = lesson_colors(lesson, is_focused);
    let content_width = width.saturating_sub(1);
    let continuation_bg = colors.base_bg;

    [
        styled_stripe_line(
            "▍",
            &title,
            stripe_color,
            colors.main_bg,
            colors.main_fg,
            content_width,
            starts_here,
            lesson.cancelled && starts_here
        ),
        styled_stripe_line(
            "▍",
            &meta,
            stripe_color,
            colors.main_bg,
            colors.subtext_fg,
            content_width,
            false,
            false
        ),
        if continues_down {
            styled_stripe_line(
                "▍",
                "",
                stripe_color,
                continuation_bg,
                colors.continuation_fg,
                content_width,
                false,
                false
            )
        } else {
            [
                Span::styled(" ", Style::default().fg(stripe_color)),
                Span::raw(" ".repeat(usize::from(content_width))),
            ]
                .into_iter()
                .collect()
        },
    ]
}

fn styled_stripe_line(
    stripe: &str,
    text: &str,
    stripe_color: Color,
    background: Color,
    foreground: Color,
    content_width: u16,
    bold: bool,
    strikethrough: bool
) -> Vec<Span<'static>> {
    let mut text_style = Style::default().bg(background).fg(foreground);
    if bold {
        text_style = text_style.add_modifier(Modifier::BOLD);
    }
    if strikethrough {
        text_style = text_style.add_modifier(Modifier::CROSSED_OUT);
    }

    vec![
        Span::styled(stripe.to_owned(), Style::default().bg(background).fg(stripe_color)),
        Span::styled(fit_text(text, usize::from(content_width)), text_style)
    ]
}

fn build_timetable_details(state: &AppState) -> Vec<Line<'static>> {
    if let Some(lesson) = state.selected_timetable_lesson() {
        let overlaps = state.current_timetable_lessons().len();
        let overlap_position = state.main.timetable.data
            .as_ref()
            .map(|data| {
                let model = build_render_model(data, 2);
                selected_lesson_position(
                    &model,
                    data,
                    state.main.timetable.selected_day_idx,
                    state.main.timetable.selected_period_idx,
                    state.main.timetable.selected_lesson_idx
                )
            })
            .unwrap_or(state.main.timetable.selected_lesson_idx + 1);

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
            lines.push(Line::from(format!("Overlap: {overlap_position}/{overlaps}")));
        }
        if !lesson.lesson_text.is_empty() {
            lines.push(Line::from(format!("Lesson text: {}", lesson.lesson_text)));
        }
        if !lesson.remarks.is_empty() {
            lines.push(Line::from(Span::styled(lesson.remarks, Style::default().fg(INFO))));
        }
        return lines;
    }
    vec![Line::from("Select a lesson to see details.")]
}

fn build_grid_divider(time_width: u16, day_width: u16, day_count: usize, junction: &str) -> String {
    let mut line = "─".repeat(usize::from(time_width));
    for _ in 0..day_count {
        line.push_str(junction);
        line.push_str(&"─".repeat(usize::from(day_width.saturating_sub(1))));
    }
    line
}

fn center_text(value: &str, width: u16) -> String {
    let clipped = truncate_text(value, usize::from(width));
    let pad = usize::from(width).saturating_sub(UnicodeWidthStr::width(clipped.as_str()));
    let left = pad / 2;
    let right = pad.saturating_sub(left);
    format!("{}{}{}", " ".repeat(left), clipped, " ".repeat(right))
}

fn pad_with_margin(value: &str, width: u16) -> String {
    if width == 0 {
        return String::new();
    }
    let inner_width = usize::from(width.saturating_sub(2));
    format!(" {} ", fit_text(value, inner_width))
}

fn truncate_text(value: &str, width: usize) -> String {
    fit_text(value, width).trim_end().to_owned()
}

fn subject_stripe_color(subject: &str, color_map: &mut HashMap<String, Color>) -> Color {
    if let Some(color) = color_map.get(subject) {
        return *color;
    }
    let color = SUBJECT_STRIPE_COLORS[color_map.len() % SUBJECT_STRIPE_COLORS.len()];
    color_map.insert(subject.to_owned(), color);
    color
}

struct LessonColors {
    base_bg: Color,
    main_bg: Color,
    main_fg: Color,
    subtext_fg: Color,
    continuation_fg: Color,
}

fn lesson_colors(lesson: &ParsedLesson, is_focused: bool) -> LessonColors {
    let cell_state = lesson.cell_state.trim().to_uppercase();
    let is_substitution_like =
        lesson.substitution ||
        matches!(
            cell_state.as_str(),
            "SUBSTITUTION" | "ADDITIONAL" | "ROOMSUBSTITUTION" | "ROOMSUBSTITION"
        );
    let is_exam = cell_state == "EXAM";
    let is_cancelled = lesson.cancelled || cell_state == "CANCELLED";

    if is_cancelled {
        return LessonColors {
            base_bg: LESSON_CANCELLED_BG,
            main_bg: if is_focused {
                LESSON_CANCELLED_FOCUS_BG
            } else {
                LESSON_CANCELLED_BG
            },
            main_fg: BRIGHT_WHITE,
            subtext_fg: NEUTRAL_MID,
            continuation_fg: NEUTRAL_MID,
        };
    }
    if is_exam {
        return LessonColors {
            base_bg: LESSON_EXAM_BG,
            main_bg: if is_focused {
                LESSON_EXAM_FOCUS_BG
            } else {
                LESSON_EXAM_BG
            },
            main_fg: BLACK,
            subtext_fg: Color::Indexed(236),
            continuation_fg: Color::Indexed(236),
        };
    }
    if is_substitution_like {
        return LessonColors {
            base_bg: LESSON_SUBSTITUTION_BG,
            main_bg: if is_focused {
                LESSON_SUBSTITUTION_FOCUS_BG
            } else {
                LESSON_SUBSTITUTION_BG
            },
            main_fg: BRIGHT_WHITE,
            subtext_fg: NEUTRAL_MID,
            continuation_fg: NEUTRAL_MID,
        };
    }

    LessonColors {
        base_bg: LESSON_DEFAULT_BG,
        main_bg: if is_focused {
            LESSON_DEFAULT_FOCUS_BG
        } else {
            LESSON_DEFAULT_BG
        },
        main_fg: BRIGHT_WHITE,
        subtext_fg: NEUTRAL_LIGHT,
        continuation_fg: NEUTRAL_LIGHT,
    }
}

use super::shared::{
    centered_message_lines, filter_chip, fit_text, line_with_right, render_input_text, styled_cell,
    to_single_line, truncate_text,
};
use super::theme::{
    ALT_BG, BORDER_GRAY, BRAND, DIM_GRAY, ERROR, EXCUSED_BG, HEADER_BG, SELECT_BG, UNEXCUSED_BG,
    WARNING,
};
use crate::app::state::AppState;
use crate::timetable_model::SHELL_HEADER_HEIGHT;
use chrono::Datelike;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Clone, Copy)]
struct AbsenceStatusMeta {
    short_label: &'static str,
    chip_label: &'static str,
    long_label: &'static str,
    chip_bg: Color,
    chip_fg: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AbsenceLayoutGeometry {
    pub history_area: Rect,
    pub history_inner: Rect,
    pub visible_start: usize,
    pub visible_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AbsenceClickTarget {
    pub selected_idx: usize,
}

pub(super) fn render_absences(frame: &mut Frame, state: &AppState, area: Rect) {
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
    let filtered_excused = filtered.iter().filter(|absence| absence.is_excused).count();
    let filtered_unexcused = filtered.len().saturating_sub(filtered_excused);
    let has_active_filters = state.has_active_absence_filters();
    let has_loaded_absences = !state.main.absences.absences.is_empty();
    let list_summary = absence_list_summary(state, &filtered);
    let history_label = absence_history_load_label(state);
    let newest_loaded = state
        .main
        .absences
        .absences
        .first()
        .map(|absence| crate::models::format_date(absence.start_date))
        .unwrap_or_else(|| "-".to_owned());
    let oldest_loaded = state
        .main
        .absences
        .absences
        .last()
        .map(|absence| crate::models::format_date(absence.start_date))
        .unwrap_or_else(|| "-".to_owned());

    frame.render_widget(
        Paragraph::new(line_with_right(
            "Absence Timeline",
            &state
                .config
                .as_ref()
                .map(|config| format!("{}@{}", config.username, config.school))
                .unwrap_or_default(),
            usize::from(layout[0].width),
            Style::default().fg(BRAND).add_modifier(Modifier::BOLD),
            Style::default().fg(DIM_GRAY),
        )),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new(line_with_right(
            &format!("Newest first | {list_summary}"),
            &format!(
                "{} days loaded | {history_label}",
                state.main.absences.days_loaded
            ),
            usize::from(layout[1].width),
            Style::default().fg(DIM_GRAY),
            Style::default().fg(DIM_GRAY),
        )),
        layout[1],
    );

    let filter_line = if compact_header {
        Line::from(Span::styled(
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
            Style::default().fg(DIM_GRAY),
        ))
    } else {
        Line::from(vec![
            filter_chip(
                &format!("Status: {}", state.main.absences.status_filter.label()),
                state.main.absences.status_filter.label() != "All",
            ),
            Span::raw(" "),
            filter_chip(
                &format!("Window: {}", state.main.absences.window_filter.label()),
                state.main.absences.window_filter.label() != "All time",
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
                !state.main.absences.search_query.is_empty(),
            ),
            Span::raw(" "),
            filter_chip("Clear", false),
        ])
    };
    frame.render_widget(Paragraph::new(filter_line), layout[2]);

    let hint_line = if state.main.absences.search_open {
        Line::from(vec![
            Span::styled("Search: ", Style::default().fg(BRAND)),
            Span::raw(render_input_text(
                &state.main.absences.search_input.value,
                state.main.absences.search_input.cursor,
                false,
            )),
        ])
    } else {
        Line::from(Span::styled(
            absence_prefetch_hint(state, filtered.len()),
            Style::default().fg(DIM_GRAY),
        ))
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
        has_loaded_absences,
    );
    render_absence_summary_pane(
        frame,
        summary_area,
        filtered_excused,
        filtered_unexcused,
        &state.main.absences.window_filter.label(),
        &newest_loaded,
        &oldest_loaded,
    );
    render_absence_details_pane(frame, details_area, selected);
}

pub(crate) fn absence_layout_geometry(
    terminal_width: u16,
    terminal_height: u16,
    filtered_len: usize,
    selected_idx: usize,
) -> AbsenceLayoutGeometry {
    let area = Rect {
        x: 0,
        y: SHELL_HEADER_HEIGHT,
        width: terminal_width,
        height: terminal_height.saturating_sub(SHELL_HEADER_HEIGHT),
    };
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

    let history_area = if area.width >= 118 {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(((f32::from(layout[4].width) * 0.6).floor() as u16).max(56)),
                Constraint::Min(28),
            ])
            .split(layout[4]);
        body[0]
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
        stacked[0]
    };

    let history_inner = Rect {
        x: history_area.x.saturating_add(1),
        y: history_area.y.saturating_add(1),
        width: history_area.width.saturating_sub(2),
        height: history_area.height.saturating_sub(2),
    };
    let list_rows = usize::from(history_inner.height).saturating_sub(2).max(3);
    let visible_start = std::cmp::min(
        selected_idx.saturating_sub(list_rows.saturating_div(2)),
        filtered_len.saturating_sub(list_rows),
    );
    let visible_rows = filtered_len.saturating_sub(visible_start).min(list_rows);

    AbsenceLayoutGeometry {
        history_area,
        history_inner,
        visible_start,
        visible_rows,
    }
}

pub(crate) fn hit_test_absence_history_click(
    terminal_width: u16,
    terminal_height: u16,
    filtered_len: usize,
    selected_idx: usize,
    column: u16,
    row: u16,
) -> Option<AbsenceClickTarget> {
    let geometry =
        absence_layout_geometry(terminal_width, terminal_height, filtered_len, selected_idx);

    if geometry.history_inner.width == 0 || geometry.history_inner.height == 0 {
        return None;
    }
    if column < geometry.history_inner.x
        || column
            >= geometry
                .history_inner
                .x
                .saturating_add(geometry.history_inner.width)
    {
        return None;
    }

    let body_start_y = geometry.history_inner.y.saturating_add(1);
    let body_end_y = body_start_y.saturating_add(geometry.visible_rows as u16);
    if row < body_start_y || row >= body_end_y {
        return None;
    }

    Some(AbsenceClickTarget {
        selected_idx: geometry.visible_start + usize::from(row.saturating_sub(body_start_y)),
    })
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
    has_loaded_absences: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_GRAY))
        .title("History")
        .title(
            Line::from(format!("{selected_count}/{}", filtered.len())).alignment(Alignment::Right),
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
    let state_header = format!("{:^width$}", "State", width = status_chip_width);
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

    let mut lines = vec![Line::from(vec![
        styled_cell("  ", Some(HEADER_BG), Some(DIM_GRAY)),
        styled_cell(
            &fit_text("When", date_col_width),
            Some(HEADER_BG),
            Some(DIM_GRAY),
        ),
        styled_cell(" ", Some(HEADER_BG), Some(DIM_GRAY)),
        styled_cell(
            &fit_text("Notes", note_col_width),
            Some(HEADER_BG),
            Some(DIM_GRAY),
        ),
        styled_cell(" ", Some(HEADER_BG), Some(DIM_GRAY)),
        styled_cell(
            &fit_text(&state_header, status_chip_width),
            Some(HEADER_BG),
            Some(DIM_GRAY),
        ),
    ])];

    if state.main.absences.loading_initial {
        lines.extend(centered_message_lines(
            "Loading absences...",
            inner.height.saturating_sub(2),
            inner.width,
            Style::default().fg(WARNING),
        ));
    } else if !state.main.absences.error.is_empty() && !has_loaded_absences {
        lines.extend(centered_message_lines(
            &state.main.absences.error,
            inner.height.saturating_sub(2),
            inner.width,
            Style::default().fg(ERROR),
        ));
    } else if filtered.is_empty() && !has_loaded_absences {
        lines.extend(centered_message_lines(
            "No absences found in loaded history.",
            inner.height.saturating_sub(2),
            inner.width,
            Style::default().fg(WARNING),
        ));
    } else if filtered.is_empty() {
        lines.extend(centered_message_lines(
            if has_active_filters {
                "No absences match current filters."
            } else {
                "No absences found in loaded history."
            },
            inner.height.saturating_sub(2),
            inner.width,
            Style::default().fg(WARNING),
        ));
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
                &to_single_line(if absence.text.is_empty() {
                    if absence.reason.is_empty() {
                        "No reason"
                    } else {
                        &absence.reason
                    }
                } else {
                    &absence.text
                }),
                note_col_width,
            );
            lines.push(Line::from(vec![
                styled_cell(
                    if is_selected { "> " } else { "  " },
                    row_bg,
                    Some(if is_selected { BRAND } else { BORDER_GRAY }),
                ),
                styled_cell(
                    &fit_text(&format_absence_range_compact(absence), date_col_width),
                    row_bg,
                    Some(if is_selected {
                        Color::Indexed(15)
                    } else {
                        DIM_GRAY
                    }),
                ),
                styled_cell(" ", row_bg, None),
                styled_cell(
                    &fit_text(&note, note_col_width),
                    row_bg,
                    Some(Color::Indexed(15)),
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
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        if state.main.absences.loading_more {
            lines.push(Line::from(Span::styled(
                fit_text("Loading older records...", usize::from(inner.width)),
                Style::default().fg(WARNING),
            )));
        } else if state.main.absences.has_more {
            lines.push(Line::from(Span::styled(
                fit_text(
                    "More records available - press m or keep scrolling",
                    usize::from(inner.width),
                ),
                Style::default().fg(DIM_GRAY),
            )));
        } else if !filtered.is_empty() {
            lines.push(Line::from(Span::styled(
                fit_text("End of available history", usize::from(inner.width)),
                Style::default().fg(DIM_GRAY),
            )));
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
    oldest_loaded: &str,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_GRAY))
        .title("Summary")
        .title(Line::from(window_label).alignment(Alignment::Right));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let lines = vec![
        Line::from(format!(
            "{excused_count} excused | {unexcused_count} unexcused"
        )),
        Line::from(Span::styled(
            format!("Loaded range: {newest_loaded} -> {oldest_loaded}"),
            Style::default().fg(DIM_GRAY),
        )),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_absence_details_pane(
    frame: &mut Frame,
    area: Rect,
    selected: Option<crate::models::ParsedAbsence>,
) {
    let block = if let Some(absence) = &selected {
        let status = absence_status_meta(absence.is_excused);
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_GRAY))
            .title("Details")
            .title(Line::from(status.chip_label).alignment(Alignment::Right))
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_GRAY))
            .title("Details")
            .title(Line::from("No selection").alignment(Alignment::Right))
    };
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines = Vec::new();
    if let Some(absence) = selected {
        lines.extend([
            Line::from(Span::styled("When", Style::default().fg(DIM_GRAY))),
            Line::from(format_absence_range_full(&absence)),
            Line::from(Span::styled("Reason", Style::default().fg(DIM_GRAY))),
            Line::from(to_single_line(if absence.reason.is_empty() {
                "No reason"
            } else {
                &absence.reason
            })),
            Line::from(Span::styled("Excuse status", Style::default().fg(DIM_GRAY))),
            Line::from(to_single_line(if absence.excuse_status.is_empty() {
                absence_status_meta(absence.is_excused).long_label
            } else {
                &absence.excuse_status
            })),
            Line::from(Span::styled("Notes", Style::default().fg(DIM_GRAY))),
            Line::from(to_single_line(if absence.text.is_empty() {
                "No additional notes"
            } else {
                &absence.text
            })),
        ]);
    } else {
        lines.push(Line::from(Span::styled(
            "Select a record from the history list.",
            Style::default().fg(DIM_GRAY),
        )));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn format_absence_range_compact(absence: &crate::models::ParsedAbsence) -> String {
    let start = format!(
        "{:02}.{:02}",
        absence.start_date.day(),
        absence.start_date.month()
    );
    let end = format!(
        "{:02}.{:02}",
        absence.end_date.day(),
        absence.end_date.month()
    );
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
        format!(
            "Showing {} of {}",
            filtered.len(),
            state.main.absences.absences.len()
        )
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

fn absence_prefetch_hint(state: &AppState, _filtered_len: usize) -> String {
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
        } else {
            format!(
                "Auto-load starts near the bottom ({} rows early). Press m to fetch now.",
                prefetch_threshold
            )
        }
    } else {
        "Reached oldest available records in loaded history.".to_owned()
    }
}

use super::theme::{ ALT_BG, BRAND, DIM_GRAY, SELECT_BG };
use ratatui::layout::{ Constraint, Direction, Layout, Rect };
use ratatui::style::{ Color, Modifier, Style };
use ratatui::text::{ Line, Span };
use unicode_width::UnicodeWidthStr;

pub(super) fn line_with_right(
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

pub(super) fn filter_chip(label: &str, active: bool) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(if active { Color::Indexed(15) } else { DIM_GRAY })
            .bg(if active { SELECT_BG } else { Color::Reset })
    )
}

pub(super) fn truncate_text(value: &str, width: usize) -> String {
    fit_text(value, width).trim_end().to_owned()
}

pub(super) fn to_single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn centered_message_lines(
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

pub(super) fn login_field_line(
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

pub(super) fn tab_span(label: &str, active: bool) -> Span<'static> {
    Span::styled(
        label.to_owned(),
        Style::default()
            .fg(if active { Color::Black } else { Color::White })
            .bg(if active { BRAND } else { ALT_BG })
            .add_modifier(if active { Modifier::BOLD } else { Modifier::empty() })
    )
}

pub(super) fn styled_cell(text: &str, bg: Option<Color>, fg: Option<Color>) -> Span<'static> {
    Span::styled(
        text.to_owned(),
        Style::default().bg(bg.unwrap_or(Color::Reset)).fg(fg.unwrap_or(Color::Reset))
    )
}

pub(super) fn render_input_text(value: &str, cursor: usize, mask: bool) -> String {
    let value = if mask { "*".repeat(value.chars().count()) } else { value.to_owned() };
    if value.is_empty() {
        return "_".to_owned();
    }
    if cursor >= value.len() {
        return format!("{value}_");
    }
    value
}

pub(super) fn fit_text(value: &str, width: usize) -> String {
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

pub(super) fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
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

use super::theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
pub(super) fn line_with_right(
    left: &str,
    right: &str,
    width: usize,
    left_style: Style,
    right_style: Style,
) -> Line<'static> {
    tui_components::ui::text::line_with_right(left, right, width, left_style, right_style)
}

pub(super) fn filter_chip(label: &str, active: bool) -> Span<'static> {
    tui_components::ui::widgets::filter_chip(label, active, theme::components_theme())
}

pub(super) fn truncate_text(value: &str, width: usize) -> String {
    tui_components::ui::text::truncate_text(value, width)
}

pub(super) fn to_single_line(value: &str) -> String {
    tui_components::ui::text::to_single_line(value)
}

pub(super) fn centered_message_lines(
    message: &str,
    height: u16,
    width: u16,
    style: Style,
) -> Vec<Line<'static>> {
    tui_components::ui::widgets::centered_message_lines(message, height, width, style)
}

pub(super) fn tab_span(label: &str, active: bool) -> Span<'static> {
    tui_components::ui::widgets::tab_span(label, active, theme::components_theme())
}

pub(super) fn styled_cell(text: &str, bg: Option<Color>, fg: Option<Color>) -> Span<'static> {
    tui_components::ui::widgets::styled_cell(text, bg, fg)
}

pub(super) fn render_input_text(value: &str, cursor: usize, mask: bool) -> String {
    tui_components::ui::text::render_input_text(value, cursor, mask)
}

pub(super) fn fit_text(value: &str, width: usize) -> String {
    tui_components::ui::text::fit_text(value, width)
}

pub(super) fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    tui_components::ui::layout::centered_rect_percent(width_percent, height_percent, area)
}

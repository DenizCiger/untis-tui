use crate::models::{DayTimetable, ParsedLesson, TimeUnit, WeekTimetable, parse_time_to_minutes};
use chrono::Local;
use std::collections::HashMap;

pub const GRID_ROW_HEIGHT: u16 = 3;
pub const TITLE_ROWS: u16 = 2;
pub const DAY_HEADER_ROWS: u16 = 2;
pub const DAY_COUNT: usize = 5;
pub const MAX_SCROLL_HINT_ROWS: u16 = 2;
pub const MIN_DETAILS_HEIGHT: u16 = 6;
pub const SHELL_HEADER_HEIGHT: u16 = 2;
pub const COMPACT_WIDTH_BREAKPOINT: u16 = 90;
pub const COMPACT_HEIGHT_BREAKPOINT: u16 = 24;
pub const SPLIT_DAY_COLUMN_MIN_WIDTH: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Continuation {
    Single,
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderLesson {
    pub lesson: ParsedLesson,
    pub continuation: Continuation,
    pub lesson_key: String,
    pub lesson_instance_id: String,
    pub continuity_key: String,
}

pub type DayLessonIndex = HashMap<String, Vec<RenderLesson>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayPeriod {
    pub split: bool,
    pub lanes: Vec<Option<RenderLesson>>,
    pub hidden_count: usize,
}

pub type DayOverlayIndex = HashMap<String, OverlayPeriod>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimetableRenderModel {
    pub day_lesson_index: Vec<DayLessonIndex>,
    pub overlay_index_by_day: Vec<DayOverlayIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedLessonRange {
    pub lesson: ParsedLesson,
    pub lesson_key: String,
    pub lesson_instance_id: String,
    pub start_period_idx: usize,
    pub end_period_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimetableGridGeometry {
    pub grid_x: u16,
    pub grid_y: u16,
    pub grid_height: u16,
    pub time_width: u16,
    pub day_width: u16,
    pub scroll_offset: usize,
    pub visible_period_count: usize,
    pub has_top_scroll_hint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimetableClickTarget {
    pub day_idx: usize,
    pub period_idx: usize,
    pub lesson_idx: usize,
}

pub fn is_compact(width: u16, height: u16) -> bool {
    width < COMPACT_WIDTH_BREAKPOINT || height < COMPACT_HEIGHT_BREAKPOINT
}

pub fn time_column_width(width: u16, height: u16) -> u16 {
    if is_compact(width, height) { 12 } else { 16 }
}

pub fn day_column_width(width: u16, height: u16) -> u16 {
    let compact = is_compact(width, height);
    let time_width = usize::from(time_column_width(width, height));
    let min_width = if compact { 10 } else { 14 };
    let calculated = (usize::from(width).saturating_sub(time_width) / 5).max(min_width);
    calculated as u16
}

pub fn timetable_body_height_from_terminal(terminal_height: u16) -> u16 {
    terminal_height.saturating_sub(SHELL_HEADER_HEIGHT)
}

pub fn timetable_rows_per_page(body_height: u16) -> usize {
    let grid_budget = body_height
        .saturating_sub(TITLE_ROWS + DAY_HEADER_ROWS + MAX_SCROLL_HINT_ROWS + MIN_DETAILS_HEIGHT);
    usize::from(grid_budget.max(GRID_ROW_HEIGHT) / GRID_ROW_HEIGHT).max(1)
}

pub fn timetable_grid_geometry(
    terminal_width: u16,
    terminal_height: u16,
    timegrid_len: usize,
    scroll_offset: usize,
) -> TimetableGridGeometry {
    let timetable_height = timetable_body_height_from_terminal(terminal_height);
    let body_height = timetable_height.saturating_sub(TITLE_ROWS);
    let details_min_height = MIN_DETAILS_HEIGHT.min(body_height);
    let time_width = time_column_width(terminal_width, timetable_height);
    let day_width = day_column_width(terminal_width, timetable_height);
    let rows_per_page = timetable_rows_per_page(timetable_height).max(1);
    let max_scroll = timegrid_len.saturating_sub(rows_per_page);
    let scroll_offset = scroll_offset.min(max_scroll);
    let visible_period_count = timegrid_len
        .saturating_sub(scroll_offset)
        .min(rows_per_page);
    let has_top_scroll_hint = scroll_offset > 0;
    let has_bottom_scroll_hint =
        timegrid_len.saturating_sub(scroll_offset + visible_period_count) > 0;
    let grid_line_count = DAY_HEADER_ROWS
        + if has_top_scroll_hint { 1 } else { 0 }
        + (visible_period_count as u16).saturating_mul(GRID_ROW_HEIGHT)
        + if has_bottom_scroll_hint { 1 } else { 0 };
    let max_grid_height = body_height.saturating_sub(details_min_height);

    TimetableGridGeometry {
        grid_x: 0,
        grid_y: SHELL_HEADER_HEIGHT + TITLE_ROWS,
        grid_height: grid_line_count.min(max_grid_height),
        time_width,
        day_width,
        scroll_offset,
        visible_period_count,
        has_top_scroll_hint,
    }
}

pub fn build_render_model(data: &WeekTimetable, lane_count: usize) -> TimetableRenderModel {
    let day_lesson_index = index_lessons_by_period(&data.days, &data.timegrid);
    let overlay_index_by_day = day_lesson_index
        .iter()
        .map(|day_index| build_overlay_index(day_index, &data.timegrid, lane_count))
        .collect();
    TimetableRenderModel {
        day_lesson_index,
        overlay_index_by_day,
    }
}

pub fn index_lessons_by_period(
    days: &[DayTimetable],
    timegrid: &[TimeUnit],
) -> Vec<DayLessonIndex> {
    let period_ranges = timegrid
        .iter()
        .map(|period| PeriodRange {
            start_time: period.start_time.clone(),
            start_minutes: parse_time_to_minutes(&period.start_time),
            end_minutes: parse_time_to_minutes(&period.end_time),
        })
        .collect::<Vec<_>>();

    days.iter()
        .map(|day| {
            let mut indexed = DayLessonIndex::new();
            let mut sorted_lessons = day.lessons.clone();
            sorted_lessons.sort_by(compare_lessons_for_display);

            let lessons_by_period = period_ranges
                .iter()
                .map(|period| {
                    let mut lessons = sorted_lessons
                        .iter()
                        .filter(|lesson| {
                            lesson_intersects_period(
                                lesson,
                                period.start_minutes,
                                period.end_minutes,
                            )
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    lessons.sort_by(|left, right| {
                        compare_lessons_for_period(left, right, &period.start_time)
                    });
                    lessons
                })
                .collect::<Vec<_>>();

            let key_counts_by_period = lessons_by_period
                .iter()
                .map(|lessons_in_period| {
                    let mut counts = HashMap::<String, usize>::new();
                    for lesson in lessons_in_period {
                        let lesson_key = lesson_key(lesson);
                        *counts.entry(lesson_key).or_insert(0) += 1;
                    }
                    counts
                })
                .collect::<Vec<_>>();

            for (period_idx, period) in period_ranges.iter().enumerate() {
                let lessons_in_period = lessons_by_period
                    .get(period_idx)
                    .cloned()
                    .unwrap_or_default();
                if lessons_in_period.is_empty() {
                    continue;
                }

                let mut seen_in_period = HashMap::<String, usize>::new();
                let rendered = lessons_in_period
                    .into_iter()
                    .map(|lesson| {
                        let lesson_key = lesson_key(&lesson);
                        let occurrence = *seen_in_period.get(&lesson_key).unwrap_or(&0);
                        seen_in_period.insert(lesson_key.clone(), occurrence + 1);

                        let previous_count = if period_idx > 0 {
                            key_counts_by_period[period_idx - 1]
                                .get(&lesson_key)
                                .copied()
                                .unwrap_or(0)
                        } else {
                            0
                        };
                        let next_count = if period_idx + 1 < period_ranges.len() {
                            key_counts_by_period[period_idx + 1]
                                .get(&lesson_key)
                                .copied()
                                .unwrap_or(0)
                        } else {
                            0
                        };

                        let has_previous = previous_count > occurrence;
                        let has_next = next_count > occurrence;
                        let continuation = match (has_previous, has_next) {
                            (true, true) => Continuation::Middle,
                            (true, false) => Continuation::End,
                            (false, true) => Continuation::Start,
                            (false, false) => Continuation::Single,
                        };

                        let lesson_instance_id = if lesson.instance_id.is_empty() {
                            lesson_key.clone()
                        } else {
                            lesson.instance_id.clone()
                        };

                        RenderLesson {
                            lesson,
                            continuation,
                            lesson_key: lesson_key.clone(),
                            lesson_instance_id,
                            continuity_key: format!("{lesson_key}#{occurrence}"),
                        }
                    })
                    .collect::<Vec<_>>();

                indexed.insert(period.start_time.clone(), rendered);
            }

            indexed
        })
        .collect()
}

pub fn build_overlay_index(
    day_index: &DayLessonIndex,
    timegrid: &[TimeUnit],
    lane_count: usize,
) -> DayOverlayIndex {
    let mut overlay = DayOverlayIndex::new();
    let lanes = lane_count.max(1);
    let mut previous_lane_keys = vec![None::<String>; lanes];

    for (period_idx, period) in timegrid.iter().enumerate() {
        let entries = day_index
            .get(&period.start_time)
            .cloned()
            .unwrap_or_default();
        let should_split = entries.len() > 1
            || should_reserve_split_for_single(day_index, timegrid, period_idx, &entries);

        if !should_split {
            overlay.insert(
                period.start_time.clone(),
                OverlayPeriod {
                    split: false,
                    lanes: vec![None; lanes],
                    hidden_count: 0,
                },
            );
            previous_lane_keys.fill(None);
            continue;
        }

        let mut lane_entries = vec![None; lanes];
        let mut remaining = entries;

        for lane_idx in 0..lanes {
            let Some(previous_key) = previous_lane_keys[lane_idx].as_ref() else {
                continue;
            };
            if let Some(match_idx) = remaining
                .iter()
                .position(|entry| &entry.continuity_key == previous_key)
            {
                lane_entries[lane_idx] = Some(remaining.remove(match_idx));
            }
        }

        if !lane_entries.is_empty() && lane_entries[0].is_none() {
            if let Some(candidate) = pick_left_lane_candidate(&remaining) {
                remove_from_remaining(&mut remaining, &candidate.continuity_key);
                lane_entries[0] = Some(candidate);
            }
        }

        if lane_entries.len() >= 2 && lane_entries[1].is_none() {
            if let Some(candidate) = pick_right_lane_candidate(&remaining) {
                remove_from_remaining(&mut remaining, &candidate.continuity_key);
                lane_entries[1] = Some(candidate);
            }
        }

        for lane in lane_entries.iter_mut().skip(2) {
            if lane.is_none() && !remaining.is_empty() {
                *lane = Some(remaining.remove(0));
            }
        }

        previous_lane_keys = lane_entries
            .iter()
            .map(|entry| entry.as_ref().map(|value| value.continuity_key.clone()))
            .collect();

        overlay.insert(
            period.start_time.clone(),
            OverlayPeriod {
                split: true,
                lanes: lane_entries,
                hidden_count: remaining.len(),
            },
        );
    }

    overlay
}

pub fn lessons_for_period<'a>(
    model: &'a TimetableRenderModel,
    timegrid: &[TimeUnit],
    day_idx: usize,
    period_idx: usize,
) -> &'a [RenderLesson] {
    let Some(day_index) = model.day_lesson_index.get(day_idx) else {
        return &[];
    };
    let Some(period) = timegrid.get(period_idx) else {
        return &[];
    };
    day_index
        .get(&period.start_time)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn hit_test_timetable_click(
    data: &WeekTimetable,
    model: &TimetableRenderModel,
    terminal_width: u16,
    terminal_height: u16,
    scroll_offset: usize,
    column: u16,
    row: u16,
) -> Option<TimetableClickTarget> {
    let geometry = timetable_grid_geometry(
        terminal_width,
        terminal_height,
        data.timegrid.len(),
        scroll_offset,
    );

    if row < geometry.grid_y || row >= geometry.grid_y.saturating_add(geometry.grid_height) {
        return None;
    }
    if column < geometry.grid_x || column >= terminal_width {
        return None;
    }

    let mut local_y = row.saturating_sub(geometry.grid_y);
    if local_y < DAY_HEADER_ROWS {
        return None;
    }
    local_y -= DAY_HEADER_ROWS;
    if geometry.has_top_scroll_hint {
        if local_y == 0 {
            return None;
        }
        local_y -= 1;
    }

    let visible_row_height = (geometry.visible_period_count as u16).saturating_mul(GRID_ROW_HEIGHT);
    if local_y >= visible_row_height {
        return None;
    }

    if column < geometry.time_width {
        return None;
    }
    let local_x = column.saturating_sub(geometry.time_width);
    let day_idx = usize::from(local_x / geometry.day_width);
    if day_idx >= DAY_COUNT || day_idx >= data.days.len() {
        return None;
    }

    let within_day = local_x % geometry.day_width;
    if within_day == 0 {
        return None;
    }
    let cell_x = within_day - 1;
    let period_idx = geometry.scroll_offset + usize::from(local_y / GRID_ROW_HEIGHT);
    if period_idx >= data.timegrid.len() {
        return None;
    }

    let lesson_idx = clicked_lesson_index(
        model,
        data,
        day_idx,
        period_idx,
        geometry.day_width.saturating_sub(1),
        cell_x,
    );

    Some(TimetableClickTarget {
        day_idx,
        period_idx,
        lesson_idx,
    })
}

pub fn visible_lesson_index_order(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    period_idx: usize,
) -> Vec<usize> {
    let lessons = lessons_for_period(model, &data.timegrid, day_idx, period_idx);
    if lessons.is_empty() {
        return Vec::new();
    }

    let Some(day_overlay) = model.overlay_index_by_day.get(day_idx) else {
        return (0..lessons.len()).collect();
    };
    let Some(period) = data.timegrid.get(period_idx) else {
        return (0..lessons.len()).collect();
    };
    let Some(overlay) = day_overlay.get(&period.start_time) else {
        return (0..lessons.len()).collect();
    };
    if !overlay.split {
        return (0..lessons.len()).collect();
    }

    let mut ordered = Vec::new();
    for lane_entry in overlay.lanes.iter().flatten() {
        if let Some(index) = lessons.iter().position(|entry| {
            entry.continuity_key == lane_entry.continuity_key
                || entry.lesson_instance_id == lane_entry.lesson_instance_id
        }) {
            if !ordered.contains(&index) {
                ordered.push(index);
            }
        }
    }

    for index in 0..lessons.len() {
        if !ordered.contains(&index) {
            ordered.push(index);
        }
    }

    ordered
}

fn clicked_lesson_index(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    period_idx: usize,
    cell_width: u16,
    cell_x: u16,
) -> usize {
    let lessons = lessons_for_period(model, &data.timegrid, day_idx, period_idx);
    if lessons.is_empty() {
        return 0;
    }

    let first_visible = visible_lesson_index_order(model, data, day_idx, period_idx)
        .into_iter()
        .next()
        .unwrap_or(0);

    if cell_width + 1 < SPLIT_DAY_COLUMN_MIN_WIDTH {
        return first_visible;
    }

    let Some(period) = data.timegrid.get(period_idx) else {
        return first_visible;
    };
    let Some(day_overlay) = model.overlay_index_by_day.get(day_idx) else {
        return first_visible;
    };
    let Some(overlay) = day_overlay.get(&period.start_time) else {
        return first_visible;
    };
    if !overlay.split {
        return first_visible;
    }

    let split_gap_width = 1u16;
    let left_lane_width = cell_width.saturating_sub(split_gap_width) / 2;
    let right_lane_start = left_lane_width + split_gap_width;
    let lane_idx = if cell_x < left_lane_width {
        Some(0)
    } else if cell_x >= right_lane_start {
        Some(1)
    } else {
        None
    };

    lane_idx
        .and_then(|index| overlay.lanes.get(index))
        .and_then(Option::as_ref)
        .and_then(|entry| {
            lessons.iter().position(|lesson| {
                lesson.continuity_key == entry.continuity_key
                    || lesson.lesson_instance_id == entry.lesson_instance_id
            })
        })
        .unwrap_or(first_visible)
}

pub fn cycle_visible_lesson_index(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    period_idx: usize,
    selected_lesson_idx: usize,
) -> usize {
    let ordered = visible_lesson_index_order(model, data, day_idx, period_idx);
    if ordered.len() <= 1 {
        return selected_lesson_idx.min(ordered.first().copied().unwrap_or(0));
    }

    let current_order_idx = ordered
        .iter()
        .position(|index| *index == selected_lesson_idx)
        .unwrap_or(0);
    ordered[(current_order_idx + 1) % ordered.len()]
}

pub fn selection_index_for_period_change(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    from_period_idx: usize,
    to_period_idx: usize,
    selected_lesson_idx: usize,
) -> usize {
    let from_lessons = lessons_for_period(model, &data.timegrid, day_idx, from_period_idx);
    let to_lessons = lessons_for_period(model, &data.timegrid, day_idx, to_period_idx);
    if to_lessons.is_empty() {
        return 0;
    }

    let Some(selected_entry) = from_lessons.get(selected_lesson_idx) else {
        return selected_lesson_idx.min(to_lessons.len().saturating_sub(1));
    };

    if let Some(index) = to_lessons
        .iter()
        .position(|entry| entry.continuity_key == selected_entry.continuity_key)
    {
        return index;
    }

    if let Some(index) = to_lessons
        .iter()
        .position(|entry| entry.lesson_instance_id == selected_entry.lesson_instance_id)
    {
        return index;
    }

    let Some(day_overlay) = model.overlay_index_by_day.get(day_idx) else {
        return 0;
    };
    let Some(from_period) = data.timegrid.get(from_period_idx) else {
        return 0;
    };
    let Some(to_period) = data.timegrid.get(to_period_idx) else {
        return 0;
    };
    let from_overlay = day_overlay.get(&from_period.start_time);
    let to_overlay = day_overlay.get(&to_period.start_time);

    if let (Some(from_overlay), Some(to_overlay)) = (from_overlay, to_overlay) {
        if from_overlay.split && to_overlay.split {
            if let Some(from_lane_idx) = from_overlay.lanes.iter().position(|entry| {
                entry
                    .as_ref()
                    .map(|value| value.lesson_instance_id == selected_entry.lesson_instance_id)
                    .unwrap_or(false)
            }) {
                if let Some(target_lane_entry) =
                    to_overlay.lanes.get(from_lane_idx).and_then(Option::as_ref)
                {
                    if let Some(index) = to_lessons.iter().position(|entry| {
                        entry.lesson_instance_id == target_lane_entry.lesson_instance_id
                    }) {
                        return index;
                    }
                }
            }
        }
    }

    0
}

pub fn selected_lesson_position(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    period_idx: usize,
    selected_lesson_idx: usize,
) -> usize {
    let lessons = lessons_for_period(model, &data.timegrid, day_idx, period_idx);
    if lessons.is_empty() {
        return 0;
    }

    let Some(selected_entry) = lessons.get(selected_lesson_idx) else {
        return 0;
    };

    let Some(day_overlay) = model.overlay_index_by_day.get(day_idx) else {
        return selected_lesson_idx.min(lessons.len().saturating_sub(1)) + 1;
    };
    let Some(period) = data.timegrid.get(period_idx) else {
        return selected_lesson_idx.min(lessons.len().saturating_sub(1)) + 1;
    };
    let Some(overlay) = day_overlay.get(&period.start_time) else {
        return selected_lesson_idx.min(lessons.len().saturating_sub(1)) + 1;
    };
    if !overlay.split {
        return selected_lesson_idx.min(lessons.len().saturating_sub(1)) + 1;
    }

    if let Some(lane_idx) = overlay.lanes.iter().position(|entry| {
        entry
            .as_ref()
            .map(|value| {
                value.continuity_key == selected_entry.continuity_key
                    || value.lesson_instance_id == selected_entry.lesson_instance_id
            })
            .unwrap_or(false)
    }) {
        let position = overlay
            .lanes
            .iter()
            .take(lane_idx + 1)
            .filter(|entry| entry.is_some())
            .count();
        return position.min(lessons.len());
    }

    let ordered = visible_lesson_index_order(model, data, day_idx, period_idx);
    ordered
        .iter()
        .position(|index| *index == selected_lesson_idx)
        .map(|position| position + 1)
        .unwrap_or_else(|| selected_lesson_idx.min(lessons.len().saturating_sub(1)) + 1)
}

pub fn selected_lesson_range(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    period_idx: usize,
    selected_lesson_idx: usize,
) -> Option<SelectedLessonRange> {
    let lessons = lessons_for_period(model, &data.timegrid, day_idx, period_idx);
    let selected_entry = lessons.get(selected_lesson_idx)?;

    let mut start_period_idx = period_idx;
    while start_period_idx > 0 {
        let previous_lessons =
            lessons_for_period(model, &data.timegrid, day_idx, start_period_idx - 1);
        if !previous_lessons
            .iter()
            .any(|entry| entry.lesson_instance_id == selected_entry.lesson_instance_id)
        {
            break;
        }
        start_period_idx -= 1;
    }

    let mut end_period_idx = period_idx;
    while end_period_idx + 1 < data.timegrid.len() {
        let next_lessons = lessons_for_period(model, &data.timegrid, day_idx, end_period_idx + 1);
        if !next_lessons
            .iter()
            .any(|entry| entry.lesson_instance_id == selected_entry.lesson_instance_id)
        {
            break;
        }
        end_period_idx += 1;
    }

    Some(SelectedLessonRange {
        lesson: selected_entry.lesson.clone(),
        lesson_key: selected_entry.lesson_key.clone(),
        lesson_instance_id: selected_entry.lesson_instance_id.clone(),
        start_period_idx,
        end_period_idx,
    })
}

pub fn find_current_period_index(timegrid: &[TimeUnit]) -> Option<usize> {
    let now = Local::now();
    let current_time = format!("{:02}:{:02}", now.hour(), now.minute());
    timegrid
        .iter()
        .position(|period| current_time >= period.start_time && current_time <= period.end_time)
}

pub fn find_next_lesson_period_index(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    from_period_idx: usize,
    direction: isize,
) -> Option<usize> {
    if data.timegrid.is_empty() {
        return None;
    }

    let max_period = data.timegrid.len().saturating_sub(1) as isize;
    let mut period_idx = (from_period_idx as isize) + direction;
    while period_idx >= 0 && period_idx <= max_period {
        if !lessons_for_period(model, &data.timegrid, day_idx, period_idx as usize).is_empty() {
            return Some(period_idx as usize);
        }
        period_idx += direction;
    }
    None
}

pub fn find_edge_lesson_period_index(
    model: &TimetableRenderModel,
    data: &WeekTimetable,
    day_idx: usize,
    from_start: bool,
) -> usize {
    if data.timegrid.is_empty() {
        return 0;
    }

    if from_start {
        for index in 0..data.timegrid.len() {
            if !lessons_for_period(model, &data.timegrid, day_idx, index).is_empty() {
                return index;
            }
        }
        return 0;
    }

    for index in (0..data.timegrid.len()).rev() {
        if !lessons_for_period(model, &data.timegrid, day_idx, index).is_empty() {
            return index;
        }
    }
    data.timegrid.len().saturating_sub(1)
}

fn lesson_key(lesson: &ParsedLesson) -> String {
    [
        lesson.subject.as_str(),
        lesson.teacher.as_str(),
        lesson.room.as_str(),
        if lesson.cancelled { "1" } else { "0" },
        if lesson.substitution { "1" } else { "0" },
    ]
    .join("|")
}

fn lesson_intersects_period(
    lesson: &ParsedLesson,
    period_start_minutes: i32,
    period_end_minutes: i32,
) -> bool {
    let lesson_start_minutes = parse_time_to_minutes(&lesson.start_time);
    let lesson_end_minutes = parse_time_to_minutes(&lesson.end_time);
    lesson_start_minutes < period_end_minutes && lesson_end_minutes > period_start_minutes
}

fn compare_lessons_for_display(left: &ParsedLesson, right: &ParsedLesson) -> std::cmp::Ordering {
    left.start_time
        .cmp(&right.start_time)
        .then_with(|| left.end_time.cmp(&right.end_time))
        .then_with(|| left.subject.cmp(&right.subject))
        .then_with(|| left.teacher.cmp(&right.teacher))
        .then_with(|| left.room.cmp(&right.room))
        .then_with(|| left.instance_id.cmp(&right.instance_id))
}

fn compare_lessons_for_period(
    left: &ParsedLesson,
    right: &ParsedLesson,
    period_start: &str,
) -> std::cmp::Ordering {
    let left_starts_here = left.start_time == period_start;
    let right_starts_here = right.start_time == period_start;
    if left_starts_here != right_starts_here {
        return if left_starts_here {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        };
    }
    compare_lessons_for_display(left, right)
}

fn should_reserve_split_for_single(
    day_index: &DayLessonIndex,
    _timegrid: &[TimeUnit],
    _period_idx: usize,
    entries: &[RenderLesson],
) -> bool {
    if entries.len() != 1 {
        return false;
    }

    let entry = &entries[0];
    if entry.continuation == Continuation::Single {
        return false;
    }

    day_index.values().any(|period_entries| {
        period_entries.len() > 1
            && period_entries.iter().any(|other| {
                other.continuity_key == entry.continuity_key
                    || other.lesson_instance_id == entry.lesson_instance_id
            })
    })
}

fn pick_left_lane_candidate(entries: &[RenderLesson]) -> Option<RenderLesson> {
    entries
        .iter()
        .find(|entry| matches!(entry.continuation, Continuation::Middle | Continuation::End))
        .cloned()
        .or_else(|| entries.first().cloned())
}

fn pick_right_lane_candidate(entries: &[RenderLesson]) -> Option<RenderLesson> {
    entries
        .iter()
        .find(|entry| {
            matches!(
                entry.continuation,
                Continuation::Start | Continuation::Single
            )
        })
        .cloned()
        .or_else(|| entries.first().cloned())
}

fn remove_from_remaining(entries: &mut Vec<RenderLesson>, continuity_key: &str) {
    if let Some(index) = entries
        .iter()
        .position(|entry| entry.continuity_key == continuity_key)
    {
        entries.remove(index);
    }
}

struct PeriodRange {
    start_time: String,
    start_minutes: i32,
    end_minutes: i32,
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DayTimetable, ParsedLesson, TimeUnit, WeekTimetable};
    use chrono::NaiveDate;

    fn lesson(
        instance_id: &str,
        subject: &str,
        room: &str,
        teacher: &str,
        start_time: &str,
        end_time: &str,
    ) -> ParsedLesson {
        ParsedLesson {
            instance_id: instance_id.into(),
            subject: subject.into(),
            subject_long_name: subject.into(),
            lesson_text: String::new(),
            cell_state: String::new(),
            teacher: teacher.into(),
            teacher_long_name: teacher.into(),
            all_teachers: vec![teacher.into()],
            all_teacher_long_names: vec![teacher.into()],
            room: room.into(),
            room_long_name: room.into(),
            all_classes: vec!["1A".into()],
            start_time: start_time.into(),
            end_time: end_time.into(),
            cancelled: false,
            substitution: false,
            remarks: String::new(),
        }
    }

    fn sample_data(lessons: Vec<ParsedLesson>) -> WeekTimetable {
        WeekTimetable {
            days: vec![
                DayTimetable {
                    date: NaiveDate::from_ymd_opt(2026, 4, 6).unwrap(),
                    day_name: "Monday".into(),
                    lessons,
                },
                DayTimetable {
                    date: NaiveDate::from_ymd_opt(2026, 4, 7).unwrap(),
                    day_name: "Tuesday".into(),
                    lessons: Vec::new(),
                },
                DayTimetable {
                    date: NaiveDate::from_ymd_opt(2026, 4, 8).unwrap(),
                    day_name: "Wednesday".into(),
                    lessons: Vec::new(),
                },
                DayTimetable {
                    date: NaiveDate::from_ymd_opt(2026, 4, 9).unwrap(),
                    day_name: "Thursday".into(),
                    lessons: Vec::new(),
                },
                DayTimetable {
                    date: NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
                    day_name: "Friday".into(),
                    lessons: Vec::new(),
                },
            ],
            timegrid: vec![
                TimeUnit {
                    name: "1".into(),
                    start_time: "08:00".into(),
                    end_time: "08:50".into(),
                },
                TimeUnit {
                    name: "2".into(),
                    start_time: "08:50".into(),
                    end_time: "09:40".into(),
                },
                TimeUnit {
                    name: "3".into(),
                    start_time: "09:40".into(),
                    end_time: "10:30".into(),
                },
            ],
        }
    }

    fn sample_data_with_period_count(
        lessons: Vec<ParsedLesson>,
        period_count: usize,
    ) -> WeekTimetable {
        let timegrid = (0..period_count)
            .map(|index| {
                let start_minutes = 8 * 60 + index as i32 * 50;
                let end_minutes = start_minutes + 50;
                TimeUnit {
                    name: (index + 1).to_string(),
                    start_time: format!("{:02}:{:02}", start_minutes / 60, start_minutes % 60),
                    end_time: format!("{:02}:{:02}", end_minutes / 60, end_minutes % 60),
                }
            })
            .collect::<Vec<_>>();
        WeekTimetable {
            timegrid,
            ..sample_data(lessons)
        }
    }

    #[test]
    fn multi_period_lessons_get_continuation_tags() {
        let data = sample_data(vec![lesson("math", "M", "A1", "T", "08:00", "09:40")]);
        let model = build_render_model(&data, 2);
        let first = &lessons_for_period(&model, &data.timegrid, 0, 0)[0];
        let second = &lessons_for_period(&model, &data.timegrid, 0, 1)[0];
        assert_eq!(first.continuation, Continuation::Start);
        assert_eq!(second.continuation, Continuation::End);
    }

    #[test]
    fn overlay_reserves_split_lane_for_continuation() {
        let data = sample_data(vec![
            lesson("math", "M", "A1", "T", "08:00", "09:40"),
            lesson("eng", "E", "A2", "S", "08:00", "08:50"),
        ]);
        let model = build_render_model(&data, 2);
        let first = model.overlay_index_by_day[0].get("08:00").unwrap();
        let second = model.overlay_index_by_day[0].get("08:50").unwrap();
        assert!(first.split);
        assert!(second.split);
        assert_eq!(
            second.lanes[1]
                .as_ref()
                .map(|entry| entry.lesson.subject.as_str()),
            Some("M")
        );
    }

    #[test]
    fn overlay_tracks_hidden_count_beyond_visible_lanes() {
        let data = sample_data(vec![
            lesson("math", "M", "A1", "T", "08:00", "08:50"),
            lesson("eng", "E", "A2", "S", "08:00", "08:50"),
            lesson("bio", "B", "A3", "R", "08:00", "08:50"),
        ]);
        let model = build_render_model(&data, 2);
        let overlay = model.overlay_index_by_day[0].get("08:00").unwrap();
        assert!(overlay.split);
        assert_eq!(overlay.hidden_count, 1);
    }

    #[test]
    fn selection_mapping_prefers_visible_lane_continuity() {
        let data = sample_data(vec![
            lesson("math", "M", "A1", "T", "08:00", "09:40"),
            lesson("eng", "E", "A2", "S", "08:00", "08:50"),
            lesson("bio", "B", "A3", "R", "08:50", "09:40"),
        ]);
        let model = build_render_model(&data, 2);
        let next_index = selection_index_for_period_change(&model, &data, 0, 0, 1, 0);
        assert_eq!(next_index, 0);
        assert_eq!(
            lessons_for_period(&model, &data.timegrid, 0, 1)[next_index]
                .lesson
                .subject,
            "B"
        );

        let continuing_index = selection_index_for_period_change(&model, &data, 0, 0, 1, 1);
        assert_eq!(continuing_index, 1);
        assert_eq!(
            lessons_for_period(&model, &data.timegrid, 0, 1)[continuing_index]
                .lesson
                .subject,
            "M"
        );
    }

    #[test]
    fn hit_test_selects_regular_and_empty_cells() {
        let data = sample_data(vec![lesson("math", "M", "A1", "T", "08:00", "08:50")]);
        let model = build_render_model(&data, 2);
        let geometry = timetable_grid_geometry(120, 24, data.timegrid.len(), 0);
        let first_period_row = geometry.grid_y + DAY_HEADER_ROWS;

        let selected = hit_test_timetable_click(
            &data,
            &model,
            120,
            24,
            0,
            geometry.time_width + 1,
            first_period_row,
        )
        .unwrap();
        assert_eq!(selected.day_idx, 0);
        assert_eq!(selected.period_idx, 0);
        assert_eq!(selected.lesson_idx, 0);

        let empty = hit_test_timetable_click(
            &data,
            &model,
            120,
            24,
            0,
            geometry.time_width + geometry.day_width + 1,
            first_period_row,
        )
        .unwrap();
        assert_eq!(empty.day_idx, 1);
        assert_eq!(empty.period_idx, 0);
        assert_eq!(empty.lesson_idx, 0);
    }

    #[test]
    fn hit_test_selects_split_lanes_and_preview_slots() {
        let data = sample_data(vec![
            lesson("math", "M", "A1", "T", "08:00", "09:40"),
            lesson("eng", "E", "A2", "S", "08:00", "08:50"),
            lesson("bio", "B", "A3", "R", "08:50", "09:40"),
        ]);
        let model = build_render_model(&data, 2);
        let wide = timetable_grid_geometry(120, 24, data.timegrid.len(), 0);
        let cell_width = wide.day_width - 1;
        let left_lane_width = cell_width.saturating_sub(1) / 2;
        let first_period_row = wide.grid_y + DAY_HEADER_ROWS;
        let day0_x = wide.time_width + 1;

        let left =
            hit_test_timetable_click(&data, &model, 120, 24, 0, day0_x, first_period_row).unwrap();
        assert_eq!(
            lessons_for_period(&model, &data.timegrid, 0, 0)[left.lesson_idx]
                .lesson
                .subject,
            "E"
        );

        let right = hit_test_timetable_click(
            &data,
            &model,
            120,
            24,
            0,
            day0_x + left_lane_width + 1,
            first_period_row,
        )
        .unwrap();
        assert_eq!(
            lessons_for_period(&model, &data.timegrid, 0, 0)[right.lesson_idx]
                .lesson
                .subject,
            "M"
        );

        let preview = hit_test_timetable_click(
            &data,
            &model,
            80,
            24,
            0,
            timetable_grid_geometry(80, 24, data.timegrid.len(), 0).time_width + 1,
            timetable_grid_geometry(80, 24, data.timegrid.len(), 0).grid_y + DAY_HEADER_ROWS,
        )
        .unwrap();
        assert_eq!(
            lessons_for_period(&model, &data.timegrid, 0, 0)[preview.lesson_idx]
                .lesson
                .subject,
            "E"
        );
    }

    #[test]
    fn hit_test_ignores_headers_hints_time_column_and_details() {
        let data = sample_data_with_period_count(
            vec![lesson("math", "M", "A1", "T", "08:00", "08:50")],
            8,
        );
        let model = build_render_model(&data, 2);
        let geometry = timetable_grid_geometry(120, 24, data.timegrid.len(), 2);
        let first_period_row = geometry.grid_y + DAY_HEADER_ROWS + 1;

        assert!(
            hit_test_timetable_click(
                &data,
                &model,
                120,
                24,
                2,
                geometry.time_width + 1,
                geometry.grid_y
            )
            .is_none()
        );
        assert!(
            hit_test_timetable_click(
                &data,
                &model,
                120,
                24,
                2,
                geometry.time_width + 1,
                geometry.grid_y + 1
            )
            .is_none()
        );
        assert!(
            hit_test_timetable_click(
                &data,
                &model,
                120,
                24,
                2,
                geometry.time_width + 1,
                geometry.grid_y + DAY_HEADER_ROWS
            )
            .is_none()
        );
        assert!(
            hit_test_timetable_click(
                &data,
                &model,
                120,
                24,
                2,
                geometry.time_width - 1,
                first_period_row
            )
            .is_none()
        );
        assert!(
            hit_test_timetable_click(
                &data,
                &model,
                120,
                24,
                2,
                geometry.time_width + 1,
                geometry.grid_y + geometry.grid_height,
            )
            .is_none()
        );
    }

    #[test]
    fn hit_test_maps_scrolled_rows_to_underlying_periods() {
        let data = sample_data_with_period_count(
            vec![lesson("math", "M", "A1", "T", "11:20", "12:10")],
            8,
        );
        let model = build_render_model(&data, 2);
        let geometry = timetable_grid_geometry(120, 24, data.timegrid.len(), 2);
        let third_visible_row = geometry.grid_y + DAY_HEADER_ROWS + 1 + (2 * GRID_ROW_HEIGHT);

        let target = hit_test_timetable_click(
            &data,
            &model,
            120,
            24,
            2,
            geometry.time_width + 1,
            third_visible_row,
        )
        .unwrap();
        assert_eq!(target.period_idx, 4);
    }
}

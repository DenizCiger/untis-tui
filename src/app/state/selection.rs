use super::AppState;
use crate::timetable_model::{
    cycle_visible_lesson_index, find_edge_lesson_period_index, find_next_lesson_period_index,
    lessons_for_period, selection_index_for_period_change,
};

impl AppState {
    pub(super) fn align_timetable_selection_to_period(&mut self, next_period_idx: usize) {
        let Some(data) = self.main.timetable.data.as_ref() else {
            self.main.timetable.selected_period_idx = next_period_idx;
            self.main.timetable.selected_lesson_idx = 0;
            self.sync_timetable_scroll();
            return;
        };
        let Some(model) = self.timetable_render_model() else {
            self.main.timetable.selected_period_idx = next_period_idx;
            self.main.timetable.selected_lesson_idx = 0;
            self.sync_timetable_scroll();
            return;
        };
        self.main.timetable.selected_lesson_idx = selection_index_for_period_change(
            &model,
            data,
            self.main.timetable.selected_day_idx,
            self.main.timetable.selected_period_idx,
            next_period_idx,
            self.main.timetable.selected_lesson_idx,
        );
        self.main.timetable.selected_period_idx = next_period_idx;
        self.ensure_timetable_selection_bounds();
        self.sync_timetable_scroll();
    }

    pub(super) fn current_timetable_period_lessons(&self) -> Vec<crate::models::ParsedLesson> {
        let Some(data) = self.main.timetable.data.as_ref() else {
            return Vec::new();
        };
        let Some(model) = self.timetable_render_model() else {
            return Vec::new();
        };
        lessons_for_period(
            &model,
            &data.timegrid,
            self.main.timetable.selected_day_idx,
            self.main.timetable.selected_period_idx,
        )
        .iter()
        .map(|entry| entry.lesson.clone())
        .collect()
    }

    pub(super) fn ensure_timetable_selection_bounds(&mut self) {
        let count = self.current_timetable_period_lessons().len();
        self.main.timetable.selected_lesson_idx = self
            .main
            .timetable
            .selected_lesson_idx
            .min(count.saturating_sub(1));
    }

    pub(super) fn move_timetable_selection(&mut self, delta: isize, jump_to_lesson: bool) {
        let Some(data) = self.main.timetable.data.as_ref() else {
            self.sync_timetable_scroll();
            return;
        };
        let Some(model) = self.timetable_render_model() else {
            self.sync_timetable_scroll();
            return;
        };
        let max_period = data.timegrid.len().saturating_sub(1);
        let mut next = self.main.timetable.selected_period_idx as isize + delta;
        next = next.clamp(0, max_period as isize);
        let direction = if delta.is_negative() { -1 } else { 1 };

        if jump_to_lesson {
            next = find_next_lesson_period_index(
                &model,
                data,
                self.main.timetable.selected_day_idx,
                self.main.timetable.selected_period_idx,
                direction,
            )
            .unwrap_or(next as usize) as isize;
        }

        self.align_timetable_selection_to_period(next as usize);
    }

    pub(super) fn find_timetable_edge_period(&self, from_start: bool) -> usize {
        let Some(data) = &self.main.timetable.data else {
            return 0;
        };
        let Some(model) = self.timetable_render_model() else {
            return 0;
        };
        find_edge_lesson_period_index(
            &model,
            data,
            self.main.timetable.selected_day_idx,
            from_start,
        )
    }

    pub fn timetable_search_results(&self) -> Vec<crate::models::TimetableSearchItem> {
        crate::webuntis::search_timetable_targets(
            &self.main.timetable.search_index,
            &self.main.timetable.search_input.value,
            None,
        )
    }

    pub fn current_timetable_lessons(&self) -> Vec<crate::models::ParsedLesson> {
        self.current_timetable_period_lessons()
    }

    pub fn timetable_lessons_for(
        &self,
        day_idx: usize,
        period_idx: usize,
    ) -> Vec<crate::models::ParsedLesson> {
        let Some(data) = self.main.timetable.data.as_ref() else {
            return Vec::new();
        };
        let Some(model) = self.timetable_render_model() else {
            return Vec::new();
        };
        lessons_for_period(&model, &data.timegrid, day_idx, period_idx)
            .iter()
            .map(|entry| entry.lesson.clone())
            .collect()
    }

    pub fn selected_timetable_lesson(&self) -> Option<crate::models::ParsedLesson> {
        self.current_timetable_period_lessons()
            .get(self.main.timetable.selected_lesson_idx)
            .cloned()
    }

    pub(super) fn cycle_timetable_overlap(&mut self) {
        let Some(data) = self.main.timetable.data.as_ref() else {
            return;
        };
        let Some(model) = self.timetable_render_model() else {
            return;
        };
        let lessons = lessons_for_period(
            &model,
            &data.timegrid,
            self.main.timetable.selected_day_idx,
            self.main.timetable.selected_period_idx,
        );
        if lessons.len() > 1 {
            self.main.timetable.selected_lesson_idx = cycle_visible_lesson_index(
                &model,
                data,
                self.main.timetable.selected_day_idx,
                self.main.timetable.selected_period_idx,
                self.main.timetable.selected_lesson_idx,
            );
        }
    }
}

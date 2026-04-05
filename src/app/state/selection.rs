use super::{AppState, lessons_for_period};

impl AppState {
    pub(super) fn current_timetable_period_lessons(&self) -> Vec<crate::models::ParsedLesson> {
        lessons_for_period(
            self.main.timetable.data.as_ref(),
            self.main.timetable.selected_day_idx,
            self.main.timetable.selected_period_idx,
        )
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
        let max_period = self
            .main
            .timetable
            .data
            .as_ref()
            .map(|data| data.timegrid.len().saturating_sub(1))
            .unwrap_or(0);
        let mut next = self.main.timetable.selected_period_idx as isize + delta;
        next = next.clamp(0, max_period as isize);
        let direction = if delta.is_negative() { -1 } else { 1 };

        if jump_to_lesson {
            while next >= 0 && next <= max_period as isize {
                if !lessons_for_period(
                    self.main.timetable.data.as_ref(),
                    self.main.timetable.selected_day_idx,
                    next as usize,
                )
                .is_empty()
                {
                    break;
                }
                next += direction;
            }
            next = next.clamp(0, max_period as isize);
        }

        self.main.timetable.selected_period_idx = next as usize;
        self.ensure_timetable_selection_bounds();
    }

    pub(super) fn find_timetable_edge_period(&self, from_start: bool) -> usize {
        let Some(data) = &self.main.timetable.data else {
            return 0;
        };
        if from_start {
            for index in 0..data.timegrid.len() {
                if !lessons_for_period(Some(data), self.main.timetable.selected_day_idx, index)
                    .is_empty()
                {
                    return index;
                }
            }
            0
        } else {
            for index in (0..data.timegrid.len()).rev() {
                if !lessons_for_period(Some(data), self.main.timetable.selected_day_idx, index)
                    .is_empty()
                {
                    return index;
                }
            }
            data.timegrid.len().saturating_sub(1)
        }
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
        lessons_for_period(self.main.timetable.data.as_ref(), day_idx, period_idx)
    }

    pub fn selected_timetable_lesson(&self) -> Option<crate::models::ParsedLesson> {
        self.current_timetable_period_lessons()
            .get(self.main.timetable.selected_lesson_idx)
            .cloned()
    }
}

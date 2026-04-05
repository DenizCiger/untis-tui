use super::{AppState, StatusFilter, WindowFilter};
use chrono::NaiveDate;

impl AppState {
    pub fn filtered_absences(&self) -> Vec<crate::models::ParsedAbsence> {
        let cutoff = self.main.absences.window_filter.cutoff_date();
        let query = self.main.absences.search_query.trim().to_lowercase();

        self.main
            .absences
            .absences
            .iter()
            .filter(|absence| match self.main.absences.status_filter {
                StatusFilter::All => true,
                StatusFilter::Excused => absence.is_excused,
                StatusFilter::Unexcused => !absence.is_excused,
            })
            .filter(|absence| cutoff.map(|date| absence.end_date >= date).unwrap_or(true))
            .filter(|absence| {
                if query.is_empty() {
                    return true;
                }
                format!(
                    "{} {} {} {} {} {}",
                    absence.student_name,
                    absence.reason,
                    absence.text,
                    absence.excuse_status,
                    crate::models::format_date(absence.start_date),
                    crate::models::format_date(absence.end_date)
                )
                .to_lowercase()
                .contains(&query)
            })
            .cloned()
            .collect()
    }

    pub fn has_active_absence_filters(&self) -> bool {
        self.main.absences.status_filter != StatusFilter::All
            || self.main.absences.window_filter != WindowFilter::All
            || !self.main.absences.search_query.trim().is_empty()
    }

    pub fn selected_absence(&self) -> Option<crate::models::ParsedAbsence> {
        self.filtered_absences()
            .get(self.main.absences.selected_idx)
            .cloned()
    }

    pub fn visible_absences(&self, rows: usize) -> (usize, Vec<crate::models::ParsedAbsence>) {
        let filtered = self.filtered_absences();
        let visible_start = std::cmp::min(
            self.main
                .absences
                .selected_idx
                .saturating_sub(rows.saturating_div(2)),
            filtered.len().saturating_sub(rows),
        );
        (
            visible_start,
            filtered
                .into_iter()
                .skip(visible_start)
                .take(rows)
                .collect(),
        )
    }
}

impl WindowFilter {
    pub fn cutoff_date(self) -> Option<NaiveDate> {
        let days = match self {
            WindowFilter::All => return None,
            WindowFilter::D30 => 30,
            WindowFilter::D90 => 90,
            WindowFilter::D180 => 180,
            WindowFilter::D365 => 365,
        };
        Some(crate::models::add_days(crate::models::today_local(), -(days as i64) + 1))
    }

    pub fn label(self) -> &'static str {
        match self {
            WindowFilter::All => "All time",
            WindowFilter::D30 => "30 days",
            WindowFilter::D90 => "90 days",
            WindowFilter::D180 => "180 days",
            WindowFilter::D365 => "365 days",
        }
    }
}

impl StatusFilter {
    pub fn label(self) -> &'static str {
        match self {
            StatusFilter::All => "All",
            StatusFilter::Excused => "Excused",
            StatusFilter::Unexcused => "Unexcused",
        }
    }
}

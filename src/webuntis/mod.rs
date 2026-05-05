mod absences;
mod api;
mod auth;
mod client;
mod search;
#[cfg(test)]
mod tests;
mod timetable;

pub use client::{WebUntisClient, WebUntisError};
pub use search::{
    SearchHighlight, format_timetable_search_type_label, highlight_indices_for_query,
    search_timetable_targets,
};

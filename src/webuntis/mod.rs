mod absences;
mod api;
mod auth;
mod client;
mod search;
mod timetable;
#[cfg(test)]
mod tests;

pub use client::{WebUntisClient, WebUntisError};
pub use search::{format_timetable_search_type_label, search_timetable_targets};

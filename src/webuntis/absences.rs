use super::auth::UntisSession;
use super::client::{WebUntisClient, WebUntisError};
use crate::models::{Config, ParsedAbsence, format_untis_date, format_untis_time, parse_untis_date};
use chrono::NaiveDate;
use reqwest::header::COOKIE;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct AbsencesPayload {
    #[serde(default)]
    pub(super) absences: Vec<RawAbsence>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AbsencesResponse {
    Wrapped { data: AbsencesPayload },
    Flat(AbsencesPayload),
}

impl AbsencesResponse {
    fn into_payload(self) -> AbsencesPayload {
        match self {
            Self::Wrapped { data } => data,
            Self::Flat(data) => data,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawAbsence {
    pub(super) id: i64,
    pub(super) start_date: i32,
    pub(super) end_date: i32,
    pub(super) start_time: i32,
    pub(super) end_time: i32,
    #[serde(default)]
    pub(super) student_name: String,
    #[serde(default)]
    pub(super) reason: String,
    #[serde(default)]
    pub(super) text: String,
    #[serde(default)]
    pub(super) excuse_status: String,
    #[serde(default)]
    pub(super) is_excused: bool,
}

impl WebUntisClient {
    pub(super) async fn get_absences(
        &self,
        session: &UntisSession,
        range_start: NaiveDate,
        range_end: NaiveDate,
    ) -> Result<AbsencesPayload, WebUntisError> {
        let response = self
            .client
            .get(self.url("/WebUntis/api/classreg/absences/students"))
            .header(COOKIE, self.cookie_header(session))
            .query(&[
                ("startDate", format_untis_date(range_start)),
                ("endDate", format_untis_date(range_end)),
                ("studentId", session.person_id.to_string()),
                ("excuseStatusId", "-1".to_owned()),
            ])
            .send()
            .await?;
        response
            .json::<AbsencesResponse>()
            .await
            .map(AbsencesResponse::into_payload)
            .map_err(Into::into)
    }
}

pub(super) fn map_absence_payload(config: &Config, payload: AbsencesPayload) -> Vec<ParsedAbsence> {
    let mut absences = payload
        .absences
        .into_iter()
        .filter_map(|absence| {
            Some(ParsedAbsence {
                id: absence.id,
                student_name: if absence.student_name.is_empty() {
                    config.username.clone()
                } else {
                    absence.student_name
                },
                reason: absence.reason,
                text: absence.text,
                excuse_status: absence.excuse_status,
                is_excused: absence.is_excused,
                start_date: parse_untis_date(absence.start_date)?,
                end_date: parse_untis_date(absence.end_date)?,
                start_time: format_untis_time(absence.start_time),
                end_time: format_untis_time(absence.end_time),
            })
        })
        .collect::<Vec<_>>();
    absences.sort_by(crate::models::compare_absence_newest_first);
    absences
}

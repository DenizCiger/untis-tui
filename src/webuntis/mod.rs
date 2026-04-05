use crate::models::{
    Config, ParsedAbsence, ParsedLesson, TimeUnit, TimetableElementType, TimetableRequestTarget,
    TimetableSearchItem, TimetableSearchTargetType, TimetableTarget, WeekTimetable, add_days,
    format_untis_date, format_untis_time, format_web_date, get_monday, get_weekday_name,
    parse_untis_date, resolve_timetable_request,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::NaiveDate;
use reqwest::header::{CACHE_CONTROL, COOKIE, HeaderMap, HeaderValue, PRAGMA, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

const APP_IDENTITY: &str = "tui-untis";

#[derive(Debug, thiserror::Error)]
pub enum WebUntisError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct WebUntisClient {
    client: reqwest::Client,
    config: Config,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UntisSession {
    session_id: String,
    person_id: i64,
    person_type: i64,
}

#[derive(Debug, Deserialize)]
struct RpcEnvelope<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSchoolYear {
    id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTimeGridDay {
    time_units: Vec<RawTimeUnit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTimeUnit {
    name: String,
    start_time: i32,
    end_time: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTeacher {
    id: i64,
    name: String,
    #[serde(default)]
    fore_name: String,
    #[serde(default)]
    long_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRoom {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    long_name: String,
    #[serde(default)]
    alternate_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawClass {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    long_name: String,
}

#[derive(Debug, Deserialize)]
struct WeeklyResponse {
    data: WeeklyInnerResponse,
}

#[derive(Debug, Deserialize)]
struct WeeklyInnerResponse {
    result: Option<WeeklyResult>,
    error: Option<WeeklyApiError>,
}

#[derive(Debug, Deserialize)]
struct WeeklyApiError {
    data: Option<WeeklyApiErrorData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeeklyApiErrorData {
    message_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WeeklyResult {
    data: WeeklyPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WeeklyPayload {
    element_periods: HashMap<String, Vec<RawWeeklyLesson>>,
    #[serde(default)]
    elements: Vec<RawDirectoryElement>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDirectoryElement {
    #[serde(rename = "type")]
    element_type: i64,
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    long_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawElementRef {
    #[serde(rename = "type")]
    element_type: i64,
    id: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLessonFlags {
    substitution: Option<bool>,
    room_substitution: Option<bool>,
    #[serde(alias = "roomSubstition", alias = "roomsubstition")]
    room_substition: Option<bool>,
    standard: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWeeklyLesson {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    lesson_id: i64,
    date: i32,
    start_time: i32,
    end_time: i32,
    #[serde(default)]
    lesson_code: String,
    #[serde(default)]
    lesson_text: String,
    #[serde(default, rename = "periodInfo", alias = "info")]
    period_info: String,
    #[serde(default, rename = "substText")]
    subst_text: String,
    #[serde(default)]
    cell_state: String,
    #[serde(default)]
    student_group: String,
    #[serde(default)]
    is: RawLessonFlags,
    #[serde(default)]
    elements: Vec<RawElementRef>,
}

#[derive(Debug, Deserialize)]
struct AbsencesPayload {
    #[serde(default)]
    absences: Vec<RawAbsence>,
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
struct RawAbsence {
    id: i64,
    start_date: i32,
    end_date: i32,
    start_time: i32,
    end_time: i32,
    #[serde(default)]
    student_name: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    excuse_status: String,
    #[serde(default)]
    is_excused: bool,
}

impl WebUntisClient {
    pub fn new(config: &Config) -> Result<Self, WebUntisError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/61.0.3163.79 Safari/537.36",
            ),
        );
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    pub async fn test_credentials(config: &Config) -> Result<bool, WebUntisError> {
        let client = Self::new(config)?;
        let session = client.login().await?;
        let _ = client.logout(&session).await;
        Ok(true)
    }

    pub async fn fetch_timetable_search_index(
        config: &Config,
    ) -> Result<Vec<TimetableSearchItem>, WebUntisError> {
        let client = Self::new(config)?;
        let session = client.login().await?;
        let result = async {
            let schoolyear = client.get_current_schoolyear(&session).await.ok();
            let teachers = client.get_teachers(&session).await?;
            let rooms = client.get_rooms(&session).await?;
            let classes = match client
                .get_classes(&session, schoolyear.as_ref().map(|value| value.id))
                .await
            {
                Ok(classes) => classes,
                Err(_) => client.get_classes(&session, None).await?,
            };
            Ok(normalize_search_items(
                map_classes_to_search_items(&classes)
                    .into_iter()
                    .chain(map_rooms_to_search_items(&rooms))
                    .chain(map_teachers_to_search_items(&teachers))
                    .collect(),
            ))
        }
        .await;
        let _ = client.logout(&session).await;
        result
    }

    pub async fn fetch_week_timetable(
        config: &Config,
        week_date: NaiveDate,
        target: &TimetableTarget,
    ) -> Result<WeekTimetable, WebUntisError> {
        let client = Self::new(config)?;
        let session = client.login().await?;
        let result = async {
            let request = resolve_timetable_request(target);
            let (element_id, element_type) = match request {
                TimetableRequestTarget::Own => (session.person_id, session.person_type),
                TimetableRequestTarget::Target { id, element_type } => (id, element_type as i64),
            };
            let weekly = client
                .get_weekly_timetable(&session, week_date, element_id, element_type)
                .await?;
            let timegrid = client.get_timegrid(&session).await?;
            let teachers = client.get_teachers(&session).await?;
            Ok(build_week_timetable(
                week_date, element_id, weekly, timegrid, &teachers,
            ))
        }
        .await;
        let _ = client.logout(&session).await;
        result
    }

    pub async fn fetch_absences_for_range(
        config: &Config,
        range_start: NaiveDate,
        range_end: NaiveDate,
    ) -> Result<Vec<ParsedAbsence>, WebUntisError> {
        let client = Self::new(config)?;
        let session = client.login().await?;
        let result = async {
            let payload = client
                .get_absences(&session, range_start, range_end)
                .await?;
            Ok(map_absence_payload(config, payload))
        }
        .await;
        let _ = client.logout(&session).await;
        result
    }

    async fn login(&self) -> Result<UntisSession, WebUntisError> {
        let response = self
            .client
            .post(self.url("/WebUntis/jsonrpc.do"))
            .query(&[("school", self.config.school.as_str())])
            .json(&serde_json::json!({
                "id": APP_IDENTITY,
                "method": "authenticate",
                "params": {
                    "user": self.config.username,
                    "password": self.config.password,
                    "client": APP_IDENTITY,
                },
                "jsonrpc": "2.0",
            }))
            .send()
            .await?;

        let envelope = response.json::<RpcEnvelope<UntisSession>>().await?;
        if let Some(result) = envelope.result {
            if result.session_id.is_empty() {
                return Err(WebUntisError::Message(
                    "Failed to login. No session id.".to_owned(),
                ));
            }
            return Ok(result);
        }

        Err(WebUntisError::Message(
            envelope
                .error
                .and_then(|error| error.message)
                .unwrap_or_else(|| "Failed to login.".to_owned()),
        ))
    }

    async fn logout(&self, session: &UntisSession) -> Result<(), WebUntisError> {
        let _ = self
            .client
            .post(self.url("/WebUntis/jsonrpc.do"))
            .query(&[("school", self.config.school.as_str())])
            .header(COOKIE, self.cookie_header(session))
            .json(&serde_json::json!({
                "id": APP_IDENTITY,
                "method": "logout",
                "params": {},
                "jsonrpc": "2.0",
            }))
            .send()
            .await?;
        Ok(())
    }

    async fn rpc_request<T: DeserializeOwned, P: Serialize>(
        &self,
        session: &UntisSession,
        method: &str,
        params: P,
    ) -> Result<T, WebUntisError> {
        let response = self
            .client
            .post(self.url("/WebUntis/jsonrpc.do"))
            .query(&[("school", self.config.school.as_str())])
            .header(COOKIE, self.cookie_header(session))
            .json(&serde_json::json!({
                "id": APP_IDENTITY,
                "method": method,
                "params": params,
                "jsonrpc": "2.0",
            }))
            .send()
            .await?;

        let envelope = response.json::<RpcEnvelope<T>>().await?;
        if let Some(result) = envelope.result {
            return Ok(result);
        }

        Err(WebUntisError::Message(
            envelope
                .error
                .and_then(|error| error.message)
                .unwrap_or_else(|| format!("Server didn't return any result for {method}")),
        ))
    }

    fn cookie_header(&self, session: &UntisSession) -> String {
        let school_cookie = format!("_{}", BASE64_STANDARD.encode(self.config.school.as_bytes()));
        format!(
            "JSESSIONID={}; schoolname={school_cookie}",
            session.session_id
        )
    }

    fn url(&self, path: &str) -> String {
        format!("https://{}{}", self.config.server, path)
    }

    async fn get_current_schoolyear(
        &self,
        session: &UntisSession,
    ) -> Result<RawSchoolYear, WebUntisError> {
        self.rpc_request(session, "getCurrentSchoolyear", serde_json::json!({}))
            .await
    }

    async fn get_teachers(&self, session: &UntisSession) -> Result<Vec<RawTeacher>, WebUntisError> {
        self.rpc_request(session, "getTeachers", serde_json::json!({}))
            .await
    }

    async fn get_rooms(&self, session: &UntisSession) -> Result<Vec<RawRoom>, WebUntisError> {
        self.rpc_request(session, "getRooms", serde_json::json!({}))
            .await
    }

    async fn get_classes(
        &self,
        session: &UntisSession,
        schoolyear_id: Option<i64>,
    ) -> Result<Vec<RawClass>, WebUntisError> {
        let params = match schoolyear_id {
            Some(id) => serde_json::json!({ "schoolyearId": id }),
            None => serde_json::json!({}),
        };
        self.rpc_request(session, "getKlassen", params).await
    }

    async fn get_timegrid(
        &self,
        session: &UntisSession,
    ) -> Result<Vec<RawTimeGridDay>, WebUntisError> {
        self.rpc_request(session, "getTimegridUnits", serde_json::json!({}))
            .await
    }

    async fn get_weekly_timetable(
        &self,
        session: &UntisSession,
        date: NaiveDate,
        element_id: i64,
        element_type: i64,
    ) -> Result<WeeklyPayload, WebUntisError> {
        let response = self
            .client
            .get(self.url("/WebUntis/api/public/timetable/weekly/data"))
            .header(COOKIE, self.cookie_header(session))
            .query(&[
                ("elementType", element_type.to_string()),
                ("elementId", element_id.to_string()),
                ("date", format_web_date(date)),
                ("formatId", "1".to_owned()),
            ])
            .send()
            .await?;
        let payload = response.json::<WeeklyResponse>().await?;
        if let Some(error) = payload.data.error {
            return Err(WebUntisError::Message(
                error
                    .data
                    .and_then(|data| data.message_key)
                    .unwrap_or_else(|| "Server responded with error".to_owned()),
            ));
        }
        payload
            .data
            .result
            .map(|result| result.data)
            .ok_or_else(|| WebUntisError::Message("Invalid weekly timetable response".to_owned()))
    }

    async fn get_absences(
        &self,
        session: &UntisSession,
        range_start: NaiveDate,
        range_end: NaiveDate,
    ) -> Result<AbsencesPayload, WebUntisError> {
        // Match the Bun app's getAbsentLesson flow, which uses this classreg endpoint.
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

fn map_absence_payload(config: &Config, payload: AbsencesPayload) -> Vec<ParsedAbsence> {
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

pub fn format_timetable_search_type_label(target_type: TimetableSearchTargetType) -> &'static str {
    match target_type {
        TimetableSearchTargetType::Class => "Class",
        TimetableSearchTargetType::Room => "Room",
        TimetableSearchTargetType::Teacher => "Teacher",
    }
}

pub fn search_timetable_targets(
    items: &[TimetableSearchItem],
    query: &str,
    limit: Option<usize>,
) -> Vec<TimetableSearchItem> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        let mut all = items.to_vec();
        all.sort_by(compare_search_items);
        if let Some(limit) = limit {
            all.truncate(limit);
        }
        return all;
    }

    let tokens = tokenize(&normalized_query);
    let compact_query = normalized_query.replace(' ', "");
    let mut ranked = items
        .iter()
        .filter_map(|item| {
            let prepared = prepare_search_item(item);
            get_match_rank(&prepared, &normalized_query, &tokens, &compact_query)
                .map(|score| (item.clone(), score))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        left.1
            .rank
            .cmp(&right.1.rank)
            .then_with(|| left.1.penalty.cmp(&right.1.penalty))
            .then_with(|| compare_search_items(&left.0, &right.0))
    });
    let mut results = ranked.into_iter().map(|entry| entry.0).collect::<Vec<_>>();
    if let Some(limit) = limit {
        results.truncate(limit);
    }
    results
}

fn build_week_timetable(
    week_date: NaiveDate,
    element_id: i64,
    weekly: WeeklyPayload,
    timegrid: Vec<RawTimeGridDay>,
    teachers: &[RawTeacher],
) -> WeekTimetable {
    let directories = weekly
        .elements
        .into_iter()
        .map(|element| ((element.element_type, element.id), element))
        .collect::<HashMap<_, _>>();
    let teacher_names = build_teacher_full_name_map(teachers);
    let monday = get_monday(week_date);
    let first_grid = timegrid
        .into_iter()
        .find(|grid| !grid.time_units.is_empty())
        .unwrap_or(RawTimeGridDay {
            time_units: Vec::new(),
        });
    let mapped_timegrid = first_grid
        .time_units
        .into_iter()
        .map(|unit| TimeUnit {
            name: unit.name,
            start_time: format_untis_time(unit.start_time),
            end_time: format_untis_time(unit.end_time),
        })
        .collect::<Vec<_>>();

    let mut by_date = HashMap::<i32, Vec<RawWeeklyLesson>>::new();
    let element_entries = weekly
        .element_periods
        .get(&element_id.to_string())
        .cloned()
        .unwrap_or_default();
    for entry in element_entries {
        by_date.entry(entry.date).or_default().push(entry);
    }

    let mut days = Vec::new();
    for offset in 0..5 {
        let day_date = add_days(monday, i64::from(offset));
        let date_num = format_untis_date(day_date)
            .parse::<i32>()
            .unwrap_or_default();
        let mut entries = by_date.remove(&date_num).unwrap_or_default();
        dedupe_day_entries(&mut entries);
        entries.sort_by(compare_lessons_for_display);
        days.push(crate::models::DayTimetable {
            date: day_date,
            day_name: get_weekday_name(day_date),
            lessons: entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    parse_timetable_entry(entry, index, date_num, &directories, &teacher_names)
                })
                .collect(),
        });
    }

    WeekTimetable {
        days,
        timegrid: mapped_timegrid,
    }
}

fn build_teacher_full_name_map(teachers: &[RawTeacher]) -> HashMap<i64, String> {
    let mut result = HashMap::new();
    for teacher in teachers {
        let combined = format!("{} {}", teacher.long_name.trim(), teacher.fore_name.trim())
            .trim()
            .to_owned();
        let display = if !combined.is_empty() {
            combined
        } else if !teacher.long_name.trim().is_empty() {
            teacher.long_name.trim().to_owned()
        } else {
            teacher.name.trim().to_owned()
        };
        if !display.is_empty() {
            result.insert(teacher.id, display);
        }
    }
    result
}

fn compare_lessons_for_display(left: &RawWeeklyLesson, right: &RawWeeklyLesson) -> Ordering {
    left.start_time
        .cmp(&right.start_time)
        .then_with(|| left.end_time.cmp(&right.end_time))
        .then_with(|| left.lesson_text.cmp(&right.lesson_text))
        .then_with(|| left.id.cmp(&right.id))
}

fn dedupe_day_entries(entries: &mut Vec<RawWeeklyLesson>) {
    let mut seen = HashSet::new();
    entries.retain(|entry| seen.insert(build_duplicate_entry_key(entry)));
}

fn build_duplicate_entry_key(entry: &RawWeeklyLesson) -> String {
    let mut classes = Vec::new();
    let mut teachers = Vec::new();
    let mut subjects = Vec::new();
    let mut rooms = Vec::new();
    for element in &entry.elements {
        match element.element_type {
            1 => classes.push(element.id),
            2 => teachers.push(element.id),
            3 => subjects.push(element.id),
            4 => rooms.push(element.id),
            _ => {}
        }
    }
    classes.sort_unstable();
    teachers.sort_unstable();
    subjects.sort_unstable();
    rooms.sort_unstable();

    let flags = [
        entry.is.substitution == Some(true),
        entry.is.room_substitution == Some(true) || entry.is.room_substition == Some(true),
        entry.is.standard == Some(true),
    ];

    format!(
        "{}|{}|{}|{}|{}|{}|{:?}|{:?}|{:?}|{:?}|{}|{:?}",
        entry.date,
        entry.start_time,
        entry.end_time,
        entry.lesson_id,
        entry.lesson_code,
        entry.cell_state,
        classes,
        teachers,
        subjects,
        rooms,
        entry.student_group,
        flags
    )
}

fn parse_timetable_entry(
    entry: &RawWeeklyLesson,
    index_in_day: usize,
    date_num: i32,
    directories: &HashMap<(i64, i64), RawDirectoryElement>,
    teacher_full_name_by_id: &HashMap<i64, String>,
) -> ParsedLesson {
    let resolved = entry
        .elements
        .iter()
        .filter_map(|element| {
            directories
                .get(&(element.element_type, element.id))
                .map(|directory| {
                    (
                        element.element_type,
                        element.id,
                        directory.name.clone(),
                        if directory.long_name.is_empty() {
                            directory.name.clone()
                        } else {
                            directory.long_name.clone()
                        },
                    )
                })
        })
        .collect::<Vec<_>>();

    let subject = resolved
        .iter()
        .find(|element| element.0 == TimetableElementType::Subject as i64)
        .map(|element| element.2.clone())
        .unwrap_or_else(|| "Unknown".to_owned());
    let subject_long_name = resolved
        .iter()
        .find(|element| element.0 == TimetableElementType::Subject as i64)
        .map(|element| element.3.clone())
        .unwrap_or_else(|| subject.clone());
    let teacher = resolved
        .iter()
        .find(|element| element.0 == TimetableElementType::Teacher as i64)
        .map(|element| element.2.clone())
        .unwrap_or_default();
    let teacher_long_name = resolved
        .iter()
        .find(|element| element.0 == TimetableElementType::Teacher as i64)
        .map(|element| {
            teacher_full_name_by_id
                .get(&element.1)
                .cloned()
                .unwrap_or_else(|| element.3.clone())
        })
        .unwrap_or_else(|| teacher.clone());
    let room = resolved
        .iter()
        .find(|element| element.0 == TimetableElementType::Room as i64)
        .map(|element| element.2.clone())
        .unwrap_or_default();
    let room_long_name = resolved
        .iter()
        .find(|element| element.0 == TimetableElementType::Room as i64)
        .map(|element| element.3.clone())
        .unwrap_or_else(|| room.clone());

    let all_teachers = resolved
        .iter()
        .filter(|element| element.0 == TimetableElementType::Teacher as i64)
        .map(|element| element.2.clone())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let all_teacher_long_names = resolved
        .iter()
        .filter(|element| element.0 == TimetableElementType::Teacher as i64)
        .map(|element| {
            teacher_full_name_by_id
                .get(&element.1)
                .cloned()
                .unwrap_or_else(|| element.3.clone())
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let all_classes = resolved
        .iter()
        .filter(|element| element.0 == TimetableElementType::Class as i64)
        .map(|element| element.2.clone())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    let instance_id = if entry.id > 0 {
        entry.id.to_string()
    } else if entry.lesson_id > 0 {
        entry.lesson_id.to_string()
    } else {
        format!(
            "{date_num}-{}-{}-{}-{}-{}-{index_in_day}",
            entry.start_time, entry.end_time, subject, teacher, room
        )
    };

    ParsedLesson {
        instance_id,
        subject,
        subject_long_name,
        lesson_text: entry.lesson_text.clone(),
        cell_state: entry.cell_state.clone(),
        teacher,
        teacher_long_name,
        all_teachers,
        all_teacher_long_names,
        room,
        room_long_name,
        all_classes,
        start_time: format_untis_time(entry.start_time),
        end_time: format_untis_time(entry.end_time),
        cancelled: (entry.is.standard == Some(false) && entry.cell_state == "SUBSTITUTION")
            || entry.lesson_code == "cancelled",
        substitution: entry.is.substitution == Some(true)
            || entry.is.room_substitution == Some(true)
            || entry.is.room_substition == Some(true),
        remarks: if !entry.period_info.is_empty() {
            entry.period_info.clone()
        } else {
            entry.subst_text.clone()
        },
    }
}

fn map_teachers_to_search_items(teachers: &[RawTeacher]) -> Vec<TimetableSearchItem> {
    teachers
        .iter()
        .map(|teacher| {
            let short = teacher.name.trim();
            let surname = teacher.long_name.trim();
            let forename = teacher.fore_name.trim();
            let combined = format!("{surname} {forename}").trim().to_owned();
            let display = if !combined.is_empty() {
                combined.clone()
            } else if !surname.is_empty() {
                surname.to_owned()
            } else if !short.is_empty() {
                short.to_owned()
            } else {
                teacher.id.to_string()
            };
            let secondary = if !short.is_empty() && short != display {
                short.to_owned()
            } else if !surname.is_empty() && surname != display {
                surname.to_owned()
            } else {
                display.clone()
            };
            TimetableSearchItem {
                r#type: TimetableSearchTargetType::Teacher,
                id: teacher.id,
                name: display.clone(),
                long_name: secondary.clone(),
                search_text: build_search_text(&[
                    display,
                    secondary,
                    short.to_owned(),
                    surname.to_owned(),
                    forename.to_owned(),
                ]),
            }
        })
        .collect()
}

fn map_rooms_to_search_items(rooms: &[RawRoom]) -> Vec<TimetableSearchItem> {
    rooms
        .iter()
        .map(|room| TimetableSearchItem {
            r#type: TimetableSearchTargetType::Room,
            id: room.id,
            name: if room.name.is_empty() {
                if room.long_name.is_empty() {
                    room.id.to_string()
                } else {
                    room.long_name.clone()
                }
            } else {
                room.name.clone()
            },
            long_name: if room.long_name.is_empty() {
                if room.name.is_empty() {
                    room.id.to_string()
                } else {
                    room.name.clone()
                }
            } else {
                room.long_name.clone()
            },
            search_text: build_search_text(&[
                room.name.clone(),
                room.long_name.clone(),
                room.alternate_name.clone(),
            ]),
        })
        .collect()
}

fn map_classes_to_search_items(classes: &[RawClass]) -> Vec<TimetableSearchItem> {
    classes
        .iter()
        .map(|class| TimetableSearchItem {
            r#type: TimetableSearchTargetType::Class,
            id: class.id,
            name: if class.name.is_empty() {
                if class.long_name.is_empty() {
                    class.id.to_string()
                } else {
                    class.long_name.clone()
                }
            } else {
                class.name.clone()
            },
            long_name: if class.long_name.is_empty() {
                if class.name.is_empty() {
                    class.id.to_string()
                } else {
                    class.name.clone()
                }
            } else {
                class.long_name.clone()
            },
            search_text: build_search_text(&[class.name.clone(), class.long_name.clone()]),
        })
        .collect()
}

fn build_search_text(parts: &[String]) -> String {
    parts
        .iter()
        .filter(|part| !part.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalize_search_items(items: Vec<TimetableSearchItem>) -> Vec<TimetableSearchItem> {
    let mut deduped = HashMap::<(TimetableSearchTargetType, i64), TimetableSearchItem>::new();
    for item in items {
        deduped.entry((item.r#type, item.id)).or_insert(item);
    }
    let mut values: Vec<_> = deduped.into_values().collect();
    values.sort_by(compare_search_items);
    values
}

fn normalize(value: &str) -> String {
    value
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .collect::<String>()
        .trim()
        .to_lowercase()
}

fn tokenize(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn to_words(value: &str) -> Vec<String> {
    let mut current = String::new();
    let mut words = Vec::new();
    for character in normalize(value).chars() {
        if character.is_ascii_alphanumeric() {
            current.push(character);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn search_type_order(target_type: TimetableSearchTargetType) -> i32 {
    match target_type {
        TimetableSearchTargetType::Class => 0,
        TimetableSearchTargetType::Room => 1,
        TimetableSearchTargetType::Teacher => 2,
    }
}

fn compare_search_items(left: &TimetableSearchItem, right: &TimetableSearchItem) -> Ordering {
    search_type_order(left.r#type)
        .cmp(&search_type_order(right.r#type))
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.long_name.cmp(&right.long_name))
        .then_with(|| left.id.cmp(&right.id))
}

fn has_all_tokens(haystack: &str, tokens: &[String]) -> bool {
    tokens.iter().all(|token| haystack.contains(token))
}

fn token_contains_penalty(haystack: &str, tokens: &[String]) -> Option<usize> {
    let mut penalty = 0;
    for token in tokens {
        penalty += haystack.find(token)?;
    }
    Some(penalty)
}

fn word_prefix_penalty(words: &[String], tokens: &[String]) -> Option<usize> {
    let mut penalty = 0;
    for token in tokens {
        let mut best_word_idx = None;
        let mut best_length_delta = usize::MAX;
        for (index, word) in words.iter().enumerate() {
            if !word.starts_with(token) {
                continue;
            }
            let length_delta = word.len().saturating_sub(token.len());
            if length_delta < best_length_delta {
                best_word_idx = Some(index);
                best_length_delta = length_delta;
            }
        }
        penalty += best_word_idx? * 8 + best_length_delta;
    }
    Some(penalty)
}

fn fuzzy_subsequence_penalty(haystack: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }
    let mut haystack_index = 0;
    let mut penalty = 0;
    let mut previous = None;
    for character in query.chars() {
        let next = haystack[haystack_index..].find(character)?;
        let absolute = haystack_index + next;
        penalty += previous
            .map(|prev| absolute.saturating_sub(prev + 1))
            .unwrap_or(absolute);
        previous = Some(absolute);
        haystack_index = absolute + character.len_utf8();
    }
    penalty += haystack.len().saturating_sub(haystack_index);
    Some(penalty)
}

#[derive(Debug, Clone)]
struct PreparedSearchItem {
    name: String,
    long_name: String,
    search_text: String,
    compact_search: String,
    name_words: Vec<String>,
    long_name_words: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct MatchRank {
    rank: usize,
    penalty: usize,
}

fn prepare_search_item(item: &TimetableSearchItem) -> PreparedSearchItem {
    let name = normalize(&item.name);
    let long_name = normalize(&item.long_name);
    let search_source = if item.search_text.is_empty() {
        format!("{} {}", item.name, item.long_name)
    } else {
        item.search_text.clone()
    };
    let search_text = normalize(&search_source);
    PreparedSearchItem {
        name: name.clone(),
        long_name: long_name.clone(),
        compact_search: search_text.replace(' ', ""),
        search_text,
        name_words: to_words(&name),
        long_name_words: to_words(&long_name),
    }
}

fn get_match_rank(
    item: &PreparedSearchItem,
    normalized_query: &str,
    tokens: &[String],
    compact_query: &str,
) -> Option<MatchRank> {
    if item.name.starts_with(normalized_query) {
        return Some(MatchRank {
            rank: 0,
            penalty: item.name.len().saturating_sub(normalized_query.len()),
        });
    }
    if item.long_name.starts_with(normalized_query) {
        return Some(MatchRank {
            rank: 1,
            penalty: item.long_name.len().saturating_sub(normalized_query.len()),
        });
    }
    if let Some(penalty) = word_prefix_penalty(&item.name_words, tokens) {
        return Some(MatchRank { rank: 2, penalty });
    }
    if let Some(penalty) = word_prefix_penalty(&item.long_name_words, tokens) {
        return Some(MatchRank { rank: 3, penalty });
    }
    if has_all_tokens(&item.name, tokens) {
        return Some(MatchRank {
            rank: 4,
            penalty: token_contains_penalty(&item.name, tokens).unwrap_or(0),
        });
    }
    if has_all_tokens(&item.long_name, tokens) {
        return Some(MatchRank {
            rank: 5,
            penalty: token_contains_penalty(&item.long_name, tokens).unwrap_or(0),
        });
    }
    if has_all_tokens(&item.search_text, tokens) {
        return Some(MatchRank {
            rank: 6,
            penalty: token_contains_penalty(&item.search_text, tokens).unwrap_or(0),
        });
    }
    fuzzy_subsequence_penalty(&item.compact_search, compact_query)
        .map(|penalty| MatchRank { rank: 7, penalty })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Config, parse_time_to_minutes};

    fn item(
        id: i64,
        target_type: TimetableSearchTargetType,
        name: &str,
        long_name: &str,
        search_text: Option<&str>,
    ) -> TimetableSearchItem {
        TimetableSearchItem {
            r#type: target_type,
            id,
            name: name.to_owned(),
            long_name: long_name.to_owned(),
            search_text: search_text
                .unwrap_or(&format!("{name} {long_name}"))
                .to_lowercase(),
        }
    }

    #[test]
    fn timetable_search_ranking_matches_contains_case_insensitively() {
        let results = search_timetable_targets(
            &[
                item(
                    1,
                    TimetableSearchTargetType::Teacher,
                    "MrMiller",
                    "Miller",
                    None,
                ),
                item(
                    2,
                    TimetableSearchTargetType::Room,
                    "Room A12",
                    "Science Room",
                    None,
                ),
            ],
            "MILL",
            Some(10),
        );
        assert_eq!(
            results.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn timetable_search_ranking_prioritizes_starts_with_over_contains_matches() {
        let results = search_timetable_targets(
            &[
                item(
                    1,
                    TimetableSearchTargetType::Teacher,
                    "Tina",
                    "Teacher Tina",
                    None,
                ),
                item(
                    2,
                    TimetableSearchTargetType::Teacher,
                    "Math",
                    "Advanced Tina Group",
                    None,
                ),
                item(
                    3,
                    TimetableSearchTargetType::Teacher,
                    "Bio",
                    "Tina Biology",
                    None,
                ),
            ],
            "ti",
            Some(10),
        );
        assert_eq!(
            results.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            vec![1, 3, 2]
        );
    }

    #[test]
    fn timetable_search_ranking_keeps_mixed_type_ordering_stable_for_equal_rank() {
        let results = search_timetable_targets(
            &[
                item(
                    2,
                    TimetableSearchTargetType::Teacher,
                    "A-Name",
                    "A-Name",
                    None,
                ),
                item(
                    1,
                    TimetableSearchTargetType::Class,
                    "A-Name",
                    "A-Name",
                    None,
                ),
                item(3, TimetableSearchTargetType::Room, "A-Name", "A-Name", None),
            ],
            "a-",
            Some(10),
        );
        assert_eq!(
            results
                .iter()
                .map(|entry| format!("{:?}:{}", entry.r#type, entry.id).to_lowercase())
                .collect::<Vec<_>>(),
            vec!["class:1", "room:3", "teacher:2"]
        );
    }

    #[test]
    fn timetable_search_ranking_matches_multi_token_queries_across_name_fields() {
        let results = search_timetable_targets(
            &[
                item(
                    1,
                    TimetableSearchTargetType::Teacher,
                    "Max Mustermann",
                    "MMAX",
                    Some("max mustermann mmax"),
                ),
                item(
                    2,
                    TimetableSearchTargetType::Teacher,
                    "Max Muster",
                    "MMUS",
                    Some("max muster mmus"),
                ),
            ],
            "max mmax",
            None,
        );
        assert_eq!(
            results.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn timetable_search_ranking_returns_all_matches_when_no_limit_is_provided() {
        let results = search_timetable_targets(
            &[
                item(1, TimetableSearchTargetType::Teacher, "AA", "AA", None),
                item(2, TimetableSearchTargetType::Teacher, "AB", "AB", None),
                item(3, TimetableSearchTargetType::Teacher, "AC", "AC", None),
            ],
            "a",
            None,
        );
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn repeated_rows_logic_repeats_multi_period_lessons() {
        let lesson = ParsedLesson {
            instance_id: "x".into(),
            subject: "Math".into(),
            subject_long_name: "Mathematics".into(),
            lesson_text: String::new(),
            cell_state: String::new(),
            teacher: "M".into(),
            teacher_long_name: "Mr M".into(),
            all_teachers: vec!["M".into()],
            all_teacher_long_names: vec!["Mr M".into()],
            room: "A1".into(),
            room_long_name: "Room A1".into(),
            all_classes: vec!["1A".into()],
            start_time: "08:00".into(),
            end_time: "09:40".into(),
            cancelled: false,
            substitution: false,
            remarks: String::new(),
        };
        let periods = vec![
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
        ];
        let hits = periods
            .iter()
            .filter(|period| {
                let lesson_start = parse_time_to_minutes(&lesson.start_time);
                let lesson_end = parse_time_to_minutes(&lesson.end_time);
                let period_start = parse_time_to_minutes(&period.start_time);
                let period_end = parse_time_to_minutes(&period.end_time);
                lesson_start < period_end && lesson_end > period_start
            })
            .count();
        assert_eq!(hits, 2);
    }

    #[test]
    fn absence_mapping_uses_bun_compatible_fields_and_sorting() {
        let config = Config {
            school: "school".into(),
            username: "user".into(),
            password: "secret".into(),
            server: "mese.webuntis.com".into(),
        };
        let payload = AbsencesPayload {
            absences: vec![
                RawAbsence {
                    id: 1,
                    start_date: 20260115,
                    end_date: 20260115,
                    start_time: 815,
                    end_time: 900,
                    student_name: String::new(),
                    reason: "Ill".into(),
                    text: String::new(),
                    excuse_status: "Open".into(),
                    is_excused: false,
                },
                RawAbsence {
                    id: 2,
                    start_date: 20260120,
                    end_date: 20260120,
                    start_time: 700,
                    end_time: 745,
                    student_name: "Student".into(),
                    reason: String::new(),
                    text: "Doctor".into(),
                    excuse_status: "Excused".into(),
                    is_excused: true,
                },
            ],
        };

        let mapped = map_absence_payload(&config, payload);

        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0].id, 2);
        assert_eq!(mapped[0].student_name, "Student");
        assert_eq!(mapped[1].id, 1);
        assert_eq!(mapped[1].student_name, "user");
        assert_eq!(mapped[1].start_time, "08:15");
    }
}

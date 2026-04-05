use crate::models::{
    Config, ParsedAbsence, TimetableRequestTarget, TimetableSearchItem, TimetableTarget,
    WeekTimetable, resolve_timetable_request,
};
use chrono::NaiveDate;
use reqwest::header::{CACHE_CONTROL, HeaderMap, HeaderValue, PRAGMA, USER_AGENT};

pub(super) const APP_IDENTITY: &str = "tui-untis";

#[derive(Debug, thiserror::Error)]
pub enum WebUntisError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct WebUntisClient {
    pub(super) client: reqwest::Client,
    pub(super) config: Config,
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
            Ok(crate::webuntis::search::normalize_search_items(
                crate::webuntis::search::map_classes_to_search_items(&classes)
                    .into_iter()
                    .chain(crate::webuntis::search::map_rooms_to_search_items(&rooms))
                    .chain(crate::webuntis::search::map_teachers_to_search_items(
                        &teachers,
                    ))
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
            Ok(crate::webuntis::timetable::build_week_timetable(
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
            Ok(crate::webuntis::absences::map_absence_payload(
                config, payload,
            ))
        }
        .await;
        let _ = client.logout(&session).await;
        result
    }
}

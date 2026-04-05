use super::auth::UntisSession;
use super::client::{WebUntisClient, WebUntisError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use reqwest::header::COOKIE;

#[derive(Debug, Deserialize)]
pub(super) struct RpcEnvelope<T> {
    pub(super) result: Option<T>,
    pub(super) error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RpcError {
    pub(super) message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawSchoolYear {
    pub(super) id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawTimeGridDay {
    pub(super) time_units: Vec<RawTimeUnit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawTimeUnit {
    pub(super) name: String,
    pub(super) start_time: i32,
    pub(super) end_time: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawTeacher {
    pub(super) id: i64,
    pub(super) name: String,
    #[serde(default)]
    pub(super) fore_name: String,
    #[serde(default)]
    pub(super) long_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawRoom {
    pub(super) id: i64,
    #[serde(default)]
    pub(super) name: String,
    #[serde(default)]
    pub(super) long_name: String,
    #[serde(default)]
    pub(super) alternate_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawClass {
    pub(super) id: i64,
    #[serde(default)]
    pub(super) name: String,
    #[serde(default)]
    pub(super) long_name: String,
}

impl WebUntisClient {
    pub(super) async fn rpc_request<T: DeserializeOwned, P: Serialize>(
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
                "id": super::client::APP_IDENTITY,
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

    pub(super) async fn get_current_schoolyear(
        &self,
        session: &UntisSession,
    ) -> Result<RawSchoolYear, WebUntisError> {
        self.rpc_request(session, "getCurrentSchoolyear", serde_json::json!({}))
            .await
    }

    pub(super) async fn get_teachers(
        &self,
        session: &UntisSession,
    ) -> Result<Vec<RawTeacher>, WebUntisError> {
        self.rpc_request(session, "getTeachers", serde_json::json!({}))
            .await
    }

    pub(super) async fn get_rooms(
        &self,
        session: &UntisSession,
    ) -> Result<Vec<RawRoom>, WebUntisError> {
        self.rpc_request(session, "getRooms", serde_json::json!({}))
            .await
    }

    pub(super) async fn get_classes(
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

    pub(super) async fn get_timegrid(
        &self,
        session: &UntisSession,
    ) -> Result<Vec<RawTimeGridDay>, WebUntisError> {
        self.rpc_request(session, "getTimegridUnits", serde_json::json!({}))
            .await
    }
}

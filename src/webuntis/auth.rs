use super::client::{APP_IDENTITY, WebUntisClient, WebUntisError};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use reqwest::header::COOKIE;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UntisSession {
    pub(super) session_id: String,
    pub(super) person_id: i64,
    pub(super) person_type: i64,
}

impl WebUntisClient {
    pub(super) async fn login(&self) -> Result<UntisSession, WebUntisError> {
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

        let envelope = response
            .json::<super::api::RpcEnvelope<UntisSession>>()
            .await?;
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

    pub(super) async fn logout(&self, session: &UntisSession) -> Result<(), WebUntisError> {
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

    pub(super) fn cookie_header(&self, session: &UntisSession) -> String {
        let school_cookie = format!("_{}", BASE64_STANDARD.encode(self.config.school.as_bytes()));
        format!(
            "JSESSIONID={}; schoolname={school_cookie}",
            session.session_id
        )
    }

    pub(super) fn url(&self, path: &str) -> String {
        format!("https://{}{}", self.config.server, path)
    }
}

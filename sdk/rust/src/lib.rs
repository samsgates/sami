//! Native API client. Mutations are never automatically retried.
use reqwest::{Client, Method, Url};
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid client input: {0}")]
    Invalid(String),
    #[error("SAMI HTTP {status}: {message}")]
    Api {
        status: u16,
        message: String,
        details: Value,
    },
    #[error("transport failed; a mutation may have completed, inspect/reconcile before retrying")]
    Transport(#[source] reqwest::Error),
}

pub struct SamiClient {
    http: Client,
    base: Url,
    token: String,
}

impl SamiClient {
    pub fn new(base: &str, token: impl Into<String>) -> Result<Self, Error> {
        let mut base = Url::parse(base).map_err(|e| Error::Invalid(e.to_string()))?;
        if !matches!(base.scheme(), "http" | "https")
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
        {
            return Err(Error::Invalid(
                "use an HTTP(S) API URL without credentials, query or fragment".into(),
            ));
        }
        if base.scheme() == "http"
            && !matches!(base.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
        {
            return Err(Error::Invalid(
                "use HTTPS for a remote API credential".into(),
            ));
        }
        let path = format!("{}/", base.path().trim_end_matches('/'));
        base.set_path(&path);
        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(Error::Transport)?;
        Ok(Self {
            http,
            base,
            token: token.into(),
        })
    }

    fn endpoint(&self, segments: &[&str]) -> Result<Url, Error> {
        if segments.iter().any(|s| s.is_empty()) {
            return Err(Error::Invalid("empty route identifier".into()));
        }
        let mut url = self.base.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| Error::Invalid("URL cannot be a base".into()))?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        Ok(url)
    }

    async fn request(
        &self,
        method: Method,
        segments: &[&str],
        body: Option<Value>,
        key: Option<&str>,
    ) -> Result<Value, Error> {
        let mut request = self
            .http
            .request(method, self.endpoint(segments)?)
            .header("Accept", "application/json");
        if !self.token.is_empty() {
            request = request.bearer_auth(&self.token);
        }
        if let Some(value) = body {
            request = request.json(&value);
        }
        if let Some(value) = key {
            request = request.header("Idempotency-Key", value);
        }
        let response = request.send().await.map_err(Error::Transport)?;
        let status = response.status();
        let text = response.text().await.map_err(Error::Transport)?;
        let value: Value = serde_json::from_str(&text)
            .unwrap_or_else(|_| json!({"message": text.chars().take(500).collect::<String>()}));
        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
                message: value
                    .pointer("/error/message")
                    .or_else(|| value.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("request rejected")
                    .into(),
                details: value,
            });
        }
        Ok(value)
    }
    pub async fn decide(&self, task_id: &str, input: Value, pack_id: &str) -> Result<Value, Error> {
        self.request(
            Method::POST,
            &["decisions"],
            Some(json!({"task_id":task_id,"input":input,"pack_id":pack_id})),
            None,
        )
        .await
    }
    pub async fn create_session(&self, state: Value) -> Result<Value, Error> {
        self.request(
            Method::POST,
            &["sessions"],
            Some(json!({"state":state})),
            None,
        )
        .await
    }
    pub async fn respond(
        &self,
        session_id: &str,
        message: &str,
        revision: Option<u64>,
    ) -> Result<Value, Error> {
        let mut body = json!({"session_id":session_id,"message":message});
        if let Some(value) = revision {
            body["revision"] = json!(value);
        }
        self.request(Method::POST, &["respond"], Some(body), None)
            .await
    }
    pub async fn list(&self, kind: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["admin", kind], None, None)
            .await
    }
    pub async fn get(&self, kind: &str, id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["admin", kind, id], None, None)
            .await
    }
    pub async fn save(&self, kind: &str, record: Value) -> Result<Value, Error> {
        self.request(Method::POST, &["admin", kind], Some(record), None)
            .await
    }
    pub async fn source_operation(
        &self,
        id: &str,
        operation: &str,
        reason: &str,
    ) -> Result<Value, Error> {
        if !matches!(operation, "publish" | "retract") {
            return Err(Error::Invalid(
                "source operation must be publish or retract".into(),
            ));
        }
        self.request(
            Method::POST,
            &["sources", id, operation],
            Some(json!({"reason":reason})),
            None,
        )
        .await
    }
    pub async fn delete_source(&self, id: &str) -> Result<Value, Error> {
        self.request(Method::DELETE, &["sources", id], None, None)
            .await
    }
    pub async fn receipt(&self, id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["receipts", id], None, None)
            .await
    }
    pub async fn replay(&self, id: &str) -> Result<Value, Error> {
        self.request(
            Method::POST,
            &["receipts", id, "replay"],
            Some(json!({})),
            None,
        )
        .await
    }
    pub async fn propose_action(
        &self,
        decision_id: &str,
        tool_id: &str,
        arguments: Value,
    ) -> Result<Value, Error> {
        self.request(
            Method::POST,
            &["actions", "propose"],
            Some(json!({"decision_id":decision_id,"tool_id":tool_id,"arguments":arguments})),
            None,
        )
        .await
    }
    pub async fn approve_action(
        &self,
        id: &str,
        reason: &str,
        expires_in_seconds: u64,
    ) -> Result<Value, Error> {
        self.request(
            Method::POST,
            &["actions", id, "approve"],
            Some(json!({"reason":reason,"expires_in_seconds":expires_in_seconds})),
            None,
        )
        .await
    }
    pub async fn execute_action(&self, id: &str, idempotency_key: &str) -> Result<Value, Error> {
        if idempotency_key.is_empty() {
            return Err(Error::Invalid(
                "execution requires an idempotency key".into(),
            ));
        }
        self.request(
            Method::POST,
            &["actions", id, "execute"],
            Some(json!({})),
            Some(idempotency_key),
        )
        .await
    }
    pub async fn reconcile_action(&self, id: &str) -> Result<Value, Error> {
        self.request(
            Method::POST,
            &["actions", id, "reconcile"],
            Some(json!({})),
            None,
        )
        .await
    }
    pub async fn feedback(&self, event: Value) -> Result<Value, Error> {
        self.request(Method::POST, &["feedback"], Some(event), None)
            .await
    }
    pub async fn research(&self, operation: &str, input: Value) -> Result<Value, Error> {
        self.request(Method::POST, &["research", operation], Some(input), None)
            .await
    }
    pub async fn create_key(&self, input: Value) -> Result<Value, Error> {
        self.request(Method::POST, &["keys"], Some(input), None)
            .await
    }
    pub async fn revoke_key(&self, id: &str) -> Result<Value, Error> {
        self.request(Method::POST, &["keys", id, "revoke"], Some(json!({})), None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_api_base_and_encodes_resource_identifier() {
        let client = SamiClient::new("http://127.0.0.1:8080/v1", "").expect("valid client");
        assert_eq!(
            client
                .endpoint(&["actions", "a/b", "execute"])
                .expect("valid path")
                .as_str(),
            "http://127.0.0.1:8080/v1/actions/a%2Fb/execute"
        );
    }
    #[test]
    fn rejects_credentials_in_base_url() {
        assert!(SamiClient::new("https://secret@example.com/v1", "").is_err());
    }
}

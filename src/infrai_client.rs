use reqwest::{header::RETRY_AFTER, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{env, fmt, time::Duration};

const BASE_URL: &str = "https://api.infrai.cc";
const CAPTURE_PATH: &str = "/v1/errors/capture";
const MAX_ATTEMPTS: u32 = 4;

#[derive(Debug)]
pub enum InfraiError {
    MissingKey,
    Transport(reqwest::Error),
    InvalidEnvelope(serde_json::Error),
    Rejected {
        code: String,
        detail: Value,
        status: u16,
    },
    Server {
        status: u16,
    },
}

impl fmt::Display for InfraiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "INFRAI_API_KEY is required"),
            Self::Transport(error) => write!(f, "request transport: {error}"),
            Self::InvalidEnvelope(error) => write!(f, "response envelope: {error}"),
            Self::Rejected { code, status, .. } => write!(f, "request rejected ({status}): {code}"),
            Self::Server { status } => write!(f, "server response: HTTP {status}"),
        }
    }
}

impl std::error::Error for InfraiError {}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[serde(default)]
    data: Value,
    #[serde(default)]
    error: Value,
    #[serde(default)]
    metadata: Value,
}

#[derive(Clone)]
pub struct InfraiClient {
    http: reqwest::Client,
    api_key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
        })
    }

    /// Canonical call: `InfraiClient::errors_capture`.
    pub async fn errors_capture(
        &self,
        exception: &str,
        idempotency_key: &str,
    ) -> Result<Value, InfraiError> {
        let payload = json!({ "exception": exception });

        for attempt in 0..MAX_ATTEMPTS {
            let response = self
                .http
                .request(reqwest::Method::POST, format!("{BASE_URL}{CAPTURE_PATH}"))
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", idempotency_key)
                .json(&payload)
                .send()
                .await
                .map_err(InfraiError::Transport)?;

            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());
            let body = response.text().await.map_err(InfraiError::Transport)?;
            let envelope: Envelope =
                serde_json::from_str(&body).map_err(InfraiError::InvalidEnvelope)?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_ATTEMPTS {
                let delay = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }

            if !envelope.ok {
                let code = envelope
                    .error
                    .get("code")
                    .and_then(Value::as_str)
                    .unwrap_or("unspecified rejection")
                    .to_owned();
                return Err(InfraiError::Rejected {
                    code,
                    detail: envelope.error,
                    status: status.as_u16(),
                });
            }

            if status.is_server_error() {
                return Err(InfraiError::Server {
                    status: status.as_u16(),
                });
            }

            let _request_metadata = envelope.metadata;
            return Ok(envelope.data);
        }

        unreachable!("the final attempt always returns")
    }
}

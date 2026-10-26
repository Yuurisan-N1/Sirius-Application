use std::time::Duration;

use reqwest::{Client, Method, StatusCode};
use serde_json::Value;

use crate::net::egress::Proxy;
use crate::support::consts::net::{
    ACCEPT_JOSE, APP_AGENT, BASE_URL, CONNECT_TIMEOUT_SECS, ENVELOPE_HEADER, READ_TIMEOUT_SECS,
};
use crate::support::error::{refusal, AppError, AppResult};
use crate::wire::seal::{seal, unseal};

pub struct Link {
    http: Client,
    bearer: String,
}

impl Link {
    pub fn open(proxy: Option<&Proxy>) -> AppResult<Link> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(READ_TIMEOUT_SECS))
            .user_agent(APP_AGENT)
            .gzip(true);
        if let Some(proxy) = proxy {
            let route = reqwest::Proxy::all(proxy.url())
                .map_err(|_| AppError::config("the proxy could not be used"))?;
            builder = builder.proxy(route);
        }
        let http = builder
            .build()
            .map_err(|_| AppError::network("the client could not be built"))?;
        Ok(Link {
            http,
            bearer: String::new(),
        })
    }

    pub fn adopt(&mut self, token: &str) {
        self.bearer = token.to_string();
    }

    pub async fn get(&self, path: &str) -> AppResult<Value> {
        self.call(Method::GET, path, "", None, &[]).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> AppResult<Value> {
        self.call(Method::POST, path, "", Some(body), &[]).await
    }

    pub async fn post_with(
        &self,
        path: &str,
        body: &Value,
        extra: &[(&str, String)],
    ) -> AppResult<Value> {
        let pairs: Vec<(&str, String)> = extra.to_vec();
        self.call(Method::POST, path, "", Some(body), &pairs).await
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        query: &str,
        body: Option<&Value>,
        extra: &[(&str, String)],
    ) -> AppResult<Value> {
        let sealed = seal(method.as_str(), path, query, body)?;
        let url = format!("{}{}", BASE_URL, path);

        let mut request = self
            .http
            .request(method.clone(), &url)
            .header("Accept", ACCEPT_JOSE);

        if method == Method::GET {
            request = request.header(ENVELOPE_HEADER, sealed.text.as_str());
        } else {
            request = request
                .header("Content-Type", ACCEPT_JOSE)
                .body(sealed.text.clone());
        }

        if !self.bearer.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", self.bearer));
        }

        for (name, value) in extra {
            request = request.header(*name, value.as_str());
        }

        let reply = request.send().await?;
        let status = reply.status();
        let text = reply.text().await?;
        self.read(status, &text, &sealed.reply)
    }

    fn read(&self, status: StatusCode, text: &str, reply: &[u8]) -> AppResult<Value> {
        let envelope = match unseal(text, reply) {
            Ok(value) => value,
            Err(_) => {
                let plain: Value = serde_json::from_str(text)
                    .map_err(|_| AppError::network("the reply could not be read"))?;
                return Err(AppError::api(refusal(&plain_text(&plain))));
            }
        };

        let body = envelope.get("body").cloned().unwrap_or(Value::Null);
        let code = body.get("code").and_then(|row| row.as_u64()).unwrap_or(0);
        let message = body
            .get("msg")
            .and_then(|row| row.as_str())
            .unwrap_or_default()
            .to_string();

        if !status.is_success() || code != 200 {
            if !message.is_empty() {
                return Err(AppError::api(refusal(&message)));
            }
            return Err(AppError::api(refusal(&format!("HTTP {}", status.as_u16()))));
        }

        Ok(body.get("data").cloned().unwrap_or(Value::Null))
    }
}

fn plain_text(value: &Value) -> String {
    for key in ["message", "msg", "error", "detail", "code"] {
        if let Some(text) = value.get(key).and_then(|row| row.as_str()) {
            return text.to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_reads_the_first_known_key() {
        let value: Value = serde_json::from_str(r#"{"code":"COMMON_INVALID_ENVELOPE"}"#).unwrap();
        assert_eq!(plain_text(&value), "COMMON_INVALID_ENVELOPE");
    }

    #[test]
    fn an_unreadable_reply_is_a_network_error() {
        let link = Link::open(None).unwrap();
        let outcome = link.read(StatusCode::OK, "not json", &[0u8; 32]);
        assert!(outcome.is_err());
    }
}

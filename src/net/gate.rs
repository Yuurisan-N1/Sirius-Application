use std::time::Duration;

use reqwest::Client;
use serde_json::Value;

use crate::support::consts::net::{
    APP_AGENT, CONNECT_TIMEOUT_SECS, READ_TIMEOUT_SECS, SOLVER_POLLS, SOLVER_POLL_MILLIS,
    SOLVER_RESULT, SOLVER_TURNSTILE, SOLVER_URL, TURNSTILE_ACTION, TURNSTILE_PAGE,
    TURNSTILE_SITEKEY,
};
use crate::support::error::{AppError, AppResult};

pub struct Gate {
    base: String,
    http: Client,
}

impl Gate {
    pub fn open() -> AppResult<Gate> {
        let base = SOLVER_URL.trim_end_matches('/').to_string();
        if base.is_empty() {
            return Err(AppError::config("the solver address is empty"));
        }
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(READ_TIMEOUT_SECS))
            .user_agent(APP_AGENT)
            .build()
            .map_err(|_| AppError::network("the solver client could not be built"))?;
        Ok(Gate { base, http })
    }

    pub async fn token(&self) -> AppResult<String> {
        let submit = format!("{}{}", self.base, SOLVER_TURNSTILE);
        let reply = self
            .http
            .get(&submit)
            .query(&[
                ("url", TURNSTILE_PAGE),
                ("sitekey", TURNSTILE_SITEKEY),
                ("action", TURNSTILE_ACTION),
            ])
            .send()
            .await
            .map_err(|_| AppError::network("the solver could not be reached"))?;

        if !reply.status().is_success() {
            return Err(AppError::network("the solver refused the task"));
        }

        let body: Value = reply
            .json()
            .await
            .map_err(|_| AppError::network("the solver reply was not understood"))?;

        let task = body
            .get("task_id")
            .and_then(|row| row.as_str())
            .map(|row| row.to_string())
            .or_else(|| {
                body.get("id")
                    .and_then(|row| row.as_str())
                    .map(|row| row.to_string())
            })
            .ok_or_else(|| AppError::network("the solver did not return a task"))?;

        self.wait(&task).await
    }

    async fn wait(&self, task: &str) -> AppResult<String> {
        let poll = format!("{}{}", self.base, SOLVER_RESULT);
        for _ in 0..SOLVER_POLLS {
            tokio::time::sleep(Duration::from_millis(SOLVER_POLL_MILLIS)).await;
            let reply = self
                .http
                .get(&poll)
                .query(&[("id", task)])
                .send()
                .await
                .map_err(|_| AppError::network("the solver could not be reached"))?;
            if !reply.status().is_success() {
                continue;
            }
            let body: Value = match reply.json().await {
                Ok(value) => value,
                Err(_) => continue,
            };
            let state = body
                .get("status")
                .and_then(|row| row.as_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            match state.as_str() {
                "success" | "ready" | "done" => {
                    let token = body
                        .get("value")
                        .and_then(|row| row.as_str())
                        .or_else(|| body.get("token").and_then(|row| row.as_str()))
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if token.is_empty() {
                        return Err(AppError::network("the solver returned an empty token"));
                    }
                    return Ok(token);
                }
                "error" | "failed" => {
                    return Err(AppError::network("the solver could not pass the human check"));
                }
                _ => continue,
            }
        }
        Err(AppError::network("the solver ran out of time"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_never_ends_with_a_slash() {
        let gate = Gate::open().unwrap();
        assert!(!gate.base.is_empty());
        assert!(!gate.base.ends_with('/'));
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Network(String),
    #[error("{0}")]
    Api(String),
}

impl AppError {
    pub fn config(message: impl Into<String>) -> AppError {
        AppError::Config(message.into())
    }

    pub fn network(message: impl Into<String>) -> AppError {
        AppError::Network(message.into())
    }

    pub fn api(message: impl Into<String>) -> AppError {
        AppError::Api(message.into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> AppError {
        if err.is_timeout() {
            AppError::Network("the request timed out".to_string())
        } else if err.is_connect() {
            AppError::Network("the connection could not be established".to_string())
        } else if err.is_decode() {
            AppError::Network("the reply could not be read".to_string())
        } else {
            AppError::Network("the request failed".to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(_: serde_json::Error) -> AppError {
        AppError::Network("the reply could not be read".to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn refusal(message: &str) -> String {
    let text = message.to_ascii_lowercase();
    let known = if text.contains("auth_invalid_credential") || text.contains("invalid credential") {
        "the identity is not known to the server"
    } else if text.contains("auth_human_verification_failed") || text.contains("human_verification") {
        "the human check was refused"
    } else if text.contains("auth_account_disabled") || text.contains("account_disabled") {
        "the account is disabled"
    } else if text.contains("mining_inviter_required") {
        "an inviter has to be bound before mining"
    } else if text.contains("mining_already_running") || text.contains("already_running") {
        "a mining round is already running"
    } else if text.contains("mining_not_started") {
        "no mining round was started yet"
    } else if text.contains("already participated") || text.contains("already_participated") {
        "the task was already claimed"
    } else if text.contains("task_not_found") {
        "the task is not listed for this account"
    } else if text.contains("invitation_already_bound") || text.contains("already_bound") {
        "an inviter was already bound to this account"
    } else if text.contains("inviter_not_found") || text.contains("invalid_inviter") {
        "the inviter code is not known to the server"
    } else if text.contains("encryption_required") {
        "the server wants a sealed request"
    } else if text.contains("invalid_envelope") || text.contains("invalid encrypted request") {
        "the sealed request was refused"
    } else if text.contains("rate limit") || text.contains("too many") {
        "the server is rate limiting the account"
    } else if text.contains("unauthorized") || text.contains("token_expired") {
        "the session was refused"
    } else if text.contains("restricted") || text.contains("region") {
        "the region is not allowed"
    } else {
        return short(message);
    };
    known.to_string()
}

fn short(message: &str) -> String {
    let clean: String = message
        .chars()
        .filter(|c| !c.is_control())
        .take(120)
        .collect();
    let clean = clean.trim();
    if clean.is_empty() {
        "the server refused the request without a reason".to_string()
    } else {
        format!("the server said {}", clean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_refusal_is_written_for_a_human() {
        assert_eq!(
            refusal("AUTH_INVALID_CREDENTIAL"),
            "the identity is not known to the server"
        );
        assert_eq!(
            refusal("MINING_INVITER_REQUIRED"),
            "an inviter has to be bound before mining"
        );
        assert_eq!(
            refusal("AUTH_HUMAN_VERIFICATION_FAILED"),
            "the human check was refused"
        );
    }

    #[test]
    fn an_unknown_refusal_keeps_the_server_words() {
        let text = refusal("COMMON_NOT_FOUND");
        assert!(text.starts_with("the server said "));
        assert!(text.contains("COMMON_NOT_FOUND"));
    }

    #[test]
    fn an_empty_refusal_still_reads() {
        assert_eq!(
            refusal("   "),
            "the server refused the request without a reason"
        );
    }

    #[test]
    fn the_kind_names_the_layer() {
        assert_eq!(AppError::api("x").to_string(), "x");
        assert_eq!(AppError::config("x").to_string(), "x");
    }
}

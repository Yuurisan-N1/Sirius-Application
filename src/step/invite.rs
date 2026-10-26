use serde_json::json;

use crate::net::client::Link;
use crate::support::error::{AppError, AppResult};
use crate::wire::endpoint::INVITER;
use crate::wire::shape::{parse, Invite};

pub async fn bind(link: &Link, code: &str) -> AppResult<Invite> {
    let code = code.trim();
    if code.is_empty() {
        return Err(AppError::config("the inviter code is empty"));
    }
    let data = link.post(INVITER, &json!({ "inviterCode": code })).await?;
    let invite: Invite = parse(data, "inviter")?;
    if !invite.invitation_bound {
        return Err(AppError::api("the server did not bind the inviter"));
    }
    Ok(invite)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn an_empty_code_is_refused_before_the_call() {
        let link = Link::open(None).unwrap();
        let outcome = bind(&link, "   ").await;
        assert!(outcome.is_err());
    }
}

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::{json, Value};

use crate::job::account::Account;
use crate::step::enroll::{fresh_identity, fresh_installation};
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::{cash, gap, lg};
use crate::wire::seal::{seal, seal_reply, unseal};
use crate::wire::shape::amount;

pub fn execute() -> AppResult<()> {
    let sealed = seal("GET", "/auth/registration-config", "", None)?;
    let parts: Vec<&str> = sealed.text.split('.').collect();

    check(parts.len() == 5, "The envelope carries five parts")?;
    check(
        parts.iter().all(|row| !row.is_empty()),
        "Every envelope part carries data",
    )?;
    check(sealed.reply.len() == 32, "The reply key is 32 bytes")?;
    check(
        sealed.request_id.len() == 32,
        "The request id is 32 characters",
    )?;

    let header: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| AppError::api("the header was not readable"))?,
    )
    .map_err(|_| AppError::api("the header was not readable"))?;
    check(
        header.get("alg").and_then(|row| row.as_str()) == Some("RSA-OAEP-256"),
        "The header asks for RSA-OAEP-256",
    )?;
    check(
        header.get("enc").and_then(|row| row.as_str()) == Some("A256GCM"),
        "The header asks for A256GCM",
    )?;
    check(
        header.get("kid").and_then(|row| row.as_str()) == Some("kex"),
        "The header points at the exchange key",
    )?;

    let reply_text = seal_reply(
        &json!({
            "v": 1,
            "requestId": sealed.request_id,
            "status": 200,
            "body": {
                "code": 200,
                "msg": "success",
                "data": { "showTurnstileChallenge": true }
            }
        }),
        &sealed.reply,
    )?;
    let back = unseal(&reply_text, &sealed.reply)?;
    check(
        back.get("body")
            .and_then(|row| row.get("data"))
            .and_then(|row| row.get("showTurnstileChallenge"))
            .and_then(|row| row.as_bool())
            == Some(true),
        "A sealed reply comes back with the same data",
    )?;

    let identity = fresh_identity();
    let account = Account::parse_line(&identity)
        .map_err(|err| AppError::api(format!("the identity was refused: {}", err)))?;
    check(
        account.identity.len() == 64,
        "A fresh identity passes the reader",
    )?;
    check(
        fresh_installation().starts_with("e-"),
        "A fresh installation looks like a Firebase id",
    )?;
    check(
        (amount("5.0000") - 5.0).abs() < 0.0001,
        "An amount is read from its text form",
    )?;
    check(
        cash(5.0, "SST") == "5 SST",
        "A reward line is written without colour",
    )?;

    gap();
    lg("Every offline check passed");
    Ok(())
}

fn check(flag: bool, msg: &str) -> AppResult<()> {
    if flag {
        lg(msg);
        Ok(())
    } else {
        Err(AppError::api(msg))
    }
}

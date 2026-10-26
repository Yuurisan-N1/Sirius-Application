use crate::net::client::Link;
use crate::step::enroll;
use crate::support::consts::net::{BASE_URL, TURNSTILE_SITEKEY};
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::{gap, lg, ly};
use crate::wire::endpoint::REGISTRATION_CONFIG;

pub async fn execute() -> AppResult<()> {
    let link = Link::open(None)?;

    ly(&format!("Talking to {}", BASE_URL));

    let config = link.get(REGISTRATION_CONFIG).await?;
    let challenge = config
        .get("showTurnstileChallenge")
        .and_then(|row| row.as_bool())
        .unwrap_or(false);
    if challenge {
        ly("The server asks every new identity for a human check");
        ly(&format!("The human check uses the key {}", TURNSTILE_SITEKEY));
    } else {
        lg("The server does not ask for a human check right now");
    }

    let region = enroll::region(&link, "REGISTER").await?;
    if region.restricted {
        ly(&format!(
            "This address is seen as {} and the region is closed",
            country(&region.detected_country_code)
        ));
        if !region.restricted_country_codes.is_empty() {
            ly(&format!(
                "The server closes the region for {}",
                region.restricted_country_codes.join(", ")
            ));
        }
    } else {
        lg(&format!(
            "This address is seen as {} and the region is open",
            country(&region.detected_country_code)
        ));
    }

    let login = enroll::region(&link, "LOGIN").await?;
    if login.restricted {
        return Err(AppError::api(
            "The server refuses a sign in from this address",
        ));
    }

    gap();
    lg("The server answered every probe");
    Ok(())
}

fn country(code: &str) -> &str {
    if code.trim().is_empty() {
        "an unknown country"
    } else {
        code
    }
}

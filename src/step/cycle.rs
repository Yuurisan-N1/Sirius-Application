use chrono::{NaiveDateTime, Utc};

use crate::net::client::Link;
use crate::support::error::AppResult;
use crate::wire::endpoint::{MINING_START, MINING_STATUS};
use crate::wire::shape::{parse, MiningStatus, Round};

const STAMP: &str = "%Y-%m-%dT%H:%M:%S%.f";

pub async fn start(link: &Link) -> AppResult<Round> {
    let data = link.post(MINING_START, &serde_json::json!({})).await?;
    parse(data, "mining start")
}

pub async fn status(link: &Link) -> AppResult<MiningStatus> {
    let data = link.get(MINING_STATUS).await?;
    parse(data, "mining status")
}

pub fn stamp(text: &str) -> Option<NaiveDateTime> {
    let clean = text.trim();
    if clean.is_empty() {
        return None;
    }
    NaiveDateTime::parse_from_str(clean, STAMP)
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(clean, "%Y-%m-%dT%H:%M:%S").ok())
}

pub fn left_seconds(text: &str) -> Option<i64> {
    let when = stamp(text)?;
    let now = Utc::now().naive_utc();
    Some((when - now).num_seconds())
}

pub fn alive(status: &MiningStatus) -> bool {
    if !status.running() {
        return false;
    }
    match status.large_cycle_ends_at.as_deref() {
        Some(text) => left_seconds(text).map(|left| left > 0).unwrap_or(true),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_server_stamp_is_read() {
        let when = stamp("2026-10-26T07:50:11.670901").unwrap();
        assert_eq!(when.format("%Y-%m-%d").to_string(), "2026-10-26");
    }

    #[test]
    fn a_stamp_without_fraction_is_read() {
        assert!(stamp("2026-10-26T07:50:11").is_some());
    }

    #[test]
    fn an_empty_stamp_is_not_a_date() {
        assert!(stamp("").is_none());
        assert!(stamp("not a date").is_none());
    }

    #[test]
    fn a_past_stamp_leaves_no_seconds() {
        let left = left_seconds("2020-01-01T00:00:00").unwrap();
        assert!(left < 0);
    }

    #[test]
    fn a_round_that_ended_is_not_alive() {
        let status: MiningStatus = serde_json::from_str(
            r#"{"miningStatus":"RUNNING","largeCycleEndsAt":"2020-01-01T00:00:00"}"#,
        )
        .unwrap();
        assert!(!alive(&status));
    }

    #[test]
    fn a_stopped_round_is_not_alive() {
        let status: MiningStatus =
            serde_json::from_str(r#"{"miningStatus":"NOT_STARTED"}"#).unwrap();
        assert!(!alive(&status));
    }
}

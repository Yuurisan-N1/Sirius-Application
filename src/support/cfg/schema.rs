use std::path::Path;

use serde_json::Value;

use crate::support::consts::app::REFERRAL_CODE;
use crate::support::error::{AppError, AppResult};

const DEFAULTS: &str = r#"{
  "settings": {
    "sleep_seconds": 3600,
    "referral_code": "EghAsP"
  }
}
"#;

#[derive(Debug, Clone)]
pub struct Config {
    value: Value,
}

impl Config {
    pub fn defaults() -> Config {
        Config {
            value: serde_json::from_str(DEFAULTS).unwrap_or(Value::Null),
        }
    }

    pub fn parse(text: &str) -> AppResult<Config> {
        let value: Value = serde_json::from_str(text)
            .map_err(|err| AppError::config(format!("config.json is not valid json: {}", err)))?;
        let config = Config { value };
        config.validate()?;
        Ok(config)
    }

    pub fn load(path: &Path) -> AppResult<Config> {
        match std::fs::read_to_string(path) {
            Ok(text) => Config::parse(&text),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                std::fs::write(path, DEFAULTS).map_err(|err| {
                    AppError::config(format!("config.json could not be written: {}", err))
                })?;
                Ok(Config::defaults())
            }
            Err(err) => Err(AppError::config(format!(
                "config.json could not be read: {}",
                err
            ))),
        }
    }

    pub const DEFAULT_SLEEP_SECONDS: u64 = 3600;

    pub fn sleep_seconds(&self) -> u64 {
        let raw = self
            .pick_u64("sleep_seconds")
            .unwrap_or(Config::DEFAULT_SLEEP_SECONDS);
        raw.clamp(5, 86_400)
    }

    pub fn referral_code(&self) -> String {
        let raw = self.pick_str("referral_code").unwrap_or_default();
        let clean = raw.trim();
        if clean.is_empty() {
            REFERRAL_CODE.to_string()
        } else {
            clean.to_string()
        }
    }

    fn settings(&self) -> Option<&Value> {
        match self.value.get("settings") {
            Some(value) if value.is_object() => Some(value),
            _ => None,
        }
    }

    fn pick_str(&self, key: &str) -> Option<String> {
        let settings = self.settings()?;
        let found = settings.get(key)?;
        if let Some(raw) = found.as_str() {
            return Some(raw.to_string());
        }
        found.as_u64().map(|raw| raw.to_string())
    }

    fn pick_u64(&self, key: &str) -> Option<u64> {
        let settings = self.settings()?;
        let found = settings.get(key)?;
        if let Some(raw) = found.as_u64() {
            return Some(raw);
        }
        found.as_str().and_then(|raw| raw.trim().parse().ok())
    }

    fn validate(&self) -> AppResult<()> {
        if self.value.get("settings").is_none() {
            return Err(AppError::config("config.json needs a settings block"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_canon_block() {
        assert_eq!(DEFAULTS.len(), 81);
        assert_eq!(DEFAULTS.lines().count(), 6);
        let config = Config::parse(DEFAULTS).unwrap();
        assert_eq!(config.sleep_seconds(), 3600);
        assert_eq!(config.referral_code(), "EghAsP");
    }

    #[test]
    fn the_referral_code_is_read_from_the_settings() {
        let config = Config::parse(r#"{"settings":{"referral_code":"AbCdEf"}}"#).unwrap();
        assert_eq!(config.referral_code(), "AbCdEf");
    }

    #[test]
    fn an_empty_referral_code_falls_back_to_the_default() {
        let config = Config::parse(r#"{"settings":{"referral_code":"   "}}"#).unwrap();
        assert_eq!(config.referral_code(), REFERRAL_CODE);
        let config = Config::parse(r#"{"settings":{"sleep_seconds":60}}"#).unwrap();
        assert_eq!(config.referral_code(), REFERRAL_CODE);
    }

    #[test]
    fn sleep_is_clamped() {
        let config = Config::parse(r#"{"settings":{"sleep_seconds":1}}"#).unwrap();
        assert_eq!(config.sleep_seconds(), 5);
        let config = Config::parse(r#"{"settings":{"sleep_seconds":99999999}}"#).unwrap();
        assert_eq!(config.sleep_seconds(), 86_400);
    }

    #[test]
    fn missing_settings_block_is_rejected() {
        assert!(Config::parse("{}").is_err());
    }

    #[test]
    fn the_settings_block_holds_the_sleep_and_the_referral() {
        let config = Config::parse(DEFAULTS).unwrap();
        let settings = config.value.get("settings").unwrap();
        assert_eq!(settings.as_object().unwrap().len(), 2);
    }
}

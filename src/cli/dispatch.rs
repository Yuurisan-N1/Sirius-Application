use std::path::Path;

use crate::cli::{enroll, probe, selftest};
use crate::job::r#loop;
use crate::support::cfg::loader;
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::ly;

pub async fn execute(root: &Path) -> AppResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args
        .first()
        .map(|row| row.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let rest = args.get(1).cloned().unwrap_or_default();

    match mode.as_str() {
        "" | "run" => {
            let setup = loader::load(root)?;
            ly(&format!(
                "The bot holds {} identities and {} proxies",
                setup.accounts.len(),
                setup.proxies.len()
            ));
            r#loop::execute(&setup).await
        }
        "enroll" => enroll::execute(root, &rest).await,
        "probe" => probe::execute().await,
        "selftest" => selftest::execute(),
        other => Err(AppError::config(format!(
            "{} is not a known mode; use run, enroll, probe or selftest",
            other
        ))),
    }
}

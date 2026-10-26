use std::path::{Path, PathBuf};

use crate::job::account::Account;
use crate::net::egress::Proxy;
use crate::support::cfg::schema::Config;
use crate::support::consts::app::{CONFIG_FILE, DATA_FILE, PROXY_FILE};
use crate::support::error::{AppError, AppResult};

pub struct Setup {
    pub root: PathBuf,
    pub config: Config,
    pub accounts: Vec<Account>,
    pub proxies: Vec<Proxy>,
}

impl Setup {
    pub fn proxy_for(&self, index: usize) -> Option<&Proxy> {
        if self.proxies.is_empty() {
            return None;
        }
        self.proxies.get(index % self.proxies.len())
    }
}

pub fn load(root: &Path) -> AppResult<Setup> {
    let config = Config::load(&root.join(CONFIG_FILE))?;

    let data_path = root.join(DATA_FILE);
    let data_text = std::fs::read_to_string(&data_path)
        .map_err(|err| AppError::config(format!("{} could not be read: {}", DATA_FILE, err)))?;

    let mut accounts: Vec<Account> = Vec::new();
    for (index, line) in data_text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let account = Account::parse_line(trimmed)
            .map_err(|err| AppError::config(format!("{} line {}: {}", DATA_FILE, index + 1, err)))?;
        if accounts.iter().any(|known| known.identity == account.identity) {
            continue;
        }
        accounts.push(account);
    }

    if accounts.is_empty() {
        return Err(AppError::config(format!(
            "No identity was found in {}",
            DATA_FILE
        )));
    }

    let mut proxies: Vec<Proxy> = Vec::new();
    let proxy_path = root.join(PROXY_FILE);
    if let Ok(text) = std::fs::read_to_string(&proxy_path) {
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let proxy = Proxy::parse(trimmed).map_err(|err| {
                AppError::config(format!("{} line {}: {}", PROXY_FILE, index + 1, err))
            })?;
            proxies.push(proxy);
        }
    }

    Ok(Setup {
        root: root.to_path_buf(),
        config,
        accounts,
        proxies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(proxies: Vec<Proxy>) -> Setup {
        Setup {
            root: PathBuf::from("."),
            config: Config::defaults(),
            accounts: Vec::new(),
            proxies,
        }
    }

    #[test]
    fn proxy_is_paired_by_position() {
        let setup = setup(vec![
            Proxy::parse("socks5://127.0.0.1:1080").unwrap(),
            Proxy::parse("socks5://127.0.0.1:1081").unwrap(),
        ]);
        assert_eq!(setup.proxy_for(0).map(|p| p.port), Some(1080));
        assert_eq!(setup.proxy_for(1).map(|p| p.port), Some(1081));
        assert_eq!(setup.proxy_for(2).map(|p| p.port), Some(1080));
    }

    #[test]
    fn empty_proxy_file_means_direct() {
        assert!(setup(Vec::new()).proxy_for(0).is_none());
    }
}

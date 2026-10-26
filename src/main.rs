mod cli;
mod job;
mod net;
mod step;
mod support;
mod wire;

use std::process::ExitCode;

use crate::cli::dispatch;
use crate::support::banner::print_banner;
use crate::support::consts::app::CONFIG_FILE;
use crate::support::log::format::{finish_status, lr};

#[tokio::main]
async fn main() -> ExitCode {
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        finish_status();
        lr("Script stopped by user");
        std::process::exit(0);
    });

    support::log::init::arm();
    print_banner();

    match dispatch::execute(&root_dir()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            lr(&format!("The program stopped: {}", err));
            ExitCode::FAILURE
        }
    }
}

fn root_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::current_dir() {
        if dir.join(CONFIG_FILE).is_file() {
            return dir;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join(CONFIG_FILE).is_file() {
                return parent.to_path_buf();
            }
            if let Some(grand) = parent.parent() {
                if grand.join(CONFIG_FILE).is_file() {
                    return grand.to_path_buf();
                }
            }
        }
    }

    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_always_has_a_value() {
        assert!(!root_dir().to_string_lossy().is_empty());
    }
}

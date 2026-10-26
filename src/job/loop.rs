use crate::job::account::{self, Stop};
use crate::support::banner::print_banner;
use crate::support::cfg::loader::Setup;
use crate::support::error::AppResult;
use crate::support::log::format::{countdown, gap, lr, ly};

pub async fn execute(setup: &Setup) -> AppResult<()> {
    let mut cycle: u64 = 0;

    loop {
        cycle += 1;
        ly(&format!("Starting automation cycle number {}", cycle));

        let base = (cycle as usize - 1) * setup.accounts.len();

        for (index, account) in setup.accounts.iter().enumerate() {
            if index > 0 {
                gap();
            }
            match account::run(setup, base + index, account).await {
                Ok(()) => {}
                Err(Stop::SignIn(err)) => {
                    lr(&format!("Account {} could not sign in: {}", index + 1, err));
                }
                Err(Stop::Work(err)) => {
                    lr(&format!("Account {} stopped early: {}", index + 1, err));
                }
            }
        }

        countdown(setup.config.sleep_seconds(), "Next cycle starts in").await;
        print_banner();
    }
}

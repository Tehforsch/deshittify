use anyhow::Result;
use telegram::run_bot;

pub mod database;
pub mod telegram;
pub mod time_frame;
pub mod action;
pub mod config;
pub mod response;

fn main() -> Result<()> {
    run_bot()
}


use anyhow::Result;
use telegram::run_bot;

pub mod action;
pub mod challenge_data;
pub mod config;
pub mod database;
pub mod response;
pub mod telegram;
pub mod time_frame;

fn main() -> Result<()> {
    run_bot()
}

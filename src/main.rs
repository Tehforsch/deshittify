use crate::telegram::run_bot;
use anyhow::Result;

pub mod action;
pub mod action_handling;
pub mod config;
pub mod database;
pub mod response;
pub mod telegram;
pub mod time_frame;
pub mod task_handling;

fn main() -> Result<()> {
    run_bot()
}

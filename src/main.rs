use crate::database::Database;
use anyhow::Result;
use telegram::run_bot;
use std::path::Path;

pub mod database;
pub mod telegram;
pub mod time_frame;
pub mod action;
pub mod config;
pub mod response;

fn main() -> Result<()> {
    let path = Path::new("test.db");
    let database = Database::new(&path);
    // poll_updates(&database)?;
    run_bot(&database)
}


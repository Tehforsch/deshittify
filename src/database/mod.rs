pub mod badge;
pub mod challenge;
pub mod group;
pub mod task;
pub mod user;

use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Database {
        Database {
            connection: Connection::open(&path).unwrap(),
        }
    }
    pub fn add_challenge(&self, challenge: &challenge::Challenge) -> Result<i64> {
        self.connection.execute(
            "INSERT INTO challenge (name, time_start, time_end) VALUES (?1, ?2, ?3)",
            params![
                challenge.name,
                challenge.time_frame.0,
                challenge.time_frame.1
            ],
        )?;

        Ok(self.connection.last_insert_rowid())
    }
}

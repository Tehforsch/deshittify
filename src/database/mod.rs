pub mod badge;
pub mod challenge;
pub mod group;
pub mod task;
pub mod user;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::Path;

use crate::challenge_data::ChallengeData;

use self::challenge::Challenge;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Database {
        Database {
            connection: Connection::open(&path).unwrap(),
        }
    }

    pub fn add_challenge(&self, challenge: &ChallengeData) -> Result<Challenge> {
        self.connection.execute(
            "INSERT INTO challenge (name, time_start, time_end) VALUES (?1, ?2, ?3)",
            params![
                challenge.name,
                challenge.time_frame.0,
                challenge.time_frame.1
            ],
        )?;

        let id = self.connection.last_insert_rowid();
        let mut statement = self
            .connection
            .prepare("SELECT id FROM challenge WHERE rowid = ?1")?;
        let challenge_id = statement
            .query_map(params![id], |row| Ok(row.get(0)?))?
            .next()
            .unwrap()?;

        Ok(Challenge {
            id: challenge_id,
            data: challenge.clone(),
        })
    }

    pub fn check_user_subscribed_to_challenge(
        &self,
        user_id: i32,
        challenge_id: i32,
    ) -> Result<bool> {
        let mut statement = self.connection.prepare(
            "SELECT user_id FROM userChallenge WHERE user_id = ?1 AND challenge_id = ?2",
        )?;
        statement
            .exists(params![user_id, challenge_id,])
            .context("")
    }

    pub fn get_challenge_id_from_name(&self, challenge_name: &str) -> Result<i32> {
        todo!()
    }

    pub fn subscribe_to_challenge(&self, user_id: i32, challenge_id: i32) -> Result<bool> {
        let user_already_subscribed =
            self.check_user_subscribed_to_challenge(user_id, challenge_id)?;
        if !user_already_subscribed {
            self.connection.execute(
                "INSERT INTO userChallenge (user_id, challenge_id) VALUES (?1, ?2)",
                params![user_id, challenge_id,],
            )?;
        }
        Ok(user_already_subscribed)
    }

    // pub fn exists(&self, statement: &str) -> Result<bool> {
    //     let statement = self.connection.prepare(&statement)?;
    //     statement.exists()
    // }
}

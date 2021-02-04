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

        Ok(Challenge {
            id: self.connection.last_insert_rowid(),
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

    pub fn subscribe_to_challenge(&self, user_id: i32, challenge_name: &str) -> Result<()> {
        let challenge_id = self.get_challenge_id_from_name(challenge_name)?;
        if !self.check_user_subscribed_to_challenge(user_id, challenge_id)? {
            self.connection.execute(
                "INSERT INTO userChallenge (user_id, challenge_id) VALUES (?1, ?2)",
                params![user_id, challenge_id,],
            )?;
        }
        Ok(())
    }

    // pub fn exists(&self, statement: &str) -> Result<bool> {
    //     let statement = self.connection.prepare(&statement)?;
    //     statement.exists()
    // }
}

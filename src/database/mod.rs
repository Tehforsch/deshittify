pub mod badge;
pub mod challenge;
pub mod challenge_data;
pub mod group;
pub mod period;
pub mod task;
pub mod task_data;
pub mod user;

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::Path;

use self::challenge_data::ChallengeData;
use self::{challenge::Challenge, task_data::TaskData};

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
                challenge.time_frame.start,
                challenge.time_frame.end
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

    pub fn add_task(&self, user_id: i32, challenge_name: &str, task_data: &TaskData) -> Result<()> {
        let challenge_id = self.get_challenge_id_by_user_id_and_name(user_id, challenge_name)?;
        self.connection.execute(
            "INSERT INTO task (user_id, challenge_id, name, count, period) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                user_id,
                challenge_id,
                task_data.name,
                task_data.count,
                task_data.period.to_string(),
            ],
        )?;
        Ok(())
    }

    fn get_challenge_id_by_user_id_and_name(
        &self,
        user_id: i32,
        challenge_name: &str,
    ) -> Result<i32> {
        let today = Local::today().to_string();
        let mut statement = self
            .connection
            .prepare("SELECT challenge.id FROM challenge, userChallenge WHERE challenge.id = userChallenge.challenge_id AND userChallenge.user_id = ?1 AND challenge.name = ?2 AND challenge.time_end > ?3")?;
        let challenge_id = statement
            .query_map(params![user_id, challenge_name, today], |row| {
                Ok(row.get(0)?)
            })?
            .next()
            .ok_or(anyhow!("No challenge with this name found for this user"))??;
        Ok(challenge_id)
    }
}

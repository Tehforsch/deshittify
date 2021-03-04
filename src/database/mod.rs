pub mod challenge;
pub mod challenge_data;
pub mod period;
pub mod task;
pub mod task_data;

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use rusqlite::{params, Connection, NO_PARAMS};
use std::path::Path;

use crate::response::UserTaskData;

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

    pub fn check_user_signed_up(&self, user_id: i32) -> Result<bool> {
        let mut statement = self
            .connection
            .prepare("SELECT user_id FROM user WHERE user_id = ?1")?;
        statement.exists(params![user_id,]).context("")
    }

    pub fn get_challenge_id_from_name(&self, challenge_name: &str) -> Result<i32> {
        todo!()
    }

    pub fn subscribe_to_challenge(&self, user_id: i32, challenge_id: i32) -> Result<bool> {
        let user_already_signed_up = self.check_user_signed_up(user_id)?;
        if !user_already_signed_up {
            return Err(anyhow!(
                "You have not signed up yet. Send a /signup to @deshittify_bot privately"
            ));
        }
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

    pub fn signup_user(&self, user_id: i32, chat_id: i64) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO user (user_id, chat_id) VALUES (?1, ?2)",
                params![user_id, chat_id,],
            )
            .context("While inserting user into table")
            .map(|_| ())
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
            .ok_or(anyhow!(
                "No (active) challenge with this name found for this user"
            ))??;
        Ok(challenge_id)
    }

    pub fn get_all_user_tasks(&self) -> Result<UserTaskData> {
        let mut statement = self.connection.prepare(
            "SELECT user.chat_id, task.name FROM user, task WHERE user.user_id = task.user_id ORDER BY user.chat_id",
        )?;
        let chat_ids_with_task_names = statement.query_map(params![], |row| {
            Ok(ChatIdWithTaskName {
                chat_id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut list_of_chat_ids_with_tasks: Vec<(i64, Vec<String>)> = vec![];
        for item in chat_ids_with_task_names {
            let item = item?;
            if list_of_chat_ids_with_tasks.len() == 0
                || item.chat_id != list_of_chat_ids_with_tasks.last().unwrap().0
            {
                list_of_chat_ids_with_tasks.push((item.chat_id, vec![]))
            }
            list_of_chat_ids_with_tasks
                .last()
                .unwrap()
                .1
                .push(item.name);
        }
        Ok(UserTaskData {
            data: list_of_chat_ids_with_tasks,
        })
    }
}

struct ChatIdWithTaskName {
    chat_id: i64,
    name: String,
}

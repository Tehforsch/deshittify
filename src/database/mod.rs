pub mod challenge;
pub mod challenge_data;
pub mod period;
pub mod task;
pub mod task_data;

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate, NaiveTime};
use itertools::Itertools;
use rusqlite::{params, Connection};
use std::path::Path;
use std::str::FromStr;

use crate::{
    action::UserPollDateInfo,
    config,
    response::{ChallengeUpdateData, ChallengeUserFractions, PollData, UserTaskData},
    task_handling::get_done_fraction,
    time_frame::TimeFrame,
};

use self::{challenge::Challenge, task_data::TaskData};
use self::{challenge_data::ChallengeData, period::Period, task::Task};

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
        user_id: &i32,
        challenge_id: &i32,
    ) -> Result<bool> {
        let mut statement = self.connection.prepare(
            "SELECT user_id FROM userChallenge WHERE user_id = ?1 AND challenge_id = ?2",
        )?;
        statement
            .exists(params![user_id, challenge_id,])
            .context("")
    }

    pub fn check_user_signed_up(&self, user_id: &i32) -> Result<bool> {
        let mut statement = self
            .connection
            .prepare("SELECT user_id FROM user WHERE user_id = ?1")?;
        statement.exists(params![user_id,]).context("")
    }

    pub fn get_challenge_id_from_name(&self, _challenge_name: &str) -> Result<i32> {
        todo!()
    }

    pub fn subscribe_to_challenge(&self, user_id: &i32, challenge_id: &i32) -> Result<bool> {
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

    pub fn signup_user(&self, user_id: &i32, chat_id: &i64, user_name: &str) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO user (user_id, chat_id, name) VALUES (?1, ?2, ?3)",
                params![user_id, chat_id, user_name,],
            )
            .context("While inserting user into table")
            .map(|_| ())
    }

    pub fn add_task(
        &self,
        user_id: &i32,
        challenge_name: &str,
        task_data: &TaskData,
    ) -> Result<()> {
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
        user_id: &i32,
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

    pub fn check_date_and_get_all_user_tasks(&self) -> Result<UserTaskData> {
        // if self.poll_already_sent_today()? || self.too_early(){
        // return Ok(UserTaskData { data: vec![] });
        // }
        println!("REMOVE COMMENT");
        self.write_poll_send_date()?;
        self.get_user_tasks()
    }

    pub fn check_date_and_get_challenge_update_data(&self) -> Result<ChallengeUpdateData> {
        // if self.challenge_update_already_sent_today()? || self.too_early() {
        // return Ok(ChallengeUpdateData { });
        // }
        println!("REMOVE COMMENT");
        self.write_challenge_update_send_date()?;
        self.get_challenge_update_data()
    }

    pub fn too_early(&self) -> bool {
        let datetime_now = Local::now().naive_local();
        let datetime_to_send_at = Local::today().naive_local().and_time(NaiveTime::from_hms(
            config::HOUR_TO_SEND_AT,
            0,
            0,
        ));
        datetime_now < datetime_to_send_at
    }

    pub fn get_user_tasks(&self) -> Result<UserTaskData> {
        let mut statement = self.connection.prepare(
            "SELECT user.user_id, user.chat_id, task.name FROM user, task WHERE user.user_id = task.user_id GROUP BY user.chat_id, task.name ORDER BY user.chat_id, task.name",
        )?;
        let mb_chat_ids_with_task_names = statement.query_map(params![], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let chat_ids_with_task_names: Vec<(i32, i64, String)> =
            mb_chat_ids_with_task_names.collect::<rusqlite::Result<Vec<(i32, i64, String)>>>()?;
        let mut data_grouped = UserTaskData { data: vec![] };
        for (key, group) in &chat_ids_with_task_names
            .into_iter()
            .group_by(|(user_id, chat_id, _)| (*user_id, *chat_id))
        {
            data_grouped.data.push(PollData {
                chat_id: key.1,
                task_names: group.map(|(_, _, name)| name).collect(),
                user_id: key.0,
            });
        }
        Ok(data_grouped)
    }

    pub fn get_challenges_and_chat_ids(&self) -> rusqlite::Result<Vec<(Challenge, i64)>> {
        let mut statement = self.connection.prepare(
            "SELECT challenge.id, challenge.name, challenge.time_start, challenge.time_end, user.chat_id FROM challenge, user, userChallenge WHERE user.user_id = userChallenge.user_id AND challenge.id = userChallenge.challenge_id",
        )?;
        let challenges_result = statement.query_map(params![], |row| {
            Ok((
                Challenge {
                    id: row.get::<_, i64>(0)?,
                    data: ChallengeData {
                        name: row.get::<_, String>(1)?,
                        time_frame: TimeFrame {
                            start: row.get::<_, NaiveDate>(2)?,
                            end: row.get::<_, NaiveDate>(3)?,
                        },
                    },
                },
                row.get::<_, i64>(4)?,
            ))
        })?;
        challenges_result.collect()
    }

    pub fn get_tasks_for_challenge_and_user(
        &self,
        challenge_id: i64,
        user_id: i64,
    ) -> rusqlite::Result<Vec<Task>> {
        let mut statement = self
            .connection
            .prepare("SELECT task.id, task.name, task.count, task.period FROM task WHERE task.challenge_id = ?1 AND task.user_id = ?2")?;
        let result = statement.query_map(params![challenge_id, user_id], |row| {
            Ok(Task {
                id: row.get::<_, i64>(0)?,
                data: TaskData {
                    name: row.get::<_, String>(1)?,
                    count: row.get::<_, i32>(2)?,
                    period: Period::from_str(&row.get::<_, String>(3)?).unwrap(),
                },
            })
        })?;
        result.collect()
    }

    pub fn get_challenge_users(&self, challenge_id: i64) -> rusqlite::Result<Vec<(i64, String)>> {
        let mut statement = self
            .connection
            .prepare("SELECT user.user_id, user.name FROM user, userChallenge WHERE userChallenge.user_id = user.user_id AND userChallenge.challenge_id = ?1")?;
        let result = statement.query_map(params![challenge_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        result.collect()
    }

    pub fn get_timestamps_for_task(&self, task_name: &str) -> rusqlite::Result<Vec<NaiveDate>> {
        let mut statement = self
            .connection
            .prepare("SELECT userPollDate.date FROM userPollDate WHERE userPollDate.done = 1 AND userPollDate.task_id = ?1")?;
        let result =
            statement.query_map(params![task_name], |row| Ok(row.get::<_, NaiveDate>(0)?))?;
        result.collect()
    }

    pub fn get_average_done_fraction(&self, challenge: &Challenge, user_id: i64) -> Result<f64> {
        let tasks = self.get_tasks_for_challenge_and_user(challenge.id, user_id)?;
        let mut average_fraction = 0.0;
        for task in tasks.iter() {
            let timestamps = self.get_timestamps_for_task(&task.data.name)?;
            let done_fraction =
                get_done_fraction(&task.data, &timestamps, &challenge.data.time_frame);
            average_fraction += done_fraction;
        }
        Ok(average_fraction / (tasks.len() as f64))
    }

    pub fn get_challenge_update_data(&self) -> Result<ChallengeUpdateData> {
        let challenges_and_chat_ids = self.get_challenges_and_chat_ids()?;
        let mut challenge_user_fractions = vec![];
        for (challenge, chat_id) in challenges_and_chat_ids.iter() {
            let mut user_fractions = vec![];
            for (user_id, user_name) in self.get_challenge_users(challenge.id)? {
                let average_fraction = self.get_average_done_fraction(&challenge, user_id)?;
                user_fractions.push((user_name, average_fraction));
            }
            challenge_user_fractions.push(ChallengeUserFractions {
                challenge: challenge.clone(),
                chat_id: *chat_id,
                user_fractions: user_fractions,
            });
        }
        Ok(ChallengeUpdateData(challenge_user_fractions))
    }

    pub fn write_poll_send_date(&self) -> Result<()> {
        let date_today = Local::today().naive_local();
        self.connection.execute(
            "INSERT INTO pollSendDate (date) VALUES (?1)",
            params![date_today],
        )?;
        Ok(())
    }

    pub fn write_challenge_update_send_date(&self) -> Result<()> {
        let date_today = Local::today().naive_local();
        self.connection.execute(
            "INSERT INTO challengeUpdateSendDate (date) VALUES (?1)",
            params![date_today],
        )?;
        Ok(())
    }

    pub fn poll_already_sent_today(&self) -> Result<bool> {
        let date_today = Local::today().naive_local();
        let mut statement = self
            .connection
            .prepare("SELECT id FROM pollSendDate WHERE date = ?1")?;
        statement.exists(params![date_today,]).context("")
    }

    pub fn challenge_update_already_sent_today(&self) -> Result<bool> {
        let date_today = Local::today().naive_local();
        let mut statement = self
            .connection
            .prepare("SELECT id FROM challengeUpdateSendDate WHERE date = ?1")?;
        statement.exists(params![date_today,]).context("")
    }

    pub fn modify_user_task_entries(&self, poll_id: &str, option_ids: &Vec<i32>) -> Result<()> {
        // Set everything to done=false first, because we're lazy af
        self.connection.execute(
            "UPDATE userPollDate SET done = 0 WHERE poll_id = ?1",
            params![poll_id],
        )?;
        // Set some of them to done now, because we're awesome
        for index in option_ids.iter() {
            self.connection.execute(
                "UPDATE userPollDate SET done = ?1 WHERE poll_id = ?2 AND task_index = ?3",
                params![1, poll_id, index],
            )?;
        }
        Ok(())
    }

    pub fn write_poll_info(&self, info: &Vec<UserPollDateInfo>) -> Result<()> {
        for user_poll_date_info in info.iter() {
            self.connection.execute(
                "INSERT INTO userPollDate (date, user_id, poll_id, task_id, task_index, done) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![user_poll_date_info.date, user_poll_date_info.user_id, user_poll_date_info.poll_id, user_poll_date_info.task_id, user_poll_date_info.task_index, 0]
            )?;
        }
        Ok(())
    }

    pub fn get_date_from_poll_id(&self, poll_id: &str) -> Result<NaiveDate> {
        let mut statement = self
            .connection
            .prepare("SELECT date FROM pollDate WHERE poll_id = ?1")?;
        let result: rusqlite::Result<NaiveDate> = statement
            .query_map(params![poll_id,], |row| row.get::<_, NaiveDate>(0))?
            .next()
            .ok_or(anyhow!(
                "The poll with id {} is not in the userPoll database",
                poll_id
            ))?;
        result.context("")
    }
}

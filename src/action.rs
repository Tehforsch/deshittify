use chrono::NaiveDate;

use crate::database::{challenge_data::ChallengeData, task_data::TaskData};

#[derive(Debug)]
pub enum Action {
    AddTask(i32, String, TaskData),
    CreateNewChallenge(ChallengeData),
    SubscribeToChallenge(i32, i32, String),
    SendHelp,
    SignupUser(i32, i64, String),
    ErrorMessage(String),
    CheckDateMaybeSendPolls,
    CheckDateMaybeSendChallengeUpdates,
    ModifyUserTaskTimestamps(String, Vec<i32>),
    WritePollInfo(Vec<UserPollDateInfo>),
    Nothing,
}

#[derive(Debug)]
pub struct UserPollDateInfo {
    pub user_id: i32,
    pub date: NaiveDate,
    pub task_id: String,
    pub poll_id: String,
    pub task_index: i32,
}

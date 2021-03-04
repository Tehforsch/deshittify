use crate::database::{challenge_data::ChallengeData, task_data::TaskData};

#[derive(Debug)]
pub enum Action {
    AddTask(i32, String, TaskData),
    CreateNewChallenge(ChallengeData),
    SubscribeToChallenge(i32, i32, String),
    SendHelp,
    SignupUser(i32, i64),
    ErrorMessage(String),
    SendTaskPoll,
}

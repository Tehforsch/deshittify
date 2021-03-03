use crate::database::{challenge_data::ChallengeData, task_data::TaskData};

#[derive(Debug)]
pub enum Action {
    AddTask(i32, String, TaskData),
    CreateNewChallenge(ChallengeData),
    SubscribeChallenge(i32, i32, String),
    SendHelp,
    Test,
    ErrorMessage(String),
}

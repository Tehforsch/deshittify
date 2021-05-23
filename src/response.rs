use crate::database::{
    challenge::Challenge, challenge_performance_data::ChallengePerformanceData,
    task_performance_data::TaskPerformanceData, user::UserData,
};

#[derive(Debug)]
pub enum Response {
    Reply(String),
    SendHelp,
    SubscriptionPrompt(Challenge),
    Nothing,
    TaskPolls(UserTaskData),
    ChallengeUpdates(ChallengeUpdateData),
}

#[derive(Debug)]
pub struct UserTaskData {
    pub data: Vec<PollData>,
}

#[derive(Debug)]
pub struct ChallengeUpdateData(pub Vec<ChallengePerformanceData>);

#[derive(Debug)]
pub struct PollData {
    pub chat_id: i64,
    pub user_id: i32,
    pub task_names: Vec<String>,
}

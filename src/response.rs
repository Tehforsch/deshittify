use crate::database::challenge::Challenge;

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
pub struct ChallengeUpdateData(pub Vec<ChallengeUserFractions>);

#[derive(Debug)]
pub struct ChallengeUserFractions {
    pub chat_id: i64,
    pub challenge: Challenge,
    pub user_fractions: Vec<(String, f64)>,
}

#[derive(Debug)]
pub struct PollData {
    pub chat_id: i64,
    pub user_id: i32,
    pub task_names: Vec<String>,
}

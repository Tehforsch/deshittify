use crate::database::challenge::Challenge;

#[derive(Debug)]
pub enum Response {
    Reply(String),
    SendHelp,
    SubscriptionPrompt(Challenge),
    Nothing,
    TaskPolls(UserTaskData),
}

#[derive(Debug)]
pub struct UserTaskData {
    pub data: Vec<(i64, Vec<String>)>,
}

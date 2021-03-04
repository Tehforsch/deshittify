use crate::database::challenge::Challenge;
use crate::database::task::Task;

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
    data: Vec<(i64, Vec<String>)>,
}

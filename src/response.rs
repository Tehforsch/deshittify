use crate::database::challenge::Challenge;

#[derive(Debug)]
pub enum Response {
    Reply(String),
    SendHelp,
    Test,
    SubscriptionPrompt(Challenge),
    Nothing,
}

use crate::database::challenge::Challenge;

pub enum Response {
    Reply(String),
    SendHelp,
    Test,
    SubscriptionPrompt(Challenge),
}

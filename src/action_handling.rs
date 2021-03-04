use anyhow::Result;
use std::path::Path;

use crate::{
    action::Action,
    config,
    database::{challenge_data::ChallengeData, task_data::TaskData, Database},
    response::Response,
};

pub fn perform_action(action: &Action) -> anyhow::Result<Response> {
    let database = Database::new(&Path::new(config::DEFAULT_DB_PATH));
    match action {
        Action::CreateNewChallenge(challenge_data) => {
            create_new_challenge(&database, challenge_data)
        }
        Action::SubscribeToChallenge(user_id, challenge_id, user_name) => {
            subscribe_to_challenge(&database, user_id, challenge_id, user_name)
        }
        Action::AddTask(user_id, challenge_name, task_data) => {
            add_task(&database, user_id, challenge_name, task_data)
        }
        Action::SignupUser(user_id, chat_id) => signup_user(&database, user_id, chat_id),
        Action::ErrorMessage(message_text) => reply(&message_text),
        Action::SendTaskPoll => send_task_polls(&database),
        Action::SendHelp => Ok(Response::SendHelp),
    }
}

fn send_task_polls(database: &Database) -> Result<Response> {
    Ok(Response::TaskPolls(database.get_all_user_tasks()?))
}

fn reply(message_text: &str) -> Result<Response> {
    Ok(Response::Reply(message_text.to_string()))
}

fn signup_user(database: &Database, user_id: &i32, chat_id: &i64) -> Result<Response> {
    database.signup_user(user_id, chat_id)?;
    Ok(Response::Reply("Thanks. You signed up.".to_owned()))
}

fn add_task(
    database: &Database,
    user_id: &i32,
    challenge_name: &str,
    task_data: &TaskData,
) -> Result<Response> {
    database.add_task(user_id, challenge_name, task_data)?;
    Ok(Response::Reply(format!(
        "Task {} added. Kaclxokca!",
        task_data.name
    )))
}

fn subscribe_to_challenge(
    database: &Database,
    user_id: &i32,
    challenge_id: &i32,
    user_name: &str,
) -> Result<Response> {
    let already_subscribed = database.subscribe_to_challenge(user_id, challenge_id)?;
    if !already_subscribed {
        Ok(Response::Reply(format!(
            "{} accepted the challenge! Kaclxokca!",
            user_name
        )))
    } else {
        Ok(Response::Nothing)
    }
}

fn create_new_challenge(database: &Database, challenge_data: &ChallengeData) -> Result<Response> {
    let challenge = database.add_challenge(challenge_data)?;
    Ok(Response::SubscriptionPrompt(challenge))
}
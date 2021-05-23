use anyhow::Result;

use std::path::Path;

use crate::{
    action::{Action, UserPollDateInfo},
    config,
    database::{challenge_data::ChallengeData, task_data::TaskData, Database},
    response::Response,
};

pub fn perform_action(action: &Action) -> Response {
    let database = Database::new(&Path::new(config::DEFAULT_DB_PATH));
    let mb_response = match action {
        Action::CreateNewChallenge(challenge_data) => {
            create_new_challenge(&database, challenge_data)
        }
        Action::SubscribeToChallenge(user_id, challenge_id, user_name) => {
            subscribe_to_challenge(&database, user_id, challenge_id, user_name)
        }
        Action::AddTask(user_id, challenge_name, task_data) => {
            add_task(&database, user_id, challenge_name, task_data)
        }
        Action::SignupUser(user_id, chat_id, user_name) => signup_user(&database, user_id, chat_id, user_name),
        Action::ErrorMessage(message_text) => reply(&message_text),
        Action::CheckDateMaybeSendPolls => {
            send_task_polls(&database)
        }
        Action::CheckDateMaybeSendChallengeUpdates => send_challenge_updates(&database),
        Action::SendHelp => Ok(Response::SendHelp),
        Action::ModifyUserTaskTimestamps(poll_id, option_ids) => {
            modify_user_task_timestamps(&database, poll_id, option_ids)
        }
        Action::WritePollInfo(info) => write_poll_info(&database, info),
        Action::Nothing => Ok(Response::Nothing),
    };
    mb_response.unwrap_or_else(|err| Response::Reply(format!("Error: {}", err.to_string())))
}

fn write_poll_info(database: &Database, info: &Vec<UserPollDateInfo>) -> Result<Response> {
    database.write_poll_info(info)?;
    Ok(Response::Nothing)
}

fn modify_user_task_timestamps(
    database: &Database,
    poll_id: &str,
    poll_option_ids: &Vec<i32>,
) -> Result<Response> {
    database.modify_user_task_entries(poll_id, poll_option_ids)?;
    Ok(Response::Nothing)
}

fn send_task_polls(database: &Database) -> Result<Response> {
    Ok(Response::TaskPolls(
        database.check_date_and_get_all_user_tasks()?,
    ))
}

fn send_challenge_updates(database: &Database) -> Result<Response> {
    Ok(Response::ChallengeUpdates(
        database.check_date_and_get_challenge_update_data()?,
    ))
}

fn reply(message_text: &str) -> Result<Response> {
    Ok(Response::Reply(message_text.to_string()))
}

fn signup_user(database: &Database, user_id: &i32, chat_id: &i64, user_name: &str) -> Result<Response> {
    database.signup_user(user_id, chat_id, user_name)?;
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

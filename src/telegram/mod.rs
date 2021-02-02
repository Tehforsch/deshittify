pub mod command;

use std::{path::Path};
use anyhow::{Result, Context};
use chrono::Utc;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;
use chrono::prelude::*;


use crate::{database::challenge::Challenge, response::Response};
use crate::{action::Action, config, database::{Database}};
use self::command::Command;



#[tokio::main]
pub async fn run_bot(database: &Database) -> Result<()> {
    teloxide::enable_logging!();
    log::info!("Starting dices_bot...");

    let bot = Bot::from_env();
    let bot_name = "deshittify";

    teloxide::commands_repl(bot, bot_name, reply_command) .await;
    // let response = perform_action(database, &action)?;
    // send_response(update, response);
    Ok(())
}

async fn reply_command(message: UpdateWithCx<Message>, command: Command) -> Result<()> {
    let action = convert_message_to_action(&message, command);
    let response = perform_action(&action)?;
    perform_reponse(&response, &message).await
}

fn convert_message_to_action(_message: &UpdateWithCx<Message>, command: Command) -> Action {
    match command {
        Command::Help => Action::SendHelp,
        Command::CreateNewChallenge{ name }  => {
            Action::CreateNewChallenge(get_test_challenge(&name))
        }
    }
}

fn perform_action(action: &Action) -> anyhow::Result<Response> {
    let database = Database::new(&Path::new(config::DEFAULT_DB_PATH));
    Ok(match action {
        Action::CreateNewChallenge(challenge) => {
            let challenge_id = database.add_challenge(&challenge)?;
            Response::Reply(format!("Creating new challenge named {}! Kaclxokca!", &challenge.name))
        }
        Action::UserSubscribedToChallenge => {
            todo!()
        }
        Action::SendHelp => {
            Response::SendHelp
        }
    })
}

async fn perform_reponse(response: &Response, message: &UpdateWithCx<Message>) -> Result<()> {
    let result = match response {
        Response::Reply(text) => message.answer(text).send().await.context("While sending reply"),
        Response::SendHelp => message.answer(Command::descriptions()).send().await.context("While sending help"),
    };
    result.map(|_| ())
}

fn get_test_challenge(name: &str) -> Challenge {
    let dt1 = Utc.ymd(2014, 7, 8).and_hms_milli(9, 10, 11, 12);
    let dt2 = Utc.ymd(2015, 7, 8).and_hms_milli(9, 10, 11, 12);
    Challenge {
        id: None,
        name: name.to_string(),
        time_frame: (dt1, dt2),
    }
}

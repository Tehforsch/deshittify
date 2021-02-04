pub mod command;

use anyhow::{anyhow, Context, Result};
use chrono::prelude::*;
use chrono::Utc;
use std::path::Path;
use teloxide::{
    prelude::*,
    requests::SendMessage,
    types::{
        InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageKind,
        ReplyMarkup,
    },
};
use teloxide::{types::CallbackQuery, utils::command::BotCommand};
use tokio::join;

use self::command::Command;
use crate::{action::Action, challenge_data::ChallengeData, config, database::Database};
use crate::{database::challenge::Challenge, response::Response};

use std::sync::atomic::{AtomicU64, Ordering};

use lazy_static::lazy_static;

lazy_static! {
    static ref MESSAGES_TOTAL: AtomicU64 = AtomicU64::new(0);
}

#[tokio::main]
pub async fn run_bot() -> Result<()> {
    teloxide::enable_logging!();
    log::info!("Starting dices_bot...");

    let bot = Bot::from_env();
    let bot_name = "deshittify";

    // let message_future = teloxide::repl(bot.clone(), |message| async move {
    //     dbg!(&message);
    //     message.answer_dice().send().await?;
    //     ResponseResult::<()>::Ok(())
    // });
    // teloxide::commands_repl(bot, bot_name, reply_command);
    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.commands(bot_name)
                .for_each_concurrent(None, |(cx, command)| async move {
                    reply_command(cx, command).await.log_on_error().await;
                })
        })
        .callback_queries_handler(move |rx: DispatcherHandlerRx<CallbackQuery>| {
            rx.for_each_concurrent(None, |cx| async move {
                reply_callback_query(cx).await.log_on_error().await;
            })
        })
        .dispatch()
        .await;

    Ok(())
}

async fn reply_command(message: UpdateWithCx<Message>, command: Command) -> Result<()> {
    let action = convert_message_to_action(&message, command)?;
    let response = perform_action(&action)?;
    perform_reponse(&response, &message).await
}

async fn reply_callback_query(message: UpdateWithCx<CallbackQuery>) -> Result<()> {
    dbg!(&message);
    if let MessageKind::Common(x) = message.update.message.unwrap().kind {
        if let InlineKeyboardButtonKind::CallbackData(data) =
            &x.reply_markup.unwrap().inline_keyboard[0][0].kind
        {
            dbg!(&data);
        }
    };
    todo!()
    // let action = convert_message_to_action(&message, command)?;
    // let response = perform_action(&action)?
    // ;
    // perform_reponse(&response, &message).await
}

fn convert_message_to_action(message: &UpdateWithCx<Message>, command: Command) -> Result<Action> {
    match command {
        Command::Help => Ok(Action::SendHelp),
        Command::CreateNewChallenge { name } => {
            Ok(Action::CreateNewChallenge(get_test_challenge(&name)))
        }
        Command::Subscribe { challenge_name } => {
            let user = message
                .update
                .from()
                .ok_or_else(|| anyhow!("Subscribe message has no user!"))?;
            Ok(Action::SubscribeChallenge(user.id, challenge_name))
        }
        Command::Test => Ok(Action::Test),
    }
}

fn perform_action(action: &Action) -> anyhow::Result<Response> {
    let database = Database::new(&Path::new(config::DEFAULT_DB_PATH));
    Ok(match action {
        Action::CreateNewChallenge(challenge_data) => {
            let challenge = database.add_challenge(challenge_data)?;
            Response::SubscriptionPrompt(challenge)
        }
        Action::SubscribeChallenge(user_id, challenge_name) => {
            database.subscribe_to_challenge(*user_id, &challenge_name)?;
            Response::Reply(format!("Subscribed! Kaclxokca!"))
        }
        Action::SendHelp => Response::SendHelp,
        Action::Test => Response::Test,
    })
}

async fn perform_reponse(response: &Response, message: &UpdateWithCx<Message>) -> Result<()> {
    let result = match response {
        Response::Reply(text) => message
            .answer(text)
            .send()
            .await
            .context("While sending reply"),
        Response::SendHelp => message
            .answer(Command::descriptions())
            .send()
            .await
            .context("While sending help"),
        Response::Test => test(response, message).await,
        Response::SubscriptionPrompt(challenge) => {
            send_subscription_prompt(challenge, message).await
        }
    };
    result.map(|_| ())
}

async fn test(response: &Response, message: &UpdateWithCx<Message>) -> Result<Message> {
    let res = message
        .answer("DO YOU WANNA SUBSCRIBE TO MY PENIS?")
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::new(
                "SUBSCRIBE ME TO YOUR PENIS SIR",
                InlineKeyboardButtonKind::CallbackData("Subscribed".into()),
            )]]),
        ))
        .send()
        .await
        .context("");
    // let res = message.answer("").send().await.context("");
    println!("ja lol hey");
    res
}

async fn send_subscription_prompt(
    challenge: &Challenge,
    message: &UpdateWithCx<Message>,
) -> Result<Message> {
    let res = message
        .answer(format!("Subscribe to {}", &challenge.data.name))
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::new(
                "Subscribe",
                InlineKeyboardButtonKind::CallbackData(format!("{}", challenge.id)),
            )]]),
        ))
        .send()
        .await
        .context("");
    // let res = message.answer("").send().await.context("");
    println!("ja lol hey");
    res
}

fn get_test_challenge(name: &str) -> ChallengeData {
    let dt1 = Utc.ymd(2014, 7, 8).and_hms_milli(9, 10, 11, 12);
    let dt2 = Utc.ymd(2015, 7, 8).and_hms_milli(9, 10, 11, 12);
    ChallengeData {
        name: name.to_string(),
        time_frame: (dt1, dt2),
    }
}

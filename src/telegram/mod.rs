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
    log::info!("Starting deshittify_bot...");

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
            rx.commands(bot_name).for_each(|(cx, command)| async move {
                reply_command(cx, command).await.log_on_error().await;
            })
        })
        .callback_queries_handler(move |rx: DispatcherHandlerRx<CallbackQuery>| {
            rx.for_each(|cx| async move {
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
    let action = convert_callback_query_to_action(&message)?;
    let response = perform_action(&action)?;
    perform_reponse_to_callback_query(&response, &message).await
}

async fn perform_reponse_to_callback_query(
    response: &Response,
    update: &UpdateWithCx<CallbackQuery>,
) -> Result<()> {
    match response {
        Response::Reply(text) => {
            let chat_id = update.update.message.as_ref().unwrap().chat.id;
            dbg!(chat_id, text);
            update
                .bot
                .send_message(chat_id, text)
                .send()
                .await
                .context("While sending reply")?;
        }
        _ => {}
    }
    Ok(())
}

fn convert_callback_query_to_action(message: &UpdateWithCx<CallbackQuery>) -> Result<Action> {
    if let MessageKind::Common(x) = &message.update.message.as_ref().unwrap().kind {
        if let InlineKeyboardButtonKind::CallbackData(data) =
            &x.reply_markup.as_ref().unwrap().inline_keyboard[0][0].kind
        {
            let challenge_id: i32 = data.parse()?;
            let user_id = message.update.from.id;
            // let user_name = "asd".to_owned();
            let user_name = message.update.from.first_name.clone();
            return Ok(Action::SubscribeChallenge(user_id, challenge_id, user_name));
        }
    };
    todo!()
}

fn convert_message_to_action(message: &UpdateWithCx<Message>, command: Command) -> Result<Action> {
    match command {
        Command::Help => Ok(Action::SendHelp),
        Command::CreateNewChallenge { name } => {
            Ok(Action::CreateNewChallenge(get_test_challenge(&name)))
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
        Action::SubscribeChallenge(user_id, challenge_id, user_name) => {
            let already_subscribed = database.subscribe_to_challenge(*user_id, *challenge_id)?;
            if !already_subscribed {
                Response::Reply(format!("{} accepted the challenge! Kaclxokca!", user_name))
            } else {
                Response::Nothing
            }
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
        Response::Nothing => return Ok(()),
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

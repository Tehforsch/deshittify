pub mod command;
pub mod response_handling;

use anyhow::{Context, Result};
use teloxide::{
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageKind,
        ReplyMarkup,
    },
};
use teloxide::{types::CallbackQuery, utils::command::BotCommand};

use self::{command::Command, response_handling::perform_response};
use crate::action_handling::perform_action;
use crate::database::{challenge_data::ChallengeData, task_data::TaskData};
use crate::{action::Action, time_frame::TimeFrame};
use crate::{response::Response};

use std::sync::atomic::AtomicU64;

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
    let action = convert_message_to_action(&message, command)
        .unwrap_or_else(|err| Action::ErrorMessage(format!("Error: {}", err.to_string())));
    let response = perform_action(&action);
    perform_response(&response, &message).await
}

async fn reply_callback_query(message: UpdateWithCx<CallbackQuery>) -> Result<()> {
    let action = convert_callback_query_to_action(&message)
        .unwrap_or_else(|err| Action::ErrorMessage(format!("Error: {}", err.to_string())));
    let response = perform_action(&action);
    perform_reponse_to_callback_query(&response, &message).await
}

async fn perform_reponse_to_callback_query(
    response: &Response,
    update: &UpdateWithCx<CallbackQuery>,
) -> Result<()> {
    match response {
        Response::Reply(text) => {
            let chat_id = update.update.message.as_ref().unwrap().chat.id;
            send_text(&update.bot, chat_id, text).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn send_text(bot: &Bot, chat_id: i64, text: &str) -> Result<()> {
    bot.send_message(chat_id, text)
        .send()
        .await
        .context("While sending reply")?;
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
            return Ok(Action::SubscribeToChallenge(
                user_id,
                challenge_id,
                user_name,
            ));
        }
    };
    todo!()
}

fn convert_message_to_action(message: &UpdateWithCx<Message>, command: Command) -> Result<Action> {
    match command {
        Command::Help => Ok(Action::SendHelp),
        Command::CreateNewChallenge { name, start, end } => {
            Ok(Action::CreateNewChallenge(ChallengeData {
                name,
                time_frame: TimeFrame::new(start, end),
            }))
        }
        Command::AddTask {
            challenge_name,
            task_name,
            count,
            period,
        } => Ok(Action::AddTask(
            message.update.from().unwrap().id,
            challenge_name,
            TaskData {
                name: task_name,
                count,
                period,
            },
        )),
        Command::Signup => Ok(Action::SignupUser(
            message.update.from().unwrap().id,
            message.update.chat.id,
        )),
        Command::DeshittifyMyDay => Ok(Action::SendTaskPoll),
    }
}

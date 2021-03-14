pub mod command;
pub mod response_handling;

use anyhow::{Result};


use teloxide::types::{CallbackQuery, PollAnswer};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButtonKind, MessageKind},
};

use self::{
    command::Command,
    response_handling::perform_reponse_to_callback_query,
    response_handling::{
        perform_reponse_to_poll_answer, perform_response_to_command, send_user_task_polls,
    },
};
use crate::{action_handling::perform_action, config};
use crate::{
    database::{challenge_data::ChallengeData, task_data::TaskData},
    response::Response,
};

use crate::{action::Action, time_frame::TimeFrame};

use std::{sync::atomic::AtomicU64};

use lazy_static::lazy_static;
use tokio::{
    join,
    time::{delay_for, Duration},
};

lazy_static! {
    static ref MESSAGES_TOTAL: AtomicU64 = AtomicU64::new(0);
}

#[tokio::main]
pub async fn run_bot() -> Result<()> {
    teloxide::enable_logging!();
    log::info!("Starting deshittify_bot...");

    let bot = Bot::from_env();
    let bot_name = "deshittify";

    let reminder_sender = deshittify_my_life(Bot::from_env());

    let dispatcher = Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.commands(bot_name).for_each(|(cx, command)| async move {
                dbg!("Dispatch 1");
                handle_command(cx, command).await.log_on_error().await;
            })
        })
        .callback_queries_handler(move |rx: DispatcherHandlerRx<CallbackQuery>| {
            rx.for_each(|cx| async move {
                handle_callback_query(cx).await.log_on_error().await;
            })
        })
        .poll_answers_handler(move |rx: DispatcherHandlerRx<PollAnswer>| {
            rx.for_each(|cx| async move {
                handle_poll(cx).await.log_on_error().await;
            })
        });
    let handler = dispatcher.dispatch();

    let (res1, _) = join!(reminder_sender, handler);
    res1?;

    Ok(())
}

async fn deshittify_my_life(bot: Bot) -> Result<()> {
    loop {
        // bot.send_message(29424511, "hey was geht n so")
        //     .send()
        //     .await
        //     .context("While sending reply")?;
        delay_for(Duration::from_secs(config::DATE_CHECK_TIMEOUT_SECS)).await;
        let response = perform_action(&Action::CheckDateMaybeSendPolls);
        if let Response::TaskPolls(user_task_data) = response {
            send_user_task_polls(&bot, &user_task_data).await?;
        }
    }
}

async fn handle_command(message: UpdateWithCx<Message>, command: Command) -> Result<()> {
    let action = convert_message_to_action(&message, command)
        .unwrap_or_else(|err| Action::ErrorMessage(format!("Error: {}", err.to_string())));
    let response = perform_action(&action);
    let maybe_action = perform_response_to_command(&response, &message).await?;
    if let Some(new_action) = maybe_action {
        perform_action(&new_action);
    }
    Ok(())
}

async fn handle_callback_query(message: UpdateWithCx<CallbackQuery>) -> Result<()> {
    let action = convert_callback_query_to_action(&message)
        .unwrap_or_else(|err| Action::ErrorMessage(format!("Error: {}", err.to_string())));
    let response = perform_action(&action);
    perform_reponse_to_callback_query(&response, &message).await
}

async fn handle_poll(message: UpdateWithCx<PollAnswer>) -> Result<()> {
    let action = convert_poll_to_action(&message)
        .unwrap_or_else(|err| Action::ErrorMessage(format!("Error: {}", err.to_string())));
    let response = perform_action(&action);
    perform_reponse_to_poll_answer(&response, &message).await
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
    unimplemented!()
}

fn convert_poll_to_action(message: &UpdateWithCx<PollAnswer>) -> Result<Action> {
    let poll_id = &message.update.poll_id;
    let poll_options = message.update.option_ids.clone();
    Ok(Action::ModifyUserTaskTimestamps(
        poll_id.clone(),
        poll_options,
    ))
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
        Command::Signup => {
            if message.update.chat.is_private() {
                Ok(Action::SignupUser(
                    message.update.from().unwrap().id,
                    message.update.chat.id,
                ))
            } else {
                Ok(Action::ErrorMessage(
                    "You can't sign up in groups. Please sign up with @deshittify_bot directly."
                        .to_owned(),
                ))
            }
        }
        Command::DeshittifyMyDay => Ok(Action::CheckDateMaybeSendPolls),
    }
}

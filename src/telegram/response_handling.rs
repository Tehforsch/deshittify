use anyhow::{Context, Result};

use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, Message, ReplyMarkup,
};
use teloxide::utils::command::BotCommand;
use teloxide::{prelude::*, types::CallbackQuery};

use crate::{database::challenge::Challenge, response::Response};

use super::command::Command;

pub async fn perform_response_to_command(
    response: &Response,
    message: &UpdateWithCx<Message>,
) -> Result<()> {
    match response {
        Response::Reply(text) => {
            message.answer(text).send().await?;
        }
        Response::SendHelp => {
            message.answer(Command::descriptions()).send().await?;
        }
        Response::SubscriptionPrompt(challenge) => {
            send_subscription_prompt(challenge, message).await?;
        }
        Response::TaskPolls(task_polls) => {
            send_user_task_polls(task_polls)?;
        }
        Response::Nothing => {}
    };
    Ok(())
}

fn send_user_task_polls(task_polls: &crate::response::UserTaskData) -> Result<()> {
    for (user, tasks) in task_polls.data.iter() {
        dbg!(user, tasks);
    }
    Ok(())
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

pub async fn perform_reponse_to_callback_query(
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

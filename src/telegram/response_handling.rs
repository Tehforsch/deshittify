use anyhow::{Context, Result};

use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, Message, ReplyMarkup,
};
use teloxide::{utils::command::BotCommand};

use crate::{database::challenge::Challenge, response::Response};

use super::command::Command;

pub async fn perform_response(response: &Response, message: &UpdateWithCx<Message>) -> Result<()> {
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

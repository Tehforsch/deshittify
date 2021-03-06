use anyhow::{Context, Result};

use chrono::Local;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, Message, ReplyMarkup,
};
use teloxide::utils::command::BotCommand;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, MediaKind, MessageKind, PollAnswer},
};

use crate::{
    action::{Action, UserPollDateInfo},
    database::{challenge::Challenge, challenge_performance_data::ChallengePerformanceData},
    response::{ChallengeUpdateData, Response},
};

use super::command::Command;

pub async fn perform_response_to_command(
    response: &Response,
    message: &UpdateWithCx<Message>,
) -> Result<Option<Action>> {
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
            return Ok(Some(send_user_task_polls(&message.bot, task_polls).await?));
        }
        Response::ChallengeUpdates(challenge_updates) => {
            return Ok(Some(
                send_challenge_updates(&message.bot, challenge_updates).await?,
            ));
        }
        Response::Nothing => {}
    };
    Ok(None)
}

pub async fn send_user_task_polls(
    bot: &Bot,
    task_polls: &crate::response::UserTaskData,
) -> Result<Action> {
    let mut user_poll_date_infos = vec![];
    for poll_data in task_polls.data.iter() {
        let send_poll = bot
            .send_poll(
                poll_data.chat_id,
                "Which tasks did you do today?",
                poll_data.task_names.clone(),
            )
            .allows_multiple_answers(true)
            .is_anonymous(false)
            .send()
            .await?;
        for (i, task) in poll_data.task_names.iter().enumerate() {
            user_poll_date_infos.push(UserPollDateInfo {
                user_id: poll_data.user_id,
                date: Local::today().naive_local(),
                poll_id: get_poll_id(&send_poll),
                task_id: task.clone(),
                task_index: i as i32,
            });
        }
    }
    Ok(Action::WritePollInfo(user_poll_date_infos))
}

pub async fn send_challenge_updates(
    bot: &Bot,
    update_data: &ChallengeUpdateData,
) -> Result<Action> {
    for challenge_performance in update_data.0.iter() {
        send_text(
            &bot,
            &challenge_performance.chat_id,
            &get_challenge_performance_text(challenge_performance),
        )
        .await?;
    }
    Ok(Action::Nothing)
}

fn get_percent(fraction: f64) -> i64 {
    (fraction * 100.0).round() as i64
}

fn get_challenge_performance_text(challenge_performance: &ChallengePerformanceData) -> String {
    let lines: Vec<String> = challenge_performance
        .user_performance
        .iter()
        .map(|user_performance| {
            let alltime_percent = get_percent(
                user_performance.get_all_time_average(&challenge_performance.challenge.data),
            );
            let weekly_percent = get_percent(user_performance.get_weekly_average());
            format!(
                "{}:\t{}%\t(Last 7 days: {}%)",
                user_performance.user.name, alltime_percent, weekly_percent
            )
        })
        .collect();
    format!(
        "Update on challenge: {}\n{}",
        challenge_performance.challenge.data.name,
        lines.join("\n")
    )
}

pub fn get_poll_id(send_poll: &Message) -> String {
    if let MessageKind::Common(ref x) = send_poll.kind {
        if let MediaKind::Poll(ref y) = x.media_kind {
            return y.poll.id.clone();
        }
    }
    unimplemented!()
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
            send_text(&update.bot, &chat_id, text).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn send_text(bot: &Bot, chat_id: &i64, text: &str) -> Result<()> {
    bot.send_message(*chat_id, text)
        .send()
        .await
        .context("While sending reply")?;
    Ok(())
}

pub async fn perform_reponse_to_poll_answer(
    response: &crate::response::Response,
    _message: &UpdateWithCx<PollAnswer>,
) -> Result<()> {
    match response {
        Response::Nothing => {}
        _ => assert!(false),
    }
    Ok(())
}

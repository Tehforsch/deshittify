use chrono::NaiveDate;
use teloxide::utils::command::BotCommand;

use crate::database::period::Period;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Display help text.")]
    Help,
    #[command(description = "Create a new challenge", parse_with = "split")]
    CreateNewChallenge {
        name: String,
        start: NaiveDate,
        end: NaiveDate,
    },
    #[command(description = "Add a new task", parse_with = "split")]
    AddTask {
        challenge_name: String,
        task_name: String,
        count: i32,
        period: Period,
    },
    #[command(description = "Sign up for reminders", parse_with = "split")]
    Signup,
    #[command(
        description = "Deshittify the day by asking me all the stuff i havent actually done yet",
        parse_with = "split"
    )]
    SendPoll,
    SendUpdates,
}

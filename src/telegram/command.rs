use chrono::NaiveDate;
use teloxide::utils::command::BotCommand;

use crate::database::period::Period;
use crate::time_frame::TimeFrame;

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
    #[command(description = "Test the new hot shit", parse_with = "split")]
    Test,
}

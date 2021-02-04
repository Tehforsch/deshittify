use teloxide::utils::command::BotCommand;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Display help text.")]
    Help,
    #[command(description = "Create a new challenge", parse_with = "split")]
    CreateNewChallenge { name: String },
    #[command(description = "Subscribe", parse_with = "split")]
    Subscribe { challenge_name: String },
    #[command(description = "Test the new hot shit", parse_with = "split")]
    Test,
}

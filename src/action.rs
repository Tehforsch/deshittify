use crate::challenge_data::ChallengeData;

pub enum Action {
    CreateNewChallenge(ChallengeData),
    SubscribeChallenge(i32, i32, String),
    SendHelp,
    Test,
}

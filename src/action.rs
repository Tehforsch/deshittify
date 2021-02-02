use crate::database::challenge::Challenge;

pub enum Action {
    CreateNewChallenge(Challenge),
    UserSubscribedToChallenge,
    SendHelp,
}

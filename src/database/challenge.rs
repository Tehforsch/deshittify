use crate::database::challenge_data::ChallengeData;

#[derive(Clone, Debug)]
pub struct Challenge {
    pub id: i64,
    pub data: ChallengeData,
}

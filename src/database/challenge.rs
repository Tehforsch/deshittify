use crate::challenge_data::ChallengeData;

#[derive(Clone)]
pub struct Challenge {
    pub id: i64,
    pub data: ChallengeData,
}

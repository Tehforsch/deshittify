use crate::time_frame::TimeFrame;

#[derive(Clone)]
pub struct ChallengeData {
    pub name: String,
    pub time_frame: TimeFrame,
}

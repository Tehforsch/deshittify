use crate::time_frame::TimeFrame;

#[derive(Clone, Debug)]
pub struct ChallengeData {
    pub name: String,
    pub time_frame: TimeFrame,
}

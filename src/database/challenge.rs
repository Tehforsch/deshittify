use crate::time_frame::TimeFrame;

pub struct Challenge {
    pub id: Option<i64>,
    pub name: String,
    pub time_frame: TimeFrame,
}

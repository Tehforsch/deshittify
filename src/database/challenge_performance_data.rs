use super::{challenge::Challenge, user_performance_data::UserPerformanceData};

#[derive(Debug)]
pub struct ChallengePerformanceData {
    pub chat_id: i64,
    pub challenge: Challenge,
    pub user_performance: Vec<UserPerformanceData>,
}

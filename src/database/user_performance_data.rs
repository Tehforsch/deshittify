use chrono::{Duration, Local};

use crate::{task_handling::get_done_fraction, time_frame::TimeFrame};

use super::{
    challenge_data::ChallengeData, task_performance_data::TaskPerformanceData, user::UserData,
};

#[derive(Debug)]
pub struct UserPerformanceData {
    pub user: UserData,
    pub task_performance: Vec<TaskPerformanceData>,
}

impl UserPerformanceData {
    pub fn get_all_time_average(&self, challenge: &ChallengeData) -> f64 {
        self.get_average_fraction_for_timeframe(&challenge.time_frame)
    }

    pub fn get_weekly_average(&self) -> f64 {
        let today = Local::today().naive_local();
        let time_frame = TimeFrame {
            start: today.checked_sub_signed(Duration::days(8)).unwrap(),
            end: today.checked_sub_signed(Duration::days(1)).unwrap(),
        };
        self.get_average_fraction_for_timeframe(&time_frame)
    }

    fn get_average_fraction_for_timeframe(&self, time_frame: &TimeFrame) -> f64 {
        if self.task_performance.len() == 0 {
            1.0
        } else {
            self.task_performance
                .iter()
                .map(|performance| {
                    get_done_fraction(&performance.task, &performance.timestamps, time_frame)
                })
                .sum::<f64>()
                / self.task_performance.len() as f64
        }
    }
}

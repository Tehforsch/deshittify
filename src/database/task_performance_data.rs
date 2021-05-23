use chrono::NaiveDate;

use super::task_data::TaskData;

#[derive(Debug)]
pub struct TaskPerformanceData {
    pub task: TaskData,
    pub timestamps: Vec<NaiveDate>,
}

use super::task_data::TaskData;

#[derive(Debug)]
pub struct Task {
    pub id: i64,
    pub data: TaskData,
}

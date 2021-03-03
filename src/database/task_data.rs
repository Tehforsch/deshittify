use super::period::Period;

#[derive(Debug)]
pub struct TaskData {
    pub name: String,
    pub count: i32,
    pub period: Period,
}

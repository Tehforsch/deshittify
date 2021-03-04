use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub struct TimeFrame {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl TimeFrame {
    pub fn new(start: NaiveDate, end: NaiveDate) -> TimeFrame {
        TimeFrame { start, end }
    }
}

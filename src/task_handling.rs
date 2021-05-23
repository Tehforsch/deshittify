use chrono::{Datelike, Duration, NaiveDate, Weekday};

use crate::{database::{period::Period, task_data::TaskData}, time_frame::TimeFrame};

pub fn get_done_fraction(
    task: &TaskData,
    done_timestamps: &[NaiveDate],
    time_frame: &TimeFrame,
) -> f64 {
    match task.period {
        Period::Week => get_done_fraction_weekly(task.count, done_timestamps, &time_frame.start, &time_frame.end),
        Period::Month => get_done_fraction_monthly(task.count, done_timestamps, &time_frame.start, &time_frame.end),
        Period::Day => todo!(),
    }
}

fn get_done_fraction_weekly(
    count: i32,
    done_timestamps: &[NaiveDate],
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> f64 {
    let mut fractions = vec![];
    let week_day_counts = get_week_day_counts(&start_date, &end_date);
    let refs = Box::new(done_timestamps.iter().map(move |x| *x));
    for (week_start, week_end, days_in_week) in week_day_counts.iter() {
        let done_count = count_days_in_range(refs.clone(), week_start, week_end);
        let done_count = done_count.min(count as usize) as f64;
        let should_have_done_count = (count as f64 * (*days_in_week as f64 / 7.0)).floor() as f64;
        if should_have_done_count == 0.0 {
            fractions.push(1.0);
            continue;
        }

        fractions.push(done_count / should_have_done_count);
    }
    average(&fractions).unwrap_or(1.0)
}

fn get_done_fraction_monthly(
    count: i32,
    done_timestamps: &[NaiveDate],
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> f64 {
    let mut fractions = vec![];
    let month_day_counts = get_month_day_counts(&start_date, &end_date);
    let refs = Box::new(done_timestamps.iter().map(move |x| *x));
    for (month_start, month_end, days_in_month) in month_day_counts.iter() {
        let done_count = count_days_in_range(refs.clone(), month_start, month_end);
        let done_count = done_count.min(count as usize) as f64;
        let total_days_in_month = (*month_end - *month_start).num_days() + 1;
        let should_have_done_count =
            (count as f64 * (*days_in_month as f64 / total_days_in_month as f64)).floor() as f64;
        if should_have_done_count == 0.0 {
            fractions.push(1.0);
            continue;
        }

        fractions.push(done_count / should_have_done_count);
    }
    average(&fractions).unwrap_or(1.0)
}

fn average(numbers: &[f64]) -> Option<f64> {
    match numbers.len() {
        0 => None,
        _ => Some(numbers.iter().sum::<f64>() / (numbers.len() as f64)),
    }
}

fn get_week_day_counts(
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> Vec<(NaiveDate, NaiveDate, usize)> {
    let days_to_last_monday = start_date.weekday().number_from_monday() - 1;
    let last_monday = *start_date - Duration::days(days_to_last_monday as i64);
    last_monday
        .iter_weeks()
        .take_while(|date| date <= end_date)
        .map(|date| {
            (
                date,
                end_of_week(&date),
                get_week_day_count(&date, start_date, end_date),
            )
        })
        .collect()
}

fn end_of_week(monday_of_week: &NaiveDate) -> NaiveDate {
    *monday_of_week + Duration::days(6)
}

fn end_of_month(day_in_month: &NaiveDate) -> NaiveDate {
    let this_month = day_in_month.month();
    day_in_month
        .iter_days()
        .take_while(|day| day.month() == this_month)
        .last()
        .unwrap()
}

fn get_week_day_count(
    monday_of_week: &NaiveDate,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> usize {
    assert_eq!(monday_of_week.weekday(), Weekday::Mon);
    let sunday_of_week = end_of_week(monday_of_week);
    count_days_in_range(
        Box::new(
            monday_of_week
                .iter_days()
                .take_while(move |day| *day <= sunday_of_week),
        ),
        start_date,
        end_date,
    )
}

fn get_month_day_counts(
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> Vec<(NaiveDate, NaiveDate, usize)> {
    let days_to_last_first = start_date.day() - 1;
    let last_first = *start_date - Duration::days(days_to_last_first as i64);
    iter_months(&last_first)
        .take_while(|date| date <= end_date)
        .map(|date| {
            (
                date,
                end_of_month(&date),
                get_month_day_count(&date, start_date, end_date),
            )
        })
        .collect()
}

fn iter_months(naive_date: &NaiveDate) -> Box<dyn Iterator<Item = NaiveDate>> {
    Box::new(naive_date.iter_days().filter(|day| day.day() == 1))
}

fn get_month_day_count(
    first_of_month: &NaiveDate,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> usize {
    let last_of_month = first_of_month
        .iter_days()
        .take_while(|day| day.month() == first_of_month.month())
        .last()
        .unwrap();
    count_days_in_range(
        Box::new(
            first_of_month
                .iter_days()
                .take_while(move |day| *day <= last_of_month),
        ),
        start_date,
        end_date,
    )
}

fn count_days_in_range<'a>(
    days: Box<dyn Iterator<Item = NaiveDate> + 'a>,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> usize {
    days.filter(|day| start_date <= day && day <= &end_date)
        .count()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::database::{period::Period, task_data::TaskData};

    use super::{get_done_fraction, get_month_day_count, get_week_day_count, get_week_day_counts};

    #[test]
    fn weekly() {
        let start_date = NaiveDate::from_ymd(1970, 01, 01);
        let end_date = NaiveDate::from_ymd(1970, 01, 10);
        let timestamps = &[start_date, end_date];
        let percentage = get_done_fraction(&Period::Week, 7, timestamps, &start_date, &end_date);
        assert_eq!(percentage, (1.0 / 4.0 + 1.0 / 6.0) / 2.0);
        let timestamps = &[
            NaiveDate::from_ymd(1970, 01, 01),
            NaiveDate::from_ymd(1970, 01, 02),
            NaiveDate::from_ymd(1970, 01, 03),
            NaiveDate::from_ymd(1970, 01, 04),
        ];
        let percentage = get_done_fraction(&Period::Week, 7, timestamps, &start_date, &end_date);
        assert_eq!(percentage, (4.0 / 4.0 + 0.0 / 6.0) / 2.0);
        let timestamps = &[
            NaiveDate::from_ymd(1970, 01, 01),
            NaiveDate::from_ymd(1970, 01, 02),
            NaiveDate::from_ymd(1970, 01, 03),
            NaiveDate::from_ymd(1970, 01, 05),
        ];
        let percentage = get_done_fraction(&Period::Week, 7, timestamps, &start_date, &end_date);
        assert_eq!(percentage, (3.0 / 4.0 + 1.0 / 6.0) / 2.0);
        let timestamps = &[
            NaiveDate::from_ymd(1970, 12, 01),
            NaiveDate::from_ymd(1970, 12, 02),
            NaiveDate::from_ymd(1970, 12, 03),
            NaiveDate::from_ymd(1970, 12, 05),
        ];
        let percentage = get_done_fraction(&Period::Week, 7, timestamps, &start_date, &end_date);
        assert_eq!(percentage, 0.0);
        // Once per week with two broken weeks means it never has to be done
        let timestamps = &[
            NaiveDate::from_ymd(1970, 01, 01),
            NaiveDate::from_ymd(1970, 01, 02),
            NaiveDate::from_ymd(1970, 01, 03),
            NaiveDate::from_ymd(1970, 01, 04),
        ];
        let percentage = get_done_fraction(&Period::Week, 1, timestamps, &start_date, &end_date);
        assert_eq!(percentage, (1.0 + 1.0) / 2.0);
    }

    #[test]
    fn monthly() {
        let start_date = NaiveDate::from_ymd(1970, 01, 01);
        let end_date = NaiveDate::from_ymd(1970, 04, 30);
        let timestamps = &[
            NaiveDate::from_ymd(1970, 01, 01),
            NaiveDate::from_ymd(1970, 02, 01),
        ];
        let percentage = get_done_fraction(&Period::Month, 1, timestamps, &start_date, &end_date);
        assert_eq!(percentage, (1.0 / 1.0 + 1.0 / 1.0 + 2.0 * 0.0 / 1.0) / 4.0);
        let timestamps = &[
            NaiveDate::from_ymd(1970, 01, 01),
            NaiveDate::from_ymd(1970, 01, 02),
            NaiveDate::from_ymd(1970, 01, 03),
            NaiveDate::from_ymd(1970, 01, 04),
        ];
        let percentage = get_done_fraction(&Period::Month, 1, timestamps, &start_date, &end_date);
        assert_eq!(percentage, (1.0 / 1.0 + 3.0 * 0.0 / 1.0) / 4.0);
    }

    #[test]
    fn test_get_week_day_counts() {
        let start_date = NaiveDate::from_ymd(1970, 01, 01);
        let end_date = NaiveDate::from_ymd(1970, 01, 20);
        let week_day_counts = get_week_day_counts(&start_date, &end_date);
        let mut week_day_counts_iter = week_day_counts.iter();
        assert_eq!(
            week_day_counts_iter.next().unwrap(),
            &(
                NaiveDate::from_ymd(1969, 12, 29),
                NaiveDate::from_ymd(1970, 01, 04),
                4
            )
        );
        assert_eq!(
            week_day_counts_iter.next().unwrap(),
            &(
                NaiveDate::from_ymd(1970, 01, 05),
                NaiveDate::from_ymd(1970, 01, 11),
                7
            )
        );
        assert_eq!(
            week_day_counts_iter.next().unwrap(),
            &(
                NaiveDate::from_ymd(1970, 01, 12),
                NaiveDate::from_ymd(1970, 01, 18),
                7
            )
        );
        assert_eq!(
            week_day_counts_iter.next().unwrap(),
            &(
                NaiveDate::from_ymd(1970, 01, 19),
                NaiveDate::from_ymd(1970, 01, 25),
                2
            )
        );
    }

    #[test]
    fn test_get_week_day_count() {
        let date = NaiveDate::from_ymd(1970, 01, 19);
        assert_eq!(
            get_week_day_count(
                &date,
                &NaiveDate::from_ymd(1970, 01, 01),
                &NaiveDate::from_ymd(1970, 01, 31)
            ),
            7
        );
        assert_eq!(
            get_week_day_count(
                &date,
                &NaiveDate::from_ymd(1970, 01, 15),
                &NaiveDate::from_ymd(1970, 01, 21)
            ),
            3
        );
        assert_eq!(
            get_week_day_count(
                &date,
                &NaiveDate::from_ymd(1970, 01, 19),
                &NaiveDate::from_ymd(1970, 01, 19)
            ),
            1
        );
    }

    #[test]
    fn test_get_month_day_count() {
        let date = NaiveDate::from_ymd(1970, 01, 01);
        assert_eq!(
            get_month_day_count(
                &date,
                &NaiveDate::from_ymd(1969, 12, 01),
                &NaiveDate::from_ymd(1970, 01, 31)
            ),
            31
        );
        assert_eq!(
            get_month_day_count(
                &date,
                &NaiveDate::from_ymd(1970, 01, 01),
                &NaiveDate::from_ymd(1970, 01, 15)
            ),
            15
        );
        assert_eq!(
            get_month_day_count(
                &date,
                &NaiveDate::from_ymd(1970, 01, 07),
                &NaiveDate::from_ymd(1970, 01, 15)
            ),
            9
        );
    }
}

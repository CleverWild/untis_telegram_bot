use chrono::{Datelike, NaiveDate};

use crate::diff_impl::Diff;

pub fn next_friday(from: NaiveDate) -> NaiveDate {
    let days_to_next_friday = (11 - from.weekday().num_days_from_monday()) % 7;
    from + chrono::Duration::days(days_to_next_friday as i64)
}

pub fn sort_diffs(diffs: &mut Vec<Diff>) {
    diffs.sort_by(|l, r| {
        l.date()
            .cmp(&r.date())
            .then_with(|| l.start_time().cmp(&r.start_time()))
            .then_with(|| l.end_time().cmp(&r.end_time()))
            .then_with(|| l.code().cmp(r.code()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_friday_test() {
        let date = NaiveDate::from_ymd_opt(2025, 5, 27).unwrap(); // Tuesday
        let next_friday_date = next_friday(date);
        assert_eq!(
            next_friday_date,
            NaiveDate::from_ymd_opt(2025, 5, 30).unwrap() // Friday
        );
    }
}

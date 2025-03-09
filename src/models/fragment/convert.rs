use chrono::NaiveDate;

use super::DateFragments;

pub enum Role {
    Start,
    Due,
    /// Whether is done.
    Completed(bool),
}

impl DateFragments {
    /// Convert fragments into complete date according to the date role.
    pub fn to_date(
        relative: &Option<DateFragments>,
        base: &Option<DateFragments>,
        role: Role,
    ) -> Option<NaiveDate> {
        // convenience functions
        let from_ymd = |y: &u32, m: &u32, d: &u32| match role {
            Role::Completed(false) => None,
            _ => NaiveDate::from_ymd_opt(*y as i32, *m, *d),
        };

        let from_ym = |y: &u32, m: &u32| {
            let day = match role {
                Role::Completed(false) => return None,
                Role::Start => 1,
                Role::Due | Role::Completed(true) => last_day_of_month(*y, *m)?,
            };
            NaiveDate::from_ymd_opt(*y as i32, *m, day)
        };
        let from_y = |y: &u32| {
            let (month, day) = match role {
                Role::Completed(false) => return None,
                Role::Start => (1, 1),
                Role::Due | Role::Completed(true) => (12, 31),
            };
            NaiveDate::from_ymd_opt(*y as i32, month, day)
        };

        use DateFragments::*;
        match (relative, base) {
            // both none
            (None, None) => None,

            // either one
            (None, Some(either)) | (Some(either), None) => match either {
                YearMonthDay(y, m, d) => from_ymd(y, m, d),
                YearMonth(y, m) => from_ym(y, m),
                Year(y) => from_y(y),
                // no year
                MonthDay(_, _) | MonthOrDay(_) => None,
            },

            // both some: relative overwrites base
            (Some(relative), Some(base)) => {
                match (relative, base) {
                    // ym
                    (YearMonthDay(y, m, d), _)
                    | (YearMonth(y, m), MonthDay(_, d))
                    | (YearMonth(y, m), MonthOrDay(d))
                    | (Year(y), MonthDay(m, d))
                    | (YearMonth(y, m), YearMonthDay(_, _, d))
                    | (MonthDay(m, d), YearMonthDay(y, _, _))
                    | (MonthDay(m, d), YearMonth(y, _))
                    | (MonthDay(m, d), Year(y))
                    | (Year(y), YearMonthDay(_, m, d))
                    | (MonthOrDay(d), YearMonthDay(y, m, _))
                    | (MonthOrDay(d), YearMonth(y, m)) => from_ymd(y, m, d),
                    // ym
                    (Year(y), MonthOrDay(m))
                    | (YearMonth(y, m), YearMonth(_, _))
                    | (YearMonth(y, m), Year(_))
                    | (Year(y), YearMonth(_, m))
                    | (MonthOrDay(m), Year(y)) => from_ym(y, m),
                    // y
                    (Year(y), Year(_)) => from_y(y),
                    // no year
                    (MonthDay(_, _), MonthDay(_, _))
                    | (MonthDay(_, _), MonthOrDay(_))
                    | (MonthOrDay(_), MonthDay(_, _))
                    | (MonthOrDay(_), MonthOrDay(_)) => None,
                }
            }
        }
    }
}

/// Get last day of month.
///
/// Return `None` when month is not in `1..=12`.
fn last_day_of_month(year: u32, month: u32) -> Option<u32> {
    let leap = year % 4 == 0;
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        // invalid month
        _ => return None,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_convert_no_base {
        ($test_name:ident: $relative:expr, $role:expr, $expected:expr) => {
            #[test]
            fn $test_name() {
                let role = $role;
                let relative = $relative;
                let expected = $expected;
                assert_eq!(
                    DateFragments::to_date(&Some(relative), &None, role),
                    expected
                );
            }
        };
    }

    // MARK: test done/undone

    test_convert_no_base!(
        completed_undone:
        DateFragments::YearMonthDay(2025, 3, 1),
        Role::Completed(false),
        None
    );

    test_convert_no_base!(
        completed_done:
        DateFragments::YearMonthDay(2025, 3, 1),
        Role::Completed(true),
        NaiveDate::from_ymd_opt(2025, 3, 1)
    );

    // MARK: test year month day

    test_convert_no_base!(
        start_year_month_day:
        DateFragments::YearMonthDay(2025, 3, 1),
        Role::Start,
        NaiveDate::from_ymd_opt(2025, 3, 1)
    );

    test_convert_no_base!(
        due_year_month_day:
        DateFragments::YearMonthDay(2025, 3, 1),
        Role::Due,
        NaiveDate::from_ymd_opt(2025, 3, 1)
    );

    // MARK: test year month

    test_convert_no_base!(
        start_year_month:
        DateFragments::YearMonth(2025, 3),
        Role::Start,
        NaiveDate::from_ymd_opt(2025, 3, 1)
    );

    test_convert_no_base!(
        due_year_month:
        DateFragments::YearMonth(2025, 3),
        Role::Due,
        NaiveDate::from_ymd_opt(2025, 3, 31)
    );

    test_convert_no_base!(
        due_year_month_april:
        DateFragments::YearMonth(2025, 2),
        Role::Due,
        NaiveDate::from_ymd_opt(2025, 2, 28)
    );

    test_convert_no_base!(
        due_year_month_february:
        DateFragments::YearMonth(2025, 2),
        Role::Due,
        NaiveDate::from_ymd_opt(2025, 2, 28)
    );

    test_convert_no_base!(
        due_year_month_february_leap:
        DateFragments::YearMonth(2024, 2),
        Role::Due,
        NaiveDate::from_ymd_opt(2024, 2, 29)
    );

    // MARK: test year

    test_convert_no_base!(
        start_year:
        DateFragments::Year(2025),
        Role::Start,
        NaiveDate::from_ymd_opt(2025, 1, 1)
    );

    test_convert_no_base!(
        due_year:
        DateFragments::Year(2025),
        Role::Due,
        NaiveDate::from_ymd_opt(2025, 12, 31)
    );

    // TODO: cover base in test
}

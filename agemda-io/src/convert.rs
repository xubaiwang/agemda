use chrono::{DateTime, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};

use crate::fragment::DateTimeFragment;

pub enum Role {
    Start,
    End,
}

impl Role {
    pub fn of_year(&self, year: i32) -> Option<DateTime<Local>> {
        let year = match self {
            Role::Start => year,
            Role::End => year + 1,
        };
        let date = NaiveDate::from_ymd_opt(year, 01, 01)?;
        let time = NaiveTime::from_hms_opt(0, 0, 0)?;
        NaiveDateTime::new(date, time)
            .and_local_timezone(Local)
            .single()
    }

    pub fn of_month(&self, year: i32, month: u32) -> Option<DateTime<Local>> {
        let date = NaiveDate::from_ymd_opt(year, month, 01)?;
        let time = NaiveTime::from_hms_opt(0, 0, 0)?;
        let start = NaiveDateTime::new(date, time)
            .and_local_timezone(Local)
            .single()?;
        match self {
            Role::Start => Some(start),
            Role::End => start.checked_add_months(Months::new(1)),
        }
    }

    pub fn of_day(&self, year: i32, month: u32, day: u32) -> Option<DateTime<Local>> {
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_opt(0, 0, 0)?;
        let start = NaiveDateTime::new(date, time)
            .and_local_timezone(Local)
            .single()?;
        match self {
            Role::Start => Some(start),
            Role::End => start.checked_add_days(Days::new(1)),
        }
    }

    pub fn of_hour(&self, year: i32, month: u32, day: u32, hour: u32) -> Option<DateTime<Local>> {
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_opt(hour, 0, 0)?;
        let start = NaiveDateTime::new(date, time)
            .and_local_timezone(Local)
            .single()?;
        match self {
            Role::Start => Some(start),
            Role::End => start.checked_add_signed(TimeDelta::hours(1)),
        }
    }

    pub fn of_minute(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
    ) -> Option<DateTime<Local>> {
        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_opt(hour, minute, 0)?;
        let start = NaiveDateTime::new(date, time)
            .and_local_timezone(Local)
            .single()?;
        match self {
            Role::Start => Some(start),
            Role::End => start.checked_add_signed(TimeDelta::minutes(1)),
        }
    }
}

pub fn of_second(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DateTime<Local>> {
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    NaiveDateTime::new(date, time)
        .and_local_timezone(Local)
        .single()
}

pub fn fragment_to_datetime(
    relative: &Option<DateTimeFragment>,
    base: &Option<DateTimeFragment>,
    role: Role,
) -> Option<DateTime<Local>> {
    match (relative, base) {
        (None, None) => None,
        (None, Some(either)) | (Some(either), None) => fragment_to_datetime_either(either, role),
        (Some(relative), Some(base)) => fragment_to_datetime_both(relative, base, role),
    }
}

pub fn fragment_to_datetime_either(
    either: &DateTimeFragment,
    role: Role,
) -> Option<DateTime<Local>> {
    let year = either.year()?;
    let Some(month) = either.month() else {
        return role.of_year(year);
    };
    let Some(day) = either.day() else {
        return role.of_month(year, month);
    };
    let Some(hour) = either.hour() else {
        return role.of_day(year, month, day);
    };
    let Some(minute) = either.minute() else {
        return role.of_hour(year, month, day, hour);
    };
    let Some(second) = either.second() else {
        return role.of_minute(year, month, day, hour, minute);
    };
    of_second(year, month, day, hour, minute, second)
}

pub fn fragment_to_datetime_both(
    relative: &DateTimeFragment,
    base: &DateTimeFragment,
    role: Role,
) -> Option<DateTime<Local>> {
    let year = relative.year().or(base.year())?;
    let Some(month) = relative.month().or(base.month()) else {
        return role.of_year(year);
    };
    let Some(day) = relative.day().or(base.day()) else {
        return role.of_month(year, month);
    };
    let Some(hour) = relative.hour().or(base.hour()) else {
        return role.of_day(year, month, day);
    };
    let Some(minute) = relative.minute().or(base.minute()) else {
        return role.of_hour(year, month, day, hour);
    };
    let Some(second) = relative.second().or(base.second()) else {
        return role.of_minute(year, month, day, hour, minute);
    };
    of_second(year, month, day, hour, minute, second)
}

#[cfg(test)]
pub mod test {
    use super::*;

    macro_rules! case {
        ($name:ident, $relative:expr, $base:expr, $start:expr, $end:expr $(,)?) => {
            mod $name {
                use super::*;

                #[test]
                pub fn start() {
                    let result = fragment_to_datetime(&$relative, &$base, Role::Start);
                    assert_eq!(result, $start);
                }

                #[test]
                pub fn end() {
                    let result = fragment_to_datetime(&$relative, &$base, Role::End);
                    assert_eq!(result, $end);
                }
            }
        };
    }

    case!(both_none, None, None, None, None);
    // relative only
    case!(
        relative_year,
        Some(DateTimeFragment::from_y(2025)),
        None,
        of_second(2025, 01, 01, 00, 00, 00),
        of_second(2026, 01, 01, 00, 00, 00),
    );
    case!(
        relative_year_month,
        Some(DateTimeFragment::from_ym(2025, 02)),
        None,
        of_second(2025, 02, 01, 00, 00, 00),
        of_second(2025, 03, 01, 00, 00, 00),
    );
    case!(
        relative_year_month_carry,
        Some(DateTimeFragment::from_ym(2025, 12)),
        None,
        of_second(2025, 12, 01, 00, 00, 00),
        of_second(2026, 01, 01, 00, 00, 00),
    );
    case!(
        relative_year_month_day,
        Some(DateTimeFragment::from_ymd(2025, 02, 03)),
        None,
        of_second(2025, 02, 03, 00, 00, 00),
        of_second(2025, 02, 04, 00, 00, 00),
    );
    case!(
        relative_year_month_day_carry,
        Some(DateTimeFragment::from_ymd(2025, 03, 31)),
        None,
        of_second(2025, 03, 31, 00, 00, 00),
        of_second(2025, 04, 01, 00, 00, 00),
    );
    case!(
        relative_year_month_day_hour,
        Some(DateTimeFragment::from_ymd_h(2025, 02, 03, 12)),
        None,
        of_second(2025, 02, 03, 12, 00, 00),
        of_second(2025, 02, 03, 13, 00, 00),
    );
    case!(
        relative_year_month_day_hour_carry,
        Some(DateTimeFragment::from_ymd_h(2025, 02, 03, 23)),
        None,
        of_second(2025, 02, 03, 23, 00, 00),
        of_second(2025, 02, 04, 00, 00, 00),
    );
    case!(
        relative_year_month_day_hour_minute,
        Some(DateTimeFragment::from_ymd_hm(2025, 02, 03, 12, 31)),
        None,
        of_second(2025, 02, 03, 12, 31, 00),
        of_second(2025, 02, 03, 12, 32, 00),
    );
    case!(
        relative_year_month_day_hour_minute_carry,
        Some(DateTimeFragment::from_ymd_hm(2025, 02, 03, 12, 59)),
        None,
        of_second(2025, 02, 03, 12, 59, 00),
        of_second(2025, 02, 03, 13, 00, 00),
    );
    case!(
        relative_year_month_day_hour_minute_second,
        Some(DateTimeFragment::from_ymd_hms(2025, 02, 03, 12, 31, 50)),
        None,
        of_second(2025, 02, 03, 12, 31, 50),
        of_second(2025, 02, 03, 12, 31, 50),
    );
    // relative month
    // TODO: test more relative without year
    case!(
        relative_month,
        Some(DateTimeFragment::from_m(02)),
        None,
        None,
        None
    );

    // both
    // TODO: more both test case
    case!(
        both_relative_year_base_year,
        Some(DateTimeFragment::from_y(2025)),
        Some(DateTimeFragment::from_y(2024)),
        of_second(2025, 01, 01, 00, 00, 00),
        of_second(2026, 01, 01, 00, 00, 00),
    );
    case!(
        both_relative_year_base_day,
        Some(DateTimeFragment::from_y(2025)),
        Some(DateTimeFragment::from_d(12)),
        of_second(2025, 01, 01, 00, 00, 00),
        of_second(2026, 01, 01, 00, 00, 00),
    );
}

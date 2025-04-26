use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{eof, not, opt, peek},
    sequence::{preceded, terminated},
};

#[derive(Clone, Debug, PartialEq)]
pub struct MinuteRest {
    pub minute: u32,
    pub rest: Option<u32>,
}

impl MinuteRest {
    pub fn from_m(minute: u32) -> Self {
        Self { minute, rest: None }
    }

    pub fn from_ms(minute: u32, second: u32) -> Self {
        Self {
            minute,
            rest: Some(second),
        }
    }

    pub fn minute(&self) -> u32 {
        self.minute
    }

    pub fn second(&self) -> Option<u32> {
        self.rest
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HourRest {
    pub hour: u32,
    pub rest: Option<MinuteRest>,
}

impl HourRest {
    pub fn from_h(hour: u32) -> Self {
        Self { hour, rest: None }
    }

    pub fn from_hm(hour: u32, minute: u32) -> Self {
        Self {
            hour,
            rest: Some(MinuteRest::from_m(minute)),
        }
    }

    pub fn from_hms(hour: u32, minute: u32, second: u32) -> Self {
        Self {
            hour,
            rest: Some(MinuteRest::from_ms(minute, second)),
        }
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }

    pub fn minute(&self) -> Option<u32> {
        self.rest.as_ref().map(|r| r.minute())
    }

    pub fn second(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.second())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DayRest {
    pub day: u32,
    pub rest: Option<HourRest>,
}

impl DayRest {
    pub fn from_d(day: u32) -> Self {
        Self { day, rest: None }
    }

    pub fn from_d_h(day: u32, hour: u32) -> Self {
        Self {
            day,
            rest: Some(HourRest::from_h(hour)),
        }
    }

    pub fn from_d_hm(day: u32, hour: u32, minute: u32) -> Self {
        Self {
            day,
            rest: Some(HourRest::from_hm(hour, minute)),
        }
    }

    pub fn from_d_hms(day: u32, hour: u32, minute: u32, second: u32) -> Self {
        Self {
            day,
            rest: Some(HourRest::from_hms(hour, minute, second)),
        }
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    pub fn hour(&self) -> Option<u32> {
        self.rest.as_ref().map(|r| r.hour())
    }

    pub fn minute(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.minute())
    }

    pub fn second(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.second())
    }
}

// TODO: month distinguish
#[derive(Clone, Debug, PartialEq)]
pub struct MonthRest {
    pub month: u32,
    pub rest: Option<DayRest>,
}

impl MonthRest {
    pub fn from_m(month: u32) -> Self {
        Self { month, rest: None }
    }

    pub fn from_md(month: u32, day: u32) -> Self {
        Self {
            month,
            rest: Some(DayRest::from_d(day)),
        }
    }

    pub fn from_md_h(month: u32, day: u32, hour: u32) -> Self {
        Self {
            month,
            rest: Some(DayRest::from_d_h(day, hour)),
        }
    }

    pub fn from_md_hm(month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self {
            month,
            rest: Some(DayRest::from_d_hm(day, hour, minute)),
        }
    }

    pub fn from_md_hms(month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        Self {
            month,
            rest: Some(DayRest::from_d_hms(day, hour, minute, second)),
        }
    }

    pub fn month(&self) -> u32 {
        self.month
    }

    pub fn day(&self) -> Option<u32> {
        self.rest.as_ref().map(|r| r.day())
    }

    pub fn hour(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.hour())
    }

    pub fn minute(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.minute())
    }

    pub fn second(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.second())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct YearRest {
    pub year: i32,
    pub rest: Option<MonthRest>,
}

impl YearRest {
    pub fn from_y(year: i32) -> Self {
        Self { year, rest: None }
    }

    pub fn from_ym(year: i32, month: u32) -> Self {
        Self {
            year,
            rest: Some(MonthRest::from_m(month)),
        }
    }

    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self {
            year,
            rest: Some(MonthRest::from_md(month, day)),
        }
    }

    pub fn from_ymd_h(year: i32, month: u32, day: u32, hour: u32) -> Self {
        Self {
            year,
            rest: Some(MonthRest::from_md_h(month, day, hour)),
        }
    }

    pub fn from_ymd_hm(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self {
            year,
            rest: Some(MonthRest::from_md_hm(month, day, hour, minute)),
        }
    }

    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        Self {
            year,
            rest: Some(MonthRest::from_md_hms(month, day, hour, minute, second)),
        }
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> Option<u32> {
        self.rest.as_ref().map(|r| r.month())
    }

    pub fn day(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.day())
    }

    pub fn hour(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.hour())
    }

    pub fn minute(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.minute())
    }

    pub fn second(&self) -> Option<u32> {
        self.rest.as_ref().and_then(|r| r.second())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DateTimeFragment {
    YearRest(YearRest),
    MonthOrDay(u32),
    MonthRest(MonthRest),
    DayRest(DayRest),
    HourRest(Option<HourRest>),
}

impl DateTimeFragment {
    pub fn from_y(year: i32) -> Self {
        Self::YearRest(YearRest::from_y(year))
    }

    pub fn from_ym(year: i32, month: u32) -> Self {
        Self::YearRest(YearRest::from_ym(year, month))
    }

    pub fn from_ymd(year: i32, month: u32, day: u32) -> Self {
        Self::YearRest(YearRest::from_ymd(year, month, day))
    }

    pub fn from_ymd_h(year: i32, month: u32, day: u32, hour: u32) -> Self {
        Self::YearRest(YearRest::from_ymd_h(year, month, day, hour))
    }

    pub fn from_ymd_hm(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self::YearRest(YearRest::from_ymd_hm(year, month, day, hour, minute))
    }

    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        Self::YearRest(YearRest::from_ymd_hms(
            year, month, day, hour, minute, second,
        ))
    }

    pub fn from_m_d(month_or_day: u32) -> Self {
        Self::MonthOrDay(month_or_day)
    }

    pub fn from_m(month: u32) -> Self {
        Self::MonthRest(MonthRest::from_m(month))
    }

    pub fn from_md(month: u32, day: u32) -> Self {
        Self::MonthRest(MonthRest::from_md(month, day))
    }

    pub fn from_md_h(month: u32, day: u32, hour: u32) -> Self {
        Self::MonthRest(MonthRest::from_md_h(month, day, hour))
    }

    pub fn from_md_hm(month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self::MonthRest(MonthRest::from_md_hm(month, day, hour, minute))
    }

    pub fn from_md_hms(month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        Self::MonthRest(MonthRest::from_md_hms(month, day, hour, minute, second))
    }

    pub fn from_d(day: u32) -> Self {
        Self::DayRest(DayRest::from_d(day))
    }

    pub fn from_d_h(day: u32, hour: u32) -> Self {
        Self::DayRest(DayRest::from_d_h(day, hour))
    }

    pub fn from_d_hm(day: u32, hour: u32, minute: u32) -> Self {
        Self::DayRest(DayRest::from_d_hm(day, hour, minute))
    }

    pub fn from_d_hms(day: u32, hour: u32, minute: u32, second: u32) -> Self {
        Self::DayRest(DayRest::from_d_hms(day, hour, minute, second))
    }

    pub fn from_h(hour: u32) -> Self {
        Self::HourRest(Some(HourRest::from_h(hour)))
    }

    pub fn from_hm(hour: u32, minute: u32) -> Self {
        Self::HourRest(Some(HourRest::from_hm(hour, minute)))
    }

    pub fn from_hms(hour: u32, minute: u32, second: u32) -> Self {
        Self::HourRest(Some(HourRest::from_hms(hour, minute, second)))
    }

    pub fn year(&self) -> Option<i32> {
        match self {
            DateTimeFragment::YearRest(year_rest) => Some(year_rest.year()),
            _ => None,
        }
    }

    pub fn month(&self) -> Option<u32> {
        match self {
            DateTimeFragment::YearRest(year_rest) => year_rest.month(),
            DateTimeFragment::MonthRest(month_rest) => Some(month_rest.month()),
            _ => None,
        }
    }

    pub fn day(&self) -> Option<u32> {
        match self {
            DateTimeFragment::YearRest(year_rest) => year_rest.day(),
            DateTimeFragment::MonthRest(month_rest) => month_rest.day(),
            DateTimeFragment::DayRest(day_rest) => Some(day_rest.day()),
            _ => None,
        }
    }

    pub fn hour(&self) -> Option<u32> {
        match self {
            DateTimeFragment::YearRest(year_rest) => year_rest.hour(),
            DateTimeFragment::MonthRest(month_rest) => month_rest.hour(),
            DateTimeFragment::DayRest(day_rest) => day_rest.hour(),
            DateTimeFragment::HourRest(hour_rest) => hour_rest.as_ref().map(|r| r.hour()),
            _ => None,
        }
    }

    pub fn minute(&self) -> Option<u32> {
        match self {
            DateTimeFragment::YearRest(year_rest) => year_rest.minute(),
            DateTimeFragment::MonthRest(month_rest) => month_rest.minute(),
            DateTimeFragment::DayRest(day_rest) => day_rest.minute(),
            DateTimeFragment::HourRest(hour_rest) => hour_rest.as_ref().and_then(|r| r.minute()),
            _ => None,
        }
    }

    pub fn second(&self) -> Option<u32> {
        match self {
            DateTimeFragment::YearRest(year_rest) => year_rest.second(),
            DateTimeFragment::MonthRest(month_rest) => month_rest.second(),
            DateTimeFragment::DayRest(day_rest) => day_rest.second(),
            DateTimeFragment::HourRest(hour_rest) => hour_rest.as_ref().and_then(|r| r.second()),
            _ => None,
        }
    }
}

pub fn date_time_fragment(input: &str) -> IResult<&str, DateTimeFragment> {
    use DateTimeFragment::*;

    alt((
        year_rest.map(YearRest),
        month_or_day.map(MonthOrDay),
        month_rest.map(MonthRest),
        day_rest.map(DayRest),
        hour_rest.map(HourRest),
    ))
    .parse(input)
}

pub fn year_rest(input: &str) -> IResult<&str, YearRest> {
    let (input, year) = four_digits(input)?;
    let (input, rest) = opt(preceded(hyphen, opt(month_rest))).parse(input)?;
    Ok((
        input,
        YearRest {
            year,
            rest: rest.flatten(),
        },
    ))
}

pub fn month_or_day(input: &str) -> IResult<&str, u32> {
    // XXX: should be neither T or -
    terminated(two_digits, eof).parse(input)
}

pub fn month_rest(input: &str) -> IResult<&str, MonthRest> {
    let (input, month) = two_digits(input)?;
    let (input, rest) = opt(preceded(hyphen, opt(day_rest))).parse(input)?;
    not(peek(cap_t)).parse(input)?;
    Ok((
        input,
        MonthRest {
            month,
            rest: rest.flatten(),
        },
    ))
}

pub fn day_rest(input: &str) -> IResult<&str, DayRest> {
    let (input, day) = two_digits(input)?;
    let (input, rest) = opt(hour_rest).parse(input)?;
    Ok((
        input,
        DayRest {
            day,
            rest: rest.flatten(),
        },
    ))
}

pub fn hour_rest(input: &str) -> IResult<&str, Option<HourRest>> {
    let (input, _) = cap_t(input)?;
    let (input, res) = opt((two_digits, opt(preceded(colon, opt(minute_rest))))).parse(input)?;
    Ok((
        input,
        res.map(|(hour, rest)| HourRest {
            hour,
            rest: rest.flatten(),
        }),
    ))
}

pub fn minute_rest(input: &str) -> IResult<&str, MinuteRest> {
    let (input, minute) = two_digits(input)?;
    let (input, rest) = opt(preceded(colon, opt(two_digits))).parse(input)?;
    Ok((
        input,
        MinuteRest {
            minute,
            rest: rest.flatten(),
        },
    ))
}

pub fn four_digits(input: &str) -> IResult<&str, i32> {
    take(4u8).map_res(|x: &str| x.parse::<i32>()).parse(input)
}

pub fn two_digits(input: &str) -> IResult<&str, u32> {
    take(2u8).map_res(|x: &str| x.parse::<u32>()).parse(input)
}

pub fn hyphen(input: &str) -> IResult<&str, &str> {
    tag("-").parse(input)
}

pub fn colon(input: &str) -> IResult<&str, &str> {
    tag(":").parse(input)
}

pub fn cap_t(input: &str) -> IResult<&str, &str> {
    tag("T").parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! parse_ok {
        ($parser:expr, $name:ident, $input:literal, $expected:expr) => {
            #[test]
            pub fn $name() {
                use nom::Parser;
                let result = $parser.parse($input);
                assert_eq!(result, Ok(("", $expected)));
            }
        };
    }

    macro_rules! parse_err {
        ($parser:expr, $name:ident, $input:literal) => {
            #[test]
            pub fn $name() {
                use nom::Parser;
                let result = $parser.parse($input);
                assert!(matches!(result, Err(_)));
            }
        };
    }

    mod hour_rest {
        use super::*;

        parse_ok!(hour_rest, t, "T", None);
        parse_ok!(hour_rest, t_hour, "T21", Some(HourRest::from_h(21)));
        parse_ok!(hour_rest, t_hour_c, "T21:", Some(HourRest::from_h(21)));
        parse_ok!(
            hour_rest,
            t_hour_c_minute,
            "T21:00",
            Some(HourRest::from_hm(21, 00))
        );
        parse_ok!(
            hour_rest,
            t_hour_c_minute_c,
            "T21:00:",
            Some(HourRest::from_hm(21, 00))
        );
        parse_ok!(
            hour_rest,
            t_hour_c_minute_c_second,
            "T21:00:05",
            Some(HourRest::from_hms(21, 00, 05))
        );
    }

    mod day_rest {
        use super::*;

        parse_ok!(day_rest, day, "21", DayRest::from_d(21));
        parse_ok!(day_rest, day_t, "21T", DayRest::from_d(21));
        parse_ok!(
            day_rest,
            day_t_time,
            "21T10:04:25",
            DayRest::from_d_hms(21, 10, 04, 25)
        );
    }

    mod month_rest {
        use super::*;

        parse_ok!(month_rest, month, "10", MonthRest::from_m(10));
        parse_ok!(month_rest, month_h, "10-", MonthRest::from_m(10));
        parse_ok!(month_rest, month_h_day, "10-21", MonthRest::from_md(10, 21));
        parse_ok!(
            month_rest,
            month_h_day_time,
            "10-21T10:04:25",
            MonthRest::from_md_hms(10, 21, 10, 04, 25)
        );
    }

    mod month_or_day {
        use super::*;

        parse_ok!(month_or_day, ok, "10", 10);
        parse_err!(month_or_day, err_h, "10-");
        parse_err!(month_or_day, err_t, "10T");
    }

    mod year_rest {
        use super::*;

        parse_ok!(year_rest, year, "2025", YearRest::from_y(2025));
        parse_ok!(year_rest, year_h, "2025-", YearRest::from_y(2025));
        parse_ok!(
            year_rest,
            year_h_month,
            "2025-10",
            YearRest::from_ym(2025, 10)
        );
        parse_ok!(
            year_rest,
            year_h_month_rest,
            "2025-10-01T06:00:00",
            YearRest::from_ymd_hms(2025, 10, 01, 06, 00, 00)
        );
    }

    mod date_time_fragment {
        use super::*;

        // year
        parse_ok!(
            date_time_fragment,
            year,
            "2025",
            DateTimeFragment::from_y(2025)
        );
        parse_ok!(
            date_time_fragment,
            year_h,
            "2025-",
            DateTimeFragment::from_y(2025)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month,
            "2025-10",
            DateTimeFragment::from_ym(2025, 10)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h,
            "2025-10-",
            DateTimeFragment::from_ym(2025, 10)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day,
            "2025-10-01",
            DateTimeFragment::from_ymd(2025, 10, 01)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day_t,
            "2025-10-01T",
            DateTimeFragment::from_ymd(2025, 10, 01)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day_t_hour,
            "2025-10-01T10",
            DateTimeFragment::from_ymd_h(2025, 10, 01, 10)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day_t_hour_c,
            "2025-10-01T10:",
            DateTimeFragment::from_ymd_h(2025, 10, 01, 10)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day_t_hour_c_minute,
            "2025-10-01T10:20",
            DateTimeFragment::from_ymd_hm(2025, 10, 01, 10, 20)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day_t_hour_c_minute_c,
            "2025-10-01T10:20:",
            DateTimeFragment::from_ymd_hm(2025, 10, 01, 10, 20)
        );
        parse_ok!(
            date_time_fragment,
            year_h_month_h_day_t_hour_c_minute_c_second,
            "2025-10-01T10:20:25",
            DateTimeFragment::from_ymd_hms(2025, 10, 01, 10, 20, 25)
        );
        // month or day
        parse_ok!(
            date_time_fragment,
            month_or_day,
            "10",
            DateTimeFragment::from_m_d(10)
        );
        // month
        parse_ok!(
            date_time_fragment,
            month,
            "10-",
            DateTimeFragment::from_m(10)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day,
            "10-01",
            DateTimeFragment::from_md(10, 01)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day_t,
            "10-01T",
            DateTimeFragment::from_md(10, 01)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day_t_hour,
            "10-01T10",
            DateTimeFragment::from_md_h(10, 01, 10)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day_t_hour_c,
            "10-01T10:",
            DateTimeFragment::from_md_h(10, 01, 10)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day_t_hour_c_minute,
            "10-01T10:20",
            DateTimeFragment::from_md_hm(10, 01, 10, 20)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day_t_hour_c_minute_c,
            "10-01T10:20:",
            DateTimeFragment::from_md_hm(10, 01, 10, 20)
        );
        parse_ok!(
            date_time_fragment,
            month_h_day_t_hour_c_minute_c_second,
            "10-01T10:20:30",
            DateTimeFragment::from_md_hms(10, 01, 10, 20, 30)
        );
        // day
        parse_ok!(
            date_time_fragment,
            day_t,
            "01T",
            DateTimeFragment::from_d(01)
        );
        parse_ok!(
            date_time_fragment,
            day_t_hour,
            "01T10",
            DateTimeFragment::from_d_h(01, 10)
        );
        parse_ok!(
            date_time_fragment,
            day_t_hour_c,
            "01T10:",
            DateTimeFragment::from_d_h(01, 10)
        );
        parse_ok!(
            date_time_fragment,
            day_t_hour_c_minute,
            "01T10:20",
            DateTimeFragment::from_d_hm(01, 10, 20)
        );
        parse_ok!(
            date_time_fragment,
            day_t_hour_c_minute_c,
            "01T10:20:",
            DateTimeFragment::from_d_hm(01, 10, 20)
        );
        parse_ok!(
            date_time_fragment,
            day_t_hour_c_minute_c_second,
            "01T10:20:30",
            DateTimeFragment::from_d_hms(01, 10, 20, 30)
        );
        // time
        parse_ok!(date_time_fragment, t, "T", DateTimeFragment::HourRest(None));
        parse_ok!(
            date_time_fragment,
            t_hour,
            "T10",
            DateTimeFragment::from_h(10)
        );
        parse_ok!(
            date_time_fragment,
            t_hour_c,
            "T10:",
            DateTimeFragment::from_h(10)
        );
        parse_ok!(
            date_time_fragment,
            t_hour_c_minute,
            "T10:20",
            DateTimeFragment::from_hm(10, 20)
        );
        parse_ok!(
            date_time_fragment,
            t_hour_c_minute_c,
            "T10:20:",
            DateTimeFragment::from_hm(10, 20)
        );
        parse_ok!(
            date_time_fragment,
            t_hour_c_minute_c_second,
            "T10:20:30",
            DateTimeFragment::from_hms(10, 20, 30)
        );
    }
}

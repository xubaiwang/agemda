use chrono::NaiveDate;

/// Part of the date.
#[derive(Debug)]
pub enum DateFragment {
    /// Year, month and day, `yyyy-mm-dd`.
    Ymd(u32, u32, u32),
    /// Year and month, `yyyy-mm`.
    Ym(u32, u32),
    /// Month and day, `mm-dd`.
    Md(u32, u32),
    /// Year only, `yyyy`.
    Y(u32),
    /// Month or day.
    ///
    /// As month and day are both of 2 digits,
    /// they cannot be distinguished.
    ///
    /// The intepretation of this arm is context dependent.
    /// When `base` is `Ymd` or `Ym`, day.
    /// When `base` is `Y`, month.
    Mod(u32),
}

impl DateFragment {
    pub fn to_date(
        relative: &Option<DateFragment>,
        base: &Option<DateFragment>,
        last: bool,
    ) -> Option<NaiveDate> {
        // convenience functions
        let from_ymd = |y: &u32, m: &u32, d: &u32| NaiveDate::from_ymd_opt(*y as i32, *m, *d);
        let from_ym = |y: &u32, m: &u32| {
            NaiveDate::from_ymd_opt(
                *y as i32,
                *m,
                if last { last_day_of_month(*y, *m)? } else { 1 },
            )
        };
        let from_y = |y: &u32| {
            NaiveDate::from_ymd_opt(
                *y as i32,
                if last { 12 } else { 1 },
                if last { 31 } else { 1 },
            )
        };

        use DateFragment::*;
        match (relative, base) {
            // both none
            (None, None) => None,
            // either one
            (None, Some(either)) | (Some(either), None) => match either {
                Ymd(y, m, d) => from_ymd(y, m, d),
                Ym(y, m) => from_ym(y, m),
                Y(y) => from_y(y),
                // no year
                Md(_, _) | Mod(_) => None,
            },
            // both some
            (Some(relative), Some(base)) => {
                match (relative, base) {
                    // ymd
                    (Ymd(y, m, d), _)
                    | (Ym(y, m), Md(_, d))
                    | (Ym(y, m), Mod(d))
                    | (Y(y), Md(m, d))
                    | (Ym(y, m), Ymd(_, _, d))
                    | (Md(m, d), Ymd(y, _, _))
                    | (Md(m, d), Ym(y, _))
                    | (Md(m, d), Y(y))
                    | (Y(y), Ymd(_, m, d))
                    | (Mod(d), Ymd(y, m, _))
                    | (Mod(d), Ym(y, m)) => from_ymd(y, m, d),
                    // ym
                    (Y(y), Mod(m))
                    | (Ym(y, m), Ym(_, _))
                    | (Ym(y, m), Y(_))
                    | (Y(y), Ym(_, m))
                    | (Mod(m), Y(y)) => from_ym(y, m),
                    // y
                    (Y(y), Y(_)) => from_y(y),
                    // no year
                    (Md(_, _), Md(_, _))
                    | (Md(_, _), Mod(_))
                    | (Mod(_), Md(_, _))
                    | (Mod(_), Mod(_)) => None,
                }
            }
        }
    }
}

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

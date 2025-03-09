pub mod convert;
pub mod parse;

/// Parts of the date.
#[derive(Debug, PartialEq)]
pub enum DateFragments {
    /// Year, month and day, `yyyy-mm-dd`.
    YearMonthDay(u32, u32, u32),
    /// Year and month, `yyyy-mm`.
    YearMonth(u32, u32),
    /// Month and day, `mm-dd`.
    MonthDay(u32, u32),
    /// Year only, `yyyy`.
    Year(u32),
    /// Month or day.
    ///
    /// As month and day are both 2 digits,
    /// they cannot be distinguished without `base` context:
    /// When `base` is `Ymd` or `Ym`, day.
    /// When `base` is `Y`, month.
    MonthOrDay(u32),
}

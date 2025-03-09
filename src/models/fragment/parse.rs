//! Parsing details of `DateFragment`.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    error::ParseError,
    IResult, Parser,
};

use super::DateFragments;

pub fn fragment(s: &str) -> IResult<&str, DateFragments> {
    alt((
        fragment_year_month_day,
        fragment_year_month,
        fragment_month_day,
        fragment_year,
        fragment_month_or_day,
    ))
    .parse(s)
}

fn fragment_year_month_day(s: &str) -> IResult<&str, DateFragments> {
    (n_digits(4), hyphen, n_digits(2), hyphen, n_digits(2))
        .map(|(year, _, month, _, day)| DateFragments::YearMonthDay(year, month, day))
        .parse(s)
}

fn fragment_year_month(s: &str) -> IResult<&str, DateFragments> {
    (n_digits(4), hyphen, n_digits(2))
        .map(|(year, _, month)| DateFragments::YearMonth(year, month))
        .parse(s)
}

fn fragment_month_day(s: &str) -> IResult<&str, DateFragments> {
    (n_digits(2), hyphen, n_digits(2))
        .map(|(month, _, day)| DateFragments::MonthDay(month, day))
        .parse(s)
}

fn fragment_year(s: &str) -> IResult<&str, DateFragments> {
    n_digits(4).map(|year| DateFragments::Year(year)).parse(s)
}

fn fragment_month_or_day(s: &str) -> IResult<&str, DateFragments> {
    n_digits(2)
        .map(|m_or_d| DateFragments::MonthOrDay(m_or_d))
        .parse(s)
}

/// Parsing n digits into number.
///
/// 4 digits = year.
/// 2 digits = month | day.
fn n_digits<'a, Error: ParseError<&'a str>>(
    n: usize,
) -> impl Parser<&'a str, Output = u32, Error = Error> {
    take_while_m_n::<_, &str, Error>(n, n, |c: char| c.is_digit(10))
        .map(|digits: &str| digits.parse::<u32>().unwrap())
}

/// Hyphen is the separater between date fragments.
fn hyphen(s: &str) -> IResult<&str, &str> {
    tag("-")(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn year_month_day() {
        let input = "2025-01-03";
        let expected = DateFragments::YearMonthDay(2025, 1, 3);
        assert_eq!(fragment(input), Ok(("", expected)));
    }

    #[test]
    fn year_month() {
        let input = "2025-01";
        let expected = DateFragments::YearMonth(2025, 1);
        assert_eq!(fragment(input), Ok(("", expected)));
    }

    #[test]
    fn month_day() {
        let input = "01-03";
        let expected = DateFragments::MonthDay(1, 3);
        assert_eq!(fragment(input), Ok(("", expected)));
    }

    #[test]
    fn year() {
        let input = "2025";
        let expected = DateFragments::Year(2025);
        assert_eq!(fragment(input), Ok(("", expected)));
    }

    #[test]
    fn month_or_day() {
        let input = "01";
        let expected = DateFragments::MonthOrDay(1);
        assert_eq!(fragment(input), Ok(("", expected)));
    }
}

use std::{fs, path::Path, sync::LazyLock};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while_m_n},
    combinator::{eof, opt},
    multi::{many0, separated_list0},
    sequence::{preceded, terminated},
    IResult, Parser,
};
use regex::Regex;

use super::{fragment::DateFragment, Agmd, Source, Task};

impl Task {
    pub fn parse_line(text: &str, line: usize, path: &Path) -> Option<Self> {
        RE.captures(text).map(|captures| {
            let done = captures.get(1).unwrap().as_str() == "x";
            let text = captures.get(2).unwrap().as_str().to_string();
            let raw = captures.get(3).unwrap().as_str();
            let agmd = Agmd::parse(raw);
            let source = Source {
                line,
                path: path.to_path_buf(),
            };
            Task {
                done,
                text,
                raw: raw.to_string(),
                agmd,
                source,
            }
        })
    }

    pub fn load_from_path(path: &Path) -> Vec<Self> {
        let file = fs::read_to_string(path).unwrap();
        let iter = file.lines().enumerate();
        iter.map(|(line, text)| Self::parse_line(text, line, path))
            .flatten()
            .collect()
    }
}

impl Agmd {
    pub fn parse(raw: &str) -> Option<Self> {
        terminated(agmd, eof).parse(&raw).ok().map(|t| t.1)
    }
}

fn n_digits(s: &str, n: usize) -> IResult<&str, u32> {
    let (s, digits) = take_while_m_n(n, n, |c: char| c.is_digit(10))(s)?;
    Ok((s, digits.parse().unwrap()))
}

fn hyphen(s: &str) -> IResult<&str, &str> {
    tag("-")(s)
}

fn semicolon(s: &str) -> IResult<&str, &str> {
    tag(";")(s)
}

fn fragment_y(s: &str) -> IResult<&str, DateFragment> {
    let (s, year) = n_digits(s, 4)?;
    Ok((s, DateFragment::Y(year)))
}

fn fragment_mod(s: &str) -> IResult<&str, DateFragment> {
    let (s, md) = n_digits(s, 2)?;
    Ok((s, DateFragment::Mod(md)))
}

fn fragment_ymd(s: &str) -> IResult<&str, DateFragment> {
    let (s, year) = n_digits(s, 4)?;
    let (s, _) = hyphen(s)?;
    let (s, month) = n_digits(s, 2)?;
    let (s, _) = hyphen(s)?;
    let (s, day) = n_digits(s, 2)?;
    Ok((s, DateFragment::Ymd(year, month, day)))
}

fn fragment_ym(s: &str) -> IResult<&str, DateFragment> {
    let (s, year) = n_digits(s, 4)?;
    let (s, _) = hyphen(s)?;
    let (s, month) = n_digits(s, 2)?;
    Ok((s, DateFragment::Ym(year, month)))
}

fn fragment_md(s: &str) -> IResult<&str, DateFragment> {
    let (s, month) = n_digits(s, 2)?;
    let (s, _) = hyphen(s)?;
    let (s, day) = n_digits(s, 2)?;
    Ok((s, DateFragment::Md(month, day)))
}

fn fragment(s: &str) -> IResult<&str, DateFragment> {
    alt((
        fragment_ymd,
        fragment_ym,
        fragment_md,
        fragment_y,
        fragment_mod,
    ))
    .parse(s)
}

fn agmd_kv(s: &str) -> IResult<&str, Option<(&str, DateFragment)>> {
    let (s, k) = take_until("=")(s)?;
    let (s, _) = tag("=")(s)?;
    match k {
        "start" | "completed" | "due" => {
            let (s, fragment) = fragment(s)?;
            Ok((s, Some((k, fragment))))
        }
        _ => Ok((s, None)),
    }
}

fn agmd(s: &str) -> IResult<&str, Agmd> {
    let (s, base) = opt(fragment).parse(s)?;
    let (s, kv) = if base.is_some() {
        many0(preceded(semicolon, agmd_kv)).parse(s)?
    } else {
        separated_list0(semicolon, agmd_kv).parse(s)?
    };
    let mut start = None;
    let mut completed = None;
    let mut due = None;
    for (k, v) in kv.into_iter().flatten() {
        match k {
            "start" => {
                start = Some(v);
            }
            "completed" => {
                completed = Some(v);
            }
            "due" => {
                due = Some(v);
            }
            _ => unreachable!(),
        }
    }
    let agmd = Agmd {
        start: DateFragment::to_date(&start, &base, false),
        completed: DateFragment::to_date(&completed, &base, true),
        due: DateFragment::to_date(&due, &base, true),
    };
    Ok((s, agmd))
}

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        // optional prefix space
        r"^ *",
        // ol or ul
        r"(?:\d+\.|[*+-])",
        // space before marker
        r" +",
        // task list marker
        r"\[([ x])\]",
        // space after marker
        r" +",
        // any text
        r"(.*?)",
        // space
        r" +",
        // agmd
        r"<agmd:(.*?)>"
    ))
    .unwrap()
});

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use super::*;

    fn test_agmd_success(input: &str, expected: Agmd) {
        assert_eq!(agmd(input), Ok(("", expected)));
    }

    #[test]
    fn test_agmd_1() {
        let input = "2024-01-23";
        let date = NaiveDate::from_ymd_opt(2024, 01, 23);
        let expected = Agmd {
            start: date,
            completed: date,
            due: date,
        };
        test_agmd_success(input, expected);
    }

    #[test]
    fn test_agmd_2() {
        let input = "due=2024-01-23";
        let date = NaiveDate::from_ymd_opt(2024, 01, 23);
        let expected = Agmd {
            start: None,
            completed: None,
            due: date,
        };
        test_agmd_success(input, expected);
    }
}

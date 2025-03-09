use nom::{
    bytes::complete::{tag, take_until},
    combinator::{eof, opt},
    multi::{many0, separated_list0},
    sequence::{preceded, terminated},
    IResult, Parser,
};

use crate::models::fragment::{convert::Role, parse::fragment, DateFragments};

use super::Agmd;

impl Agmd {
    pub fn parse(raw: &str, done: bool) -> Option<Self> {
        agmd(raw, done).ok().map(|p| p.1)
    }
}

pub fn agmd(input: &str, done: bool) -> IResult<&str, Agmd> {
    // parse base and kvs
    let (input, base) = opt(fragment).parse(input)?;
    let (input, kvs) = if base.is_some() {
        many0(preceded(semicolon, key_value_pair)).parse(input)?
    } else {
        separated_list0(semicolon, key_value_pair).parse(input)?
    };
    // ensure that agmd parse all
    eof(input)?;

    // collect date fragments
    let (start, due, completed) = collect_date_fragments(kvs);

    // convert fragments to date
    let start = DateFragments::to_date(&start, &base, Role::Start);
    let due = DateFragments::to_date(&due, &base, Role::Due);
    let completed = DateFragments::to_date(&completed, &base, Role::Completed(done));
    // `completed` should snap to `due || start` iff done
    let completed = if done {
        completed.or(due).or(start)
    } else {
        completed
    };

    Ok((
        input,
        Agmd {
            start,
            due,
            completed,
        },
    ))
}

/// Parse each key-value pair in agmd url.
///
/// ```markdown
/// <agmd:2025-03-01;start=02;due=04;completed=03>
///                  <------> <----> <---------->
///                    These parts are parsed.
/// ```
fn key_value_pair(input: &str) -> IResult<&str, Option<(&str, DateFragments)>> {
    let (input, key) = terminated(take_until("="), tag("=")).parse(input)?;
    match key {
        // parse these tags only
        "start" | "completed" | "due" => {
            let (input, fragment) = fragment(input)?;
            Ok((input, Some((key, fragment))))
        }
        // ignore all other keys
        _ => Ok((input, None)),
    }
}

/// Collect date fragments from kv parsed.
///
/// Currently, only `start`, `due`, `completed` is collected.
fn collect_date_fragments(
    kvs: Vec<Option<(&str, DateFragments)>>,
) -> (
    Option<DateFragments>,
    Option<DateFragments>,
    Option<DateFragments>,
) {
    // extract date fragments
    let mut start = None;
    let mut due = None;
    let mut completed = None;
    for (k, v) in kvs
        .into_iter()
        // remove None items
        .flatten()
    {
        match k {
            "start" => {
                start = Some(v);
            }
            "due" => {
                due = Some(v);
            }
            "completed" => {
                completed = Some(v);
            }
            _ => {}
        }
    }
    (start, due, completed)
}

/// Parse semicolon which is used to separate agmd kvs.
fn semicolon(s: &str) -> IResult<&str, &str> {
    tag(";")(s)
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn agmd_base_undone() {
        let input = "2025-03-01";
        let expected = Agmd {
            start: NaiveDate::from_ymd_opt(2025, 03, 01),
            due: NaiveDate::from_ymd_opt(2025, 03, 01),
            completed: None,
        };
        assert_eq!(agmd(input, false), Ok(("", expected)));
    }

    #[test]
    fn agmd_base_done() {
        let input = "2025-03-01";
        let expected = Agmd {
            start: NaiveDate::from_ymd_opt(2025, 03, 01),
            due: NaiveDate::from_ymd_opt(2025, 03, 01),
            completed: NaiveDate::from_ymd_opt(2025, 03, 01),
        };
        assert_eq!(agmd(input, true), Ok(("", expected)));
    }

    #[test]
    fn agmd_base_start() {
        let input = "2025-03-01;start=02";
        let expected = Agmd {
            start: NaiveDate::from_ymd_opt(2025, 03, 02),
            due: NaiveDate::from_ymd_opt(2025, 03, 01),
            completed: None,
        };
        assert_eq!(agmd(input, false), Ok(("", expected)));
    }

    #[test]
    fn agmd_base_start_due() {
        let input = "2025-03-01;start=02;due=03";
        let expected = Agmd {
            start: NaiveDate::from_ymd_opt(2025, 03, 02),
            due: NaiveDate::from_ymd_opt(2025, 03, 03),
            completed: None,
        };
        assert_eq!(agmd(input, false), Ok(("", expected)));
    }

    #[test]
    fn agmd_empty() {
        let input = "";
        let expected = Agmd {
            start: None,
            due: None,
            completed: None,
        };
        assert_eq!(agmd(input, true), Ok(("", expected)));
    }

    #[test]
    fn agmd_start() {
        let input = "start=2025-01-01";
        let expected = Agmd {
            start: NaiveDate::from_ymd_opt(2025, 01, 01),
            due: None,
            completed: NaiveDate::from_ymd_opt(2025, 01, 01),
        };
        assert_eq!(agmd(input, true), Ok(("", expected)));
    }

    #[test]
    fn agmd_start_due() {
        let input = "start=2025-01-01;due=2025-03-01";
        let expected = Agmd {
            start: NaiveDate::from_ymd_opt(2025, 01, 01),
            due: NaiveDate::from_ymd_opt(2025, 03, 01),
            completed: NaiveDate::from_ymd_opt(2025, 03, 01),
        };
        assert_eq!(agmd(input, true), Ok(("", expected)));
    }
}

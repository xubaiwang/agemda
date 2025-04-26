use nom::{
    IResult, Parser,
    bytes::{complete::tag, take_until},
    combinator::{eof, opt},
    multi::{many0, separated_list0},
    sequence::{preceded, terminated},
};

use crate::fragment::{DateTimeFragment, date_time_fragment};

/// The structure of agmd link
pub struct Link {
    pub base: Option<DateTimeFragment>,
    pub start: Option<DateTimeFragment>,
    pub due: Option<DateTimeFragment>,
    pub completed: Option<DateTimeFragment>,
}

pub fn link(input: &str) -> IResult<&str, Link> {
    // parse base
    let (input, base) = opt(date_time_fragment).parse(input)?;
    // parse kvs
    let (input, kvs) = if base.is_some() {
        many0(preceded(semicolon, key_value_pair)).parse(input)?
    } else {
        separated_list0(semicolon, key_value_pair).parse(input)?
    };
    // ensure that agmd parse all
    let (input, _) = eof(input)?;

    // collect all fragments
    let (start, due, completed) = collect_fragments(kvs);

    Ok((
        input,
        Link {
            base,
            start,
            due,
            completed,
        },
    ))
}

fn key_value_pair(input: &str) -> IResult<&str, Option<(&str, DateTimeFragment)>> {
    let (input, key) = terminated(take_until("="), tag("=")).parse(input)?;
    match key {
        // parse these tags only
        "start" | "completed" | "due" => {
            let (input, fragment) = date_time_fragment(input)?;
            Ok((input, Some((key, fragment))))
        }
        // ignore all other keys
        _ => Ok((input, None)),
    }
}

fn semicolon(input: &str) -> IResult<&str, &str> {
    tag(";")(input)
}

fn collect_fragments(
    kvs: Vec<Option<(&str, DateTimeFragment)>>,
) -> (
    Option<DateTimeFragment>,
    Option<DateTimeFragment>,
    Option<DateTimeFragment>,
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

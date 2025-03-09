use std::sync::LazyLock;

use regex::Regex;

use crate::models::agmd::Agmd;

use super::Todo;

impl Todo {
    /// Parse todo from a line.
    ///
    /// `None` means this line does not contains an agmd tracked todo.
    pub fn parse_line(text: &str) -> Option<Self> {
        RE.captures(text).map(|captures| {
            let done = captures.get(1).unwrap().as_str() == "x";
            let text = captures.get(2).unwrap().as_str().to_string();
            let raw = captures.get(3).unwrap().as_str();
            let agmd = Agmd::parse(raw, done);
            Todo {
                summary: text,
                raw: raw.to_string(),
                agmd,
            }
        })
    }

    /// Parse file into todo lines.
    pub fn parse_file(file: &str) -> Vec<(usize, Self)> {
        let iter = file.lines().enumerate();
        iter.map(|(line, text)| Self::parse_line(text).map(|s| (line, s)))
            .flatten()
            .collect()
    }
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

    #[test]
    fn todo() {
        let input = "- [x] summary <agmd:2025-03-01>";
        let date = NaiveDate::from_ymd_opt(2025, 03, 01);
        let expected = Todo {
            summary: "summary".to_string(),
            raw: "2025-03-01".to_string(),
            agmd: Some(Agmd {
                start: date,
                completed: date,
                due: date,
            }),
        };
        assert_eq!(Todo::parse_line(input), Some(expected));
    }
}

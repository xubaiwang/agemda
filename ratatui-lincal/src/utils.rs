use agemda_core::Todo;
use chrono::NaiveDate;
use ratatui::{buffer::Buffer, style::Style};

pub fn set_string_opt(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    string: impl AsRef<str>,
    style: impl Into<Style>,
) {
    if y >= buf.area().top() && y < buf.area.bottom() {
        buf.set_string(x, y, string, style);
    }
}

/// Utility function for filtering todo out of day.
pub fn has_overlap(todo: &Todo, date: NaiveDate, should_show_completed: bool) -> bool {
    if let Ok(agmd) = &todo.attributes {
        // match (agmd.start, agmd.due) {
        //     // TODO: both none for ad-hoc for every day
        //     // (None, None) => true,
        //     // due only
        //     // TODO: handle overdue
        //     // (None, Some(due)) => date <= due,
        //     // start only
        //     // (Some(start), None) => start <= date,
        //     // both
        //     // (Some(start), Some(due)) => date <= due && start <= date,

        //     _ => false,
        // }
        // TODO: too many, only due currently
        let is_due = if let Some(due) = agmd.due {
            due.date_naive() == date
        } else {
            false
        };
        if should_show_completed {
            is_due
        } else {
            is_due && agmd.completed.is_none()
        }
    } else {
        // TODO: handle malform
        false
    }
}

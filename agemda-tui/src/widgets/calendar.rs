use std::sync::Arc;

use agemda_core::Todo;
use chrono::{Datelike, Days, NaiveDate};
use ratatui::{
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    widgets::StatefulWidget,
};

pub type CalendarData = Arc<Vec<Todo>>;

/// Calendar state, use by both `Calendar` and `CalendarRow`.
pub struct CalendarState {
    pub selected: NaiveDate,
    pub selected_item: usize,
}

impl CalendarState {
    pub fn new(selected: NaiveDate) -> Self {
        Self {
            selected,
            selected_item: 0,
        }
    }

    pub fn select_next(&mut self) {
        self.selected = self.selected.checked_add_days(Days::new(1)).unwrap();
        self.selected_item = 0;
    }

    pub fn select_previous(&mut self) {
        self.selected = self.selected.checked_sub_days(Days::new(1)).unwrap();
        self.selected_item = 0;
    }

    pub fn select_next_item(&mut self) {
        self.selected_item += 1;
    }

    pub fn select_previous_item(&mut self) {
        self.selected_item = self.selected_item.saturating_sub(1);
    }
}

/// Calendar row widget.
pub struct CalendarRow {
    data: CalendarData,
    start: NaiveDate,
    today: NaiveDate,
    day_width: u16,
    should_show_completed: bool,
}

impl CalendarRow {
    pub fn new(
        data: CalendarData,
        today: NaiveDate,
        start: NaiveDate,
        day_width: u16,
        should_show_completed: bool,
    ) -> Self {
        Self {
            data,
            today,
            start,
            day_width,
            should_show_completed,
        }
    }
}

impl StatefulWidget for CalendarRow {
    type State = CalendarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let default_style = Style::default();

        // calculate y for axis and label
        let y_axis = area.bottom() - 2;
        let y_label = area.bottom() - 1;

        // how many days in this row
        let day_count = area.width / self.day_width;

        // calculate width to be left on both side
        let rest_width = area.width - day_count * self.day_width;
        let indent = rest_width / 2;
        let indented_x = area.x + indent;

        // pre allocate box drawing chars
        let horizontals = "─".repeat(self.day_width as usize - 1);

        // for each day
        for day_index in 0..day_count {
            // calculate offseted x
            let x = indented_x + day_index * self.day_width;

            // calculate the corresponding date
            let date = self
                .start
                .checked_add_days(Days::new(day_index as u64))
                .unwrap();
            let is_selected_date = state.selected == date;

            // filter out data of this date
            let filtered: Vec<_> = self
                .data
                .iter()
                .filter(|todo| has_overlap(todo, date, self.should_show_completed))
                .collect();

            // TODO: fix selection out of range
            // if state.selected_item >= filtered.len() {
            //     if filtered.len() > 0 {
            //         state.selected_item = filtered.len();
            //     }
            // }

            // render axis
            set_string_opt(buf, x, y_axis, "┬", default_style);
            set_string_opt(buf, x + 1, y_axis, &horizontals, default_style);

            // render label
            // two case:
            // first day of month => show year-month,
            // otherwise => day
            let label_string = if date.day0() == 0 {
                format!("{}-{}", date.year(), date.month())
            } else {
                date.day().to_string()
            };
            set_string_opt(
                buf,
                x,
                y_label,
                label_string,
                if is_selected_date {
                    default_style.reversed().bold()
                } else {
                    default_style
                },
            );

            // render each todo item
            for (item_index, item) in filtered.iter().enumerate() {
                // the first y is kept empty for visual separation, so plus 1
                let y = area.y + 1 + item_index as u16;

                let is_selected_item = is_selected_date && state.selected_item == item_index;

                // TODO: padding and trim
                let style = if let Ok(agmd) = &item.attributes {
                    if agmd.completed.is_some() {
                        default_style.dim()
                    } else {
                        default_style
                    }
                } else {
                    default_style
                };
                let style = if is_selected_item {
                    style.reversed()
                } else {
                    style
                };
                set_string_opt(buf, x + 2, y, &item.summary, style);
            }

            // TODO: render today indicator if is today
            if date == self.today {
                for y in (area.y + 1)..=y_axis {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        if cell.symbol() == "┌" {
                            cell.set_symbol("┠");
                        } else {
                            cell.set_symbol("┃");
                        }
                        cell.set_style(default_style.reset().red());
                    }
                }
            }
        }

        // render last tick
        let last_tick_x = indented_x + day_count * self.day_width;
        set_string_opt(buf, last_tick_x, y_axis, "┬", default_style);
        set_string_opt(buf, last_tick_x, y_label, ">", default_style);
    }
}

/// Calendar widget.
pub struct Calendar {
    data: CalendarData,
    start: NaiveDate,
    today: NaiveDate,
    day_width: u16,
    should_show_completed: bool,
}

impl Calendar {
    pub fn new(
        data: CalendarData,
        today: NaiveDate,
        start: NaiveDate,
        day_width: u16,
        should_show_completed: bool,
    ) -> Self {
        Self {
            data,
            today,
            start,
            day_width,
            should_show_completed,
        }
    }
}

impl StatefulWidget for Calendar {
    type State = CalendarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // accumulate height for comparing with area height
        let mut acc_height = 0;

        // number of days per row
        let days_count = area.width / self.day_width;

        // start argument of row
        let mut row_start = self.start;

        loop {
            // cease when height overflow
            if acc_height >= area.height {
                break;
            }

            // calculate height of this row
            let row_height = (0..days_count)
            .map(|day_index| {
                // the corresponding date
                let date = row_start
                    .checked_add_days(Days::new(day_index as u64))
                    .unwrap();
                // TODO: duplicate filtering shoud be cached and passed
                let filtered = self.data.iter().filter(|todo| has_overlap(todo, date, self.should_show_completed));
                filtered.count()
            })
            .max()
            .unwrap_or(0)
            // axis, label, empty, so three
            + 3;

            // create subarea for row
            let row_area = Rect {
                y: area.y + acc_height,
                height: row_height as u16,
                ..area
            };
            CalendarRow::new(
                self.data.clone(),
                self.today,
                row_start,
                self.day_width,
                self.should_show_completed,
            )
            .render(row_area, buf, state);

            // accumulate height and row_start
            acc_height += row_height as u16;
            row_start = row_start
                .checked_add_days(Days::new(days_count as u64))
                .unwrap();
        }
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

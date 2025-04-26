use chrono::{Datelike, Days, NaiveDate};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::StatefulWidget,
};

use crate::{
    data::{CalendarData, CalendarState},
    utils::{has_overlap, set_string_opt},
};

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

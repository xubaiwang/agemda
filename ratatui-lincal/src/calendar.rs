use chrono::{Days, NaiveDate};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::{
    data::{CalendarData, CalendarState},
    row::CalendarRow,
    utils::has_overlap,
};

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

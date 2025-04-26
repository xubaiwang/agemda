use std::sync::Arc;

use agemda_core::Todo;
use chrono::{Days, NaiveDate};

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

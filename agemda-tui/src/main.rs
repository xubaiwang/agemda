use std::sync::Arc;

use agemda::cli::Cli;
use agemda_core::Todo;
use agemda_io::load::load_todos_from_root;
use chrono::{Days, Local, NaiveDate};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    prelude::{Buffer, Rect},
    widgets::{StatefulWidget, Widget},
    DefaultTerminal,
};
use ratatui_lincal::{calendar::Calendar, data::CalendarState, utils::has_overlap};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = argh::from_env();
    let mut terminal = ratatui::init();
    App::new(cli)?.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

// MARK: app

struct App {
    cli: Cli,
    should_quit: bool,
    should_show_completed: bool,

    day_width: u16,

    // date related fields
    /// The current date of real world time.
    today: NaiveDate,
    /// The start of date to be rendered.
    start: NaiveDate,

    state: CalendarState,

    data: Arc<Vec<Todo>>,
}

impl App {
    /// Create a new app using given cli options.
    pub fn new(cli: Cli) -> Result<Self, Box<dyn std::error::Error>> {
        let should_quit = false;
        let should_show_completed = false;

        // TODO: make into cli option and dynamically changable
        let day_width = 25;

        let today = Local::now().date_naive();
        let start = today.checked_sub_days(Days::new(3)).unwrap();

        let state = CalendarState::new(today);

        let data = Arc::new(load_todos_from_root(&cli.root)?);

        Ok(Self {
            cli,
            should_quit,
            should_show_completed,
            day_width,
            today,
            start,
            state,
            data,
        })
    }

    /// Run the draw-event loop on terminal.
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // draw ui
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;

            // read and handle events
            self.handle_event(event::read()?)?;

            // quitting
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// How the app handle events.
    ///
    /// Currently the keybinding is hardcoded and handle only key event.
    pub fn handle_event(&mut self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            // handle key only
            Event::Key(key_event) => {
                // handle key code only (ignoring modifiers)
                match key_event.code {
                    // q => quit
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Char('r') => self.reload()?,
                    KeyCode::Char('.') => self.toggle_show_completed(),
                    // TODO: d for show overdue
                    KeyCode::Enter => self.open_selected(),
                    KeyCode::Char('k') | KeyCode::Up => self.state.select_previous_item(),
                    KeyCode::Char('j') | KeyCode::Down => self.state.select_next_item(),
                    KeyCode::Char('h') | KeyCode::Left => self.state.select_previous(),
                    KeyCode::Char('l') | KeyCode::Right => self.state.select_next(),
                    KeyCode::Char('[') => self.select_previous_start(),
                    KeyCode::Char(']') => self.select_next_start(),
                    // other key code is ignored
                    _ => {}
                }
            }
            // other events than key is ignored
            _ => {}
        }
        Ok(())
    }

    /// Reload data
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.data = Arc::new(load_todos_from_root(&self.cli.root)?);
        self.today = Local::now().date_naive();
        Ok(())
    }

    pub fn toggle_show_completed(&mut self) {
        self.should_show_completed = !self.should_show_completed;
    }

    pub fn open_selected(&self) {
        let mut filtered = self
            .data
            .iter()
            .filter(|todo| has_overlap(todo, self.state.selected, self.should_show_completed));
        if let Some(selected) = filtered.nth(self.state.selected_item) {
            let path = selected.metadata.path.as_path();
            _ = open::that_detached(path);
        }
    }

    pub fn select_previous_start(&mut self) {
        self.start = self.start.checked_sub_days(Days::new(1)).unwrap();
    }

    pub fn select_next_start(&mut self) {
        self.start = self.start.checked_add_days(Days::new(1)).unwrap();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let calendar = Calendar::new(
            self.data.clone(),
            self.today,
            self.start,
            self.day_width,
            self.should_show_completed,
        );
        StatefulWidget::render(calendar, area, buf, &mut self.state);
    }
}

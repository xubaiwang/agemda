use std::{env, path::PathBuf};

use agemda::load::{Loader, TodoMapGrouped};
use bpaf::{construct, positional, OptionParser, Parser};
use pathdiff::diff_paths;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Style, Stylize},
    widgets::{Block, List, ListState, StatefulWidget, Tabs, Widget},
    DefaultTerminal,
};

fn main() -> anyhow::Result<()> {
    let options = options().run();
    let app = App::new(options)?;

    let mut terminal = ratatui::init();
    app.run(&mut terminal)?;
    ratatui::restore();

    Ok(())
}

struct App {
    running: bool,
    options: Options,
    loader: Loader,
    map: TodoMapGrouped,
    selected_group: usize,
    list_state: ListState,
}

impl App {
    pub fn new(options: Options) -> anyhow::Result<Self> {
        let loader = Loader::new(&options.root);
        let map = loader.load()?;
        Ok(Self {
            running: true,
            options,
            loader,
            selected_group: 3,
            list_state: ListState::default(),
            map,
        })
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> anyhow::Result<()> {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    self.running = false;
                }
                KeyCode::Char('r') => {
                    // reload data
                    self.map = self.loader.load()?;
                }
                KeyCode::Right => {
                    self.next_group();
                }
                KeyCode::Left => {
                    self.prev_group();
                }
                KeyCode::Char('1') => self.select_group(0),
                KeyCode::Char('2') => self.select_group(1),
                KeyCode::Char('3') => self.select_group(2),
                KeyCode::Char('4') => self.select_group(3),
                KeyCode::Char('5') => self.select_group(4),
                KeyCode::Char('6') => self.select_group(5),
                KeyCode::Char('7') => self.select_group(6),
                KeyCode::Char('8') => self.select_group(7),
                KeyCode::Char('9') => self.select_group(8),
                KeyCode::Char('0') => self.select_group(9),

                KeyCode::Up => self.prev_todo(),
                KeyCode::Down => self.next_todo(),

                KeyCode::Enter => {
                    let parent = key.modifiers == KeyModifiers::SHIFT;
                    self.open_selected(parent);
                }

                _ => {}
            }
        }
        Ok(())
    }

    fn reset_list(&mut self) {
        self.list_state = ListState::default();
    }

    fn prev_todo(&mut self) {
        match self.list_state.selected() {
            Some(i) => self.list_state.select(Some(i.saturating_sub(1))),
            None => self.list_state.select(Some(0)),
        }
    }

    fn next_todo(&mut self) {
        match self.list_state.selected() {
            Some(i) => self.list_state.select(Some(i + 1)),
            None => self.list_state.select(Some(0)),
        }
    }

    fn prev_group(&mut self) {
        if self.selected_group == 0 {
            self.selected_group = self.map.groups().len() - 1;
        } else {
            self.selected_group -= 1;
        }
        self.reset_list();
    }

    fn next_group(&mut self) {
        let pre = self.selected_group + 1;
        if pre >= self.map.groups().len() {
            self.selected_group = 0;
        } else {
            self.selected_group = pre;
        }
        self.reset_list();
    }

    fn select_group(&mut self, group: usize) {
        if group < self.map.groups().len() {
            self.selected_group = group;
        }
        self.reset_list();
    }

    fn open_selected(&self, parent: bool) {
        if let Some(selected_todo) = self.list_state.selected() {
            let path = self.map.groups()[self.selected_group].1[selected_todo].0;
            if parent {
                if let Some(path) = path.parent() {
                    _ = open::that_detached(path);
                }
            } else {
                _ = open::that_detached(path);
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);
        self.render_tabs(layout[0], buf);
        self.render_todo_list(layout[1], buf);
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let tabs = Tabs::new(
            self.map
                .groups()
                .iter()
                .enumerate()
                .map(|(i, (name, _))| format!("{} {}", i + 1, name)),
        )
        .block(Block::bordered().title("Groups"))
        .highlight_style(Style::default().bold().underlined())
        .select(self.selected_group);
        tabs.render(area, buf);
    }

    fn render_todo_list(&mut self, area: Rect, buf: &mut Buffer) {
        let todos = &self.map.groups()[self.selected_group].1;

        let list = List::new(todos.iter().map(|(path, (line, todo))| {
            let path = diff_paths(&path, &self.options.root).unwrap();
            format!(
                "- [{}] {} <agmd:{}>  @ {}#L{}",
                if todo.done() { "x" } else { " " },
                todo.summary,
                todo.raw,
                path.display(),
                line + 1
            )
        }))
        .block(Block::bordered().title("Todos"))
        .highlight_style(Style::default().reversed());
        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}

/// CLI definition.
#[derive(Debug, Clone)]
struct Options {
    // positional
    root: PathBuf,
}

fn options() -> OptionParser<Options> {
    let root = positional("ROOT")
        .help("The root dir to search for markdown files")
        .fallback_with(|| env::current_dir());
    construct!(Options { root }).to_options()
}

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use agemda_core::{Attributes, Metadata, Todo};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::{
    convert::{Role, fragment_to_datetime},
    link::link,
};

pub fn parse_file(acc: &mut Vec<Todo>, path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;

    // create parser
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&text, options);

    // states
    let mut state = State::new(path);

    // handle events
    for event in parser {
        if let Some(todo) = state.handle(&event) {
            acc.push(todo);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    path: PathBuf,
    hier: Vec<ListState>,
    in_agmd_link: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum ListState {
    Plain,
    Task(bool, String, Option<String>),
}

impl State {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            hier: vec![],
            in_agmd_link: false,
        }
    }

    fn handle(&mut self, event: &Event) -> Option<Todo> {
        self.handle_item_start(event);
        self.handle_task(event);
        self.handle_link(event);
        self.handle_text(event);
        self.handle_item_end(event)
    }

    fn handle_item_start(&mut self, event: &Event) {
        match event {
            Event::Start(Tag::Item) => {
                self.hier.push(ListState::Plain);
            }
            _ => {}
        }
    }

    fn handle_task(&mut self, event: &Event) {
        match event {
            Event::TaskListMarker(b) => {
                if let Some(list_state) = self.hier.last_mut() {
                    *list_state = ListState::Task(*b, String::new(), None);
                }
            }
            _ => {}
        }
    }

    fn handle_link(&mut self, event: &Event) {
        if let Some(last) = self.hier.last_mut() {
            match last {
                ListState::Task(_, _, agmd) => match event {
                    Event::Start(Tag::Link { dest_url, .. }) => {
                        if dest_url.starts_with("agmd:") {
                            *agmd = Some(dest_url[5..].to_string());
                            self.in_agmd_link = true;
                        }
                    }
                    Event::End(TagEnd::Link) => self.in_agmd_link = false,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn handle_text(&mut self, event: &Event) {
        if !self.in_agmd_link {
            if let Some(last) = self.hier.last_mut() {
                match last {
                    ListState::Task(_, text, _) => match event {
                        Event::Text(cow_str) => text.push_str(&cow_str),
                        _ => {}
                    },
                    ListState::Plain => {}
                }
            }
        }
    }

    fn handle_item_end(&mut self, event: &Event) -> Option<Todo> {
        match event {
            Event::End(TagEnd::Item) => {
                let last = self.hier.pop()?;
                match last {
                    ListState::Task(b, summary, Some(agmd)) => {
                        let attributes = match link(&agmd) {
                            Ok((_, link)) => {
                                let start =
                                    fragment_to_datetime(&link.start, &link.base, Role::Start);
                                let due = fragment_to_datetime(&link.due, &link.base, Role::End);
                                let completed = match b {
                                    true => match &link.completed {
                                        Some(_) => fragment_to_datetime(
                                            &link.completed,
                                            &link.base,
                                            Role::End,
                                        ),
                                        None => due,
                                    },
                                    false => None,
                                };
                                Ok(Attributes {
                                    start,
                                    due,
                                    completed,
                                })
                            }
                            Err(_) => Err(agmd),
                        };
                        Some(Todo {
                            metadata: Metadata {
                                path: self.path.clone(),
                            },
                            summary,
                            attributes,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

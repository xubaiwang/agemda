use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use agemda::{load::load_todos_from_root, models::todo::Todo};
use chrono::{Days, Local, NaiveDate};
use clap::Parser;
use colored::Colorize;
use pathdiff::diff_paths;
use url::Url;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let todos = load_todos_from_root(&cli.root)?;
    show_todos(&todos, &cli);
    Ok(())
}

fn show_todos(todo_map: &HashMap<PathBuf, Vec<(usize, Todo)>>, opts: &Cli) {
    let show_done = opts.done;
    let show_all = opts.all;
    let show_week = opts.week;
    let strict_start = opts.strict;

    let today = opts.today;
    let tomorrow = today.checked_add_days(Days::new(1)).unwrap();
    let three_days = today.checked_add_days(Days::new(2)).unwrap();
    let week = today.checked_add_days(Days::new(7)).unwrap();

    let mut mal: Vec<(&Path, usize, &Todo)> = vec![];
    let mut overdue: Vec<(&Path, usize, &Todo)> = vec![];
    let mut due_today: Vec<(&Path, usize, &Todo)> = vec![];
    let mut due_tomorrow: Vec<(&Path, usize, &Todo)> = vec![];
    let mut due_overmorrow: Vec<(&Path, usize, &Todo)> = vec![];
    let mut due_week: Vec<(&Path, usize, &Todo)> = vec![];
    let mut due_all: Vec<(&Path, usize, &Todo)> = vec![];

    // collect into groups
    for (path, todos) in todo_map {
        for (line, todo) in todos {
            // malform
            match &todo.agmd {
                // parse error => mal
                None => mal.push((path, *line, &todo)),

                Some(agmd) => {
                    if !show_done && agmd.completed.is_some() {
                        continue;
                    }
                    match agmd.due {
                        Some(due) => {
                            if let Some(start) = agmd.start {
                                // start > due => mal
                                if start > due {
                                    mal.push((path, *line, &todo));
                                    continue;
                                }
                                if strict_start && start > today {
                                    continue;
                                }
                            }
                            if due < today {
                                overdue.push((path, *line, &todo));
                                continue;
                            }
                            if due == today {
                                due_today.push((path, *line, &todo));
                                continue;
                            }
                            if due <= tomorrow {
                                due_tomorrow.push((path, *line, &todo));
                                continue;
                            }
                            if due <= three_days {
                                due_overmorrow.push((path, *line, &todo));
                                continue;
                            }
                            if show_week && due <= week {
                                due_week.push((path, *line, &todo));
                                continue;
                            }
                            if show_all {
                                due_all.push((path, *line, &todo));
                            }
                        }
                        None => {
                            if let Some(start) = agmd.start {
                                if strict_start && start > today {
                                    continue;
                                }
                            }
                            due_today.push((path, *line, &todo));
                        }
                    }
                }
            }
        }
    }

    // print each group
    let print_todo = |(path, line, todo): (&Path, usize, &Todo)| {
        let linked_path = match Url::from_file_path(path) {
            Ok(url) => {
                format!(
                    "{}{}{}",
                    osc8::Hyperlink::new(url.as_str()),
                    diff_paths(path, &opts.root).unwrap().display(),
                    osc8::Hyperlink::END
                )
            }
            Err(_) => format!("{}", path.display()),
        };
        let line = line + 1;
        let meta = format!("@ {linked_path}#L{line}").dimmed();

        let summary = &todo.summary.bold();

        let raw = &todo.raw;
        let agmd_bracketed = format!("<agmd:{raw}>").bright_blue();

        let done_sign = match &todo.agmd {
            Some(agmd) => {
                if agmd.completed.is_some() {
                    "x".green()
                } else {
                    " ".yellow()
                }
            }
            None => "?".red(),
        };
        println!("- [{done_sign}] {summary} {agmd_bracketed}  {meta}");
    };

    if !mal.is_empty() {
        println!("{}", "Malform:".bold());
        mal.into_iter().for_each(print_todo);
    }
    if !overdue.is_empty() {
        println!("{}", "Overdue:".bold());
        overdue.into_iter().for_each(print_todo);
    }
    println!("{}", "Today:".bold());
    due_today.into_iter().for_each(print_todo);
    println!("{}", "Tomorrow:".bold());
    due_tomorrow.into_iter().for_each(print_todo);
    println!("{}", "Overmorrow:".bold());
    due_overmorrow.into_iter().for_each(print_todo);
    if show_week {
        println!("{}", "This Week:".bold());
        due_week.into_iter().for_each(print_todo);
    }
    if show_all {
        println!("{}", "All:".bold());
        due_all.into_iter().for_each(print_todo);
    }
}

/// CLI definition.
#[derive(Debug, Clone, Parser)]
struct Cli {
    /// The root dir to search for markdown files
    #[arg(
        default_value_os_t =
            env::current_dir().expect("Fail to get current dir")
    )]
    root: PathBuf,
    /// The date of today.
    #[arg(
        long,
        short,
        default_value_t = Local::now().date_naive()
    )]
    today: NaiveDate,
    /// Whether to show done task.
    #[arg(long, short, default_value_t = false)]
    done: bool,
    /// Whether to show all task that is started.
    #[arg(long, short, default_value_t = false)]
    all: bool,
    /// Whether start is strict.
    #[arg(long, short, default_value_t = false)]
    strict: bool,
    /// Whether to show week.
    #[arg(long, short, default_value_t = false)]
    week: bool,
}

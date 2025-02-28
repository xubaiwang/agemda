use std::path::{absolute, Path, PathBuf};

use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use ignore::{types::TypesBuilder, DirEntry, WalkBuilder};

use agemda::Task;

fn main() {
    let options = Options::parse();

    // create walk
    let walk = WalkBuilder::new(&options.path)
        .types(
            TypesBuilder::new()
                .add_defaults()
                .select("md")
                .build()
                .unwrap(),
        )
        .add_custom_ignore_filename(".agmdignore")
        .build();

    // iter
    for entry in walk {
        match entry {
            Ok(entry) => handle_entry(&entry, &options),
            Err(err) => println!("ERROR: {}", err),
        }
    }
}

fn handle_entry(entry: &DirEntry, options: &Options) {
    let path = entry.path();
    // skip dir
    if path.is_dir() {
        return;
    }
    let tasks = Task::load_from_path(path);
    for task in tasks {
        handle_task(&task, options);
    }
}

fn handle_task(task: &Task, options: &Options) {
    match options.command {
        Command::Mal => {
            match &task.agmd {
                Some(agmd) => {
                    if agmd.start.is_some() || agmd.completed.is_some() {
                        return;
                    }
                }
                None => {}
            }
            println!(
                "MALF {}#{}: {} <agmd:{}>",
                task.source
                    .path
                    .strip_prefix(&options.path)
                    .unwrap()
                    .display(),
                task.source.line + 1,
                task.text,
                task.raw
            );
        }
        Command::Today { date, done } => {
            if task.done != done {
                return;
            }
            if !task.source.path.starts_with(&options.path) {
                return;
            }
            if let Some(agmd) = &task.agmd {
                match (agmd.start, agmd.due) {
                    (None, None) => return,
                    (None, Some(due)) => {
                        if due < date {
                            return;
                        }
                    }
                    (Some(start), None) => {
                        if start > date {
                            return;
                        }
                    }
                    (Some(start), Some(due)) => {
                        if start > date {
                            return;
                        }
                        if due < date {
                            return;
                        }
                    }
                }
                println!(
                    "{} {}#{}: {} <agmd:{}>",
                    if task.done { "DONE" } else { "TODO" },
                    task.source
                        .path
                        .strip_prefix(&options.path)
                        .unwrap()
                        .display(),
                    task.source.line + 1,
                    task.text,
                    task.raw
                )
            }
        }
    }
}

#[derive(Debug, Clone, Parser)]
struct Options {
    /// The path must be absolute to be used strip_prefix.
    #[arg(default_value = ".", value_parser = parse_path_absolute)]
    path: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Find malform items
    Mal,
    Today {
        #[arg(long, default_value_t = current_date())]
        date: NaiveDate,
        #[arg(long, default_value_t = false)]
        done: bool,
    },
}

fn current_date() -> NaiveDate {
    Local::now().date_naive()
}

fn parse_path_absolute(s: &str) -> Result<PathBuf, std::io::Error> {
    absolute(Path::new(s))
}

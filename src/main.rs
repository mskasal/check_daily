use chrono::{DateTime, Duration, Utc};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod date;
mod todos;

use crate::date::{Analyzer, TimestampAnalyzer};
use crate::todos::{Readable as _, Todo, Todos, TraidTodo as _, TraitTodos, Writable as _};

#[derive(Parser)]
#[command(author, version, about, long_about)]
struct Cli {
    name: Option<String>,

    #[arg(short, long, value_name = "conf")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Test {
        #[arg(short, long)]
        list: bool,
    },
    Add {
        #[arg(short, long)]
        todo: String,
    },
    Check {
        number: usize,
    },
    UnCheck {
        number: usize,
    },
    Delete {
        number: usize,
    },
    List,
}

fn main() {
    let cli = Cli::parse();
    let mut todos = Todos::new();

    let _ = todos.read();

    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Dont't be crazy"),
    }

    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }

        Some(Commands::Add { todo }) => {
            todos.add(Todo::new(todo.to_string()));
            print_items(&todos.todos);
            let _ = todos.write();
        }

        Some(Commands::Check { number }) => {
            todos.mark_completed(todos.todos[*number].id);
            print_items(&todos.todos);
            let _ = todos.write();
        }

        Some(Commands::UnCheck { number }) => {
            todos.unmark_completed(todos.todos[*number].id);
            print_items(&todos.todos);
            let _ = todos.write();
        }

        Some(Commands::Delete { number }) => {
            todos.remove(todos.todos[*number].id);
            print_items(&todos.todos);
            let _ = todos.write();
        }

        Some(Commands::List) => {
            print_items(&todos.todos);
        }

        None => {}
    }
}

fn print_items(items: &[Todo]) {
    const ITALIC: &str = "\x1B[3m";
    const RESET: &str = "\x1B[0m";

    let (today_items, other_items): (Vec<_>, Vec<_>) = items.iter().partition(|obj| {
        let date_time = DateTime::from_timestamp(obj.timestamp, 0).unwrap();
        let obj_date = date_time.date_naive();
        let today_date = Utc::now().date_naive();
        obj_date == today_date || obj_date == today_date - Duration::days(1)
    });

    if !other_items.is_empty() {
        let time_analyzer = TimestampAnalyzer::new(other_items[0].timestamp);
        println!("{}", time_analyzer);
        for (index, item) in other_items.iter().enumerate() {
            print!("{ITALIC}{index}.{RESET} ");
            println!("{item}");
        }
    }

    if !today_items.is_empty() {
        let time_analyzer = TimestampAnalyzer::new(today_items[0].timestamp);
        println!("{}", time_analyzer);
        for (index, item) in today_items.iter().enumerate() {
            print!("{ITALIC}{index}.{RESET} ");
            println!("{item}");
        }
    }
}

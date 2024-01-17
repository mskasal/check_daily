use chrono::{DateTime, Duration, Utc};
use clap::{Parser, Subcommand};
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use std::path::PathBuf;
use todos::TraidTodo;

mod date;
mod todos;

use crate::date::{Analyzer, TimestampAnalyzer};
use crate::todos::{Readable as _, Todo, Todos, TraitTodos, Writable as _};

use anyhow::Result;

use crossterm::{
    event::{self, Event::Key},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Frame, Terminal};

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
    UI,
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

        Some(Commands::UI) => {
            let _ = run_app(&mut todos.todos);
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

struct StatefulList<'a, T> {
    state: ListState,
    items: &'a mut [T],
}

trait StatefulListTrait<'a, T> {
    fn with_items(items: &'a mut [T]) -> StatefulList<'a, T>;

    fn next(&mut self);

    fn previous(&mut self);

    fn unselect(&mut self);
}

impl<'a, T> StatefulListTrait<'a, T> for StatefulList<'a, T> {
    fn with_items(items: &'a mut [T]) -> StatefulList<'a, T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a> {
    items: StatefulList<'a, Todo>,
    should_quit: bool,
}

impl<'a> App<'a> {
    fn new(items: &'a mut [Todo]) -> App<'a> {
        App {
            items: StatefulList::with_items(items),
            should_quit: false,
        }
    }

    fn toggle(&mut self, index: usize) {
        self.items.items[index].toggle_status(!self.items.items[index].completed);
    }
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn ui(app: &App, f: &mut Frame) {
    let area = Rect::new(0, 0, 106, 16);
    let items: Vec<ListItem<'_>> = app
        .items
        .items
        .iter()
        .enumerate()
        .map(|(_, item)| {
            let mut list_item = ListItem::new(format!("Item {}", item.id));

            // Customize the design based on the 'highlighted' property
            if item.completed {
                list_item = list_item.style(Style::default().fg(Color::Yellow));
            }

            list_item
        })
        .collect();
    // .map(|i| i.text.clone());
    let list = List::new(items)
        .block(Block::default().title("Todos").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .repeat_highlight_symbol(true);

    f.render_stateful_widget(list, area, &mut app.items.state.clone());
}

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    KeyCode::Left | KeyCode::Char('h') => app.items.unselect(),
                    KeyCode::Down | KeyCode::Char('j') => app.items.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.items.previous(),
                    KeyCode::Char('t') => {
                        if let Some(selected_index) = app.items.state.selected() {
                            println!("{selected_index}");
                            app.toggle(selected_index);
                        }
                    }
                    KeyCode::Char('q') => app.should_quit = true,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn run_ui(items: &mut [Todo]) -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // application state
    let mut app = App::new(items);

    loop {
        t.draw(|f| {
            ui(&app, f);
        })?;

        update(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn run_app(items: &mut [Todo]) -> Result<()> {
    startup()?;
    let result = run_ui(items);
    shutdown()?;
    result?;

    Ok(())
}

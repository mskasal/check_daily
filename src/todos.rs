use core::fmt;
use std::fs::OpenOptions;
use std::io::Read;

use chrono::Local;
use serde::{Deserialize, Serialize};

const DB_FILE_PATH: &str = "db.json";

#[derive(Serialize, Deserialize)]
pub struct Todos {
    pub todos: Vec<Todo>,
}

pub trait TraitTodos {
    fn new() -> Self;

    fn add(&mut self, todo: Todo);

    fn remove(&mut self, id: usize);

    fn mark_completed(&mut self, id: usize);

    fn unmark_completed(&mut self, id: usize);
}

impl TraitTodos for Todos {
    fn new() -> Self {
        Todos { todos: Vec::new() }
    }

    fn remove(&mut self, id: usize) {
        self.todos.retain(|t| t.id != id);
    }

    fn add(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    fn mark_completed(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.toggle_status(true);
        }
    }

    fn unmark_completed(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.toggle_status(false);
        }
    }
}

pub trait Writable {
    fn write(&self) -> std::io::Result<()>;
}

impl Writable for Todos {
    fn write(&self) -> Result<(), std::io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(DB_FILE_PATH)?;

        serde_json::to_writer_pretty(&file, &self)?;

        Ok(())
    }
}

pub trait Readable {
    fn read(&mut self) -> std::io::Result<()>;
}

impl Readable for Todos {
    fn read(&mut self) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(DB_FILE_PATH)?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let is_empty = { buffer.trim().is_empty() };

        if is_empty {
            self.todos = Vec::new();
        }

        match serde_json::from_str::<Todos>(&buffer) {
            Ok(todos) => {
                self.todos = todos.todos;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: usize,
    pub timestamp: i64,
    pub date: String,
    pub text: String,
    pub completed: bool,
}

pub trait TraidTodo {
    fn new(text: String) -> Self;

    fn toggle_status(&mut self, status: bool);
}

impl TraidTodo for Todo {
    fn new(text: String) -> Todo {
        Todo {
            id: generate_unique_id(),
            text,
            completed: false,
            timestamp: get_current_timestamp(),
            date: get_current_date(),
        }
    }

    fn toggle_status(&mut self, status: bool) {
        self.completed = status;
    }
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RED: &str = "\x1B[91m";
        const GREEN_STRIKETHROUGH: &str = "\x1B[32;9m";
        const RESET: &str = "\x1B[0m";

        let color_satus = if self.completed {
            GREEN_STRIKETHROUGH
        } else {
            RED
        };

        write!(f, "{color_satus}{}{RESET}", self.text)
    }
}

fn generate_unique_id() -> usize {
    Local::now().timestamp().try_into().unwrap()
}

fn get_current_timestamp() -> i64 {
    Local::now().timestamp()
}

fn get_current_date() -> String {
    let current_date = Local::now().date_naive();

    let formatted_date = current_date.format("%d/%m/%Y").to_string();
    formatted_date
}

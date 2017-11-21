
#![feature(try_from)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate nom;
extern crate chrono;


use std::cmp::max;
use std::io;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use chrono::NaiveDateTime;

mod models;
mod schema;
mod parsers;

use self::models::{Todo, NewTodo};

fn db_connect() -> SqliteConnection {
    let db_url = "/home/ross/.config/northship/northship.db";
    SqliteConnection::establish(&db_url).expect("Failure connecting to database.")
}

struct Northship {
    database: SqliteConnection,
    mapping: Vec<i32>,
}

impl Northship {
    fn format_todos(&self) -> Result<String, diesel::result::Error> {
        use schema::todos::dsl::*;
        let results = todos.filter(room.eq("roomids"))
            .limit(20)
            .load::<Todo>(&self.database)
            .expect("Error loading Todos");

        let mut maxes = vec![0, 0, 0];
        for todo in results.iter() {
            maxes[0] = max(todo.content.len(), maxes[0]);
            match todo.deadline {
                Some(ref item) => {
                    maxes[1] = max(item.format("%Y-%m-%d").to_string().len(), maxes[1])
                }
                None => {}
            }
            match todo.scheduled {
                Some(ref item) => {
                    maxes[2] = max(item.format("%Y-%m-%d %H:%M:%S").to_string().len(), maxes[2])
                }
                None => {}
            }
        }

        let mut formatted_results: String = String::new();
        formatted_results.push_str(&format!("| #|{todo_title: ^widtha$}|{dead_title: ^widthb$}|{sched_title: \
                                        ^widthc$}|{eff_title:^4}|\n|{rule:-<widthd$}|\n",
                                        todo_title = "TODOs",
                                        widtha = maxes[0] + 2,
                                        dead_title = "Deadline",
                                        widthb = maxes[1] + 2,
                                        sched_title = "Scheduled",
                                        widthc = maxes[2] + 2,
                                        eff_title = "Effort",
                                        rule = "",
                                        widthd = 12 + 6 + maxes[0] + maxes[1] + maxes[2]));
        for (index, todo) in results.iter().enumerate() {
            formatted_results.push_str(&format!("|{number:>2}|{the_todo: ^widtha$}|{dead: ^widthb$}|{sched: ^widthc$}|{eff:>6}|\n|{rule:-<widthd$}|\n",
                                                number = &(index + 1).to_string(),
                                                the_todo = &todo.content,
                                                widtha = maxes[0] + 2,
                                                dead = match todo.deadline {
                                                    Some(ref duedate) => format!("{}", duedate.format("%Y-%m-%d")),
                                                    None => "          ".to_string(),
                                                },
                                                widthb = maxes[1] + 2,
                                                sched = match todo.scheduled {
                                                    Some(ref scheddate) =>format!("{}", scheddate.format("%Y-%m-%d %H:%M:%S")),
                                                    None => "                   ".to_string(),
                                                },
                                                widthc = maxes[2] + 2,
                                                eff = match &todo.effort {
                                                    &Some(ref minutes) => minutes.to_string(),
                                                    &None => "   ".to_string(),
                                                },
                                                rule = "",
                                                widthd = 12 + 6 + maxes[0] + maxes[1] + maxes[2]));
        }
        Ok(formatted_results)
    }

    fn new_todo(&self,
                content: String,
                deadline: Option<NaiveDateTime>,
                scheduled: Option<NaiveDateTime>,
                effort: Option<i32>,
                room: String)
                -> Result<(), diesel::result::Error> {
        use schema::todos;

        let obligation = NewTodo {
            content: &content,
            deadline: deadline,
            scheduled: scheduled,
            effort: effort,
            room: &room,
        };

        diesel::insert(&obligation)
            .into(todos::table)
            .execute(&self.database)
            .expect("Error saving new todo");
        Ok(())
    }
}

fn main() {
    use schema::todos::dsl::*;
    let dbc = db_connect();
    let host = Northship {
        database: dbc,
        mapping: vec![0, 0],
    };
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input_parsed = parsers::command(&input);

        match input_parsed {
            Some(result) => {
                match result {
                    parsers::Command::Todo(todo) => {
                        match host.new_todo(todo.body,
                                            todo.deadline,
                                            todo.scheduled,
                                            None,
                                            "roomids".to_string()) {
                            Ok(()) => println!("New TODO added..."),
                            Err(_) => println!("Error: Couldn't insert TODO in the database."),
                        }
                    }
                    parsers::Command::Agenda => println!("{}", host.format_todos().unwrap()),
                };
            }
            None => println!("Sorry, I didn't catch that. Try again?"),
        }
    }
}

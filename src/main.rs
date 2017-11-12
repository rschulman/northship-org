
#![feature(try_from)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate nom;
extern crate chrono;


use std::convert::TryFrom;
use std::collections::HashMap;
use std::cmp::max;
use std::num::ParseIntError;
use std::io;

use diesel::prelude::*;
use diesel::sqlite;
use diesel::sqlite::SqliteConnection;
use chrono::{DateTime, FixedOffset};
use nom::IResult::{Done, Error as NomError};

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

    fn format_todos(&self) -> Result<String, Error> {
        use schema::todos::dsl::*;
        let results = todos.filter(room.eq("roomids"))
            .limit(20)
            .load::<Todo>(&self.database)
            .expect("Error loading Todos");

        let mut maxes = vec![0, 0, 0];
        for todo in results.iter() {
            maxes[0] = max(todo.content.len(), maxes[0]);
            match todo.deadline {
                Some(ref item) => maxes[1] = max(item.len(), maxes[1]),
                None => {}
            }
            match todo.scheduled {
                Some(ref item) => maxes[2] = max(item.len(), maxes[2]),
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
                                                dead = match &todo.deadline {
                                                    &Some(ref duedate) => &duedate,
                                                    &None => "          ",
                                                },
                                                widthb = maxes[1] + 2,
                                                sched = match &todo.scheduled {
                                                    &Some(ref scheddate) => &scheddate,
                                                    &None => "                   ",
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

    fn set_deadline(&self, cmd: Vec<String>) -> Result<(), String> {
        use schema::todos::dsl::{todos, deadline};

        let which_todo: usize = match cmd[1].parse() {
            Ok(number) => number,
            Err(error) => return Err("Couldn't parse.".to_owned()),
        };
        let db_todo = self.mapping[which_todo - 1];
        let be_done = match cmd[2].parse::<DateTime<FixedOffset>>() {
            Ok(parsed) => parsed,
            Err(error) => return Err("Bad date.".to_owned()),
        };

        let updated = diesel::update(todos.find(db_todo))
            .set(deadline.eq(be_done.to_string()))
            .execute(&self.database)
            .expect(&format!("Unable to find todo {}", db_todo));

        Ok(())
    }

    fn new_todo(&self,
                content: String,
                deadline: Option<String>,
                scheduled: Option<String>,
                effort: Option<i32>,
                room: String)
        -> Result<(), Error> {
            use schema::todos;

            let obligation = NewTodo {
                content: &content,
                deadline: deadline.as_ref().map_or(None, |x| Some(&**x)),
                scheduled: scheduled.as_ref().map_or(None, |x| Some(&**x)),
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
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input_parsed = parsers::command(&input);

    println!("{:?}", input_parsed);
    match input_parsed {
        Done(_, result) => {
            match result {
                parsers::Command::Todo(todo) => {
                    println!("{:?}", &todo.body);
                    host.new_todo(todo.body, todo.deadline, todo.scheduled, None, "rooids".to_string());
                }
            };
        },
        NomError(_) => {
            println!("Sorry, I didn't catch that. Try again?")
        },
        _ => {}
    }
    println!("{}", host.format_todos().unwrap());
}

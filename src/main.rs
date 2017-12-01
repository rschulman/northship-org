#![feature(conservative_impl_trait)]
#![feature(generators)]
#![feature(proc_macro)]
#![feature(try_from)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate nom;
extern crate chrono;
//extern crate futures;
extern crate ruma_client;
extern crate ruma_events;
extern crate ruma_identifiers;
extern crate tokio_core;
extern crate url;
extern crate futures_await as futures;
extern crate hyper;

use std::cmp::max;
use std::io;
use std::convert::TryFrom;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use chrono::NaiveDateTime;

use futures::prelude::*;
use ruma_client::api::r0;
use ruma_client::Client;
use ruma_events::EventType;
use ruma_events::collections::all::RoomEvent;
use ruma_events::room::message::{MessageType, MessageEvent, MessageEventContent,
                                 TextMessageEventContent};
use ruma_identifiers::{RoomAliasId, RoomId};
use tokio_core::reactor::{Core as TokioCore, Handle as TokioHandle};
use url::Url;
use hyper::client::Connect;

mod models;
mod schema;
mod parsers;

use self::models::{Todo, NewTodo};

// from https://stackoverflow.com/a/43992218/1592377
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move || $body
        }
        );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move |$(clone!(@param $p),)+| $body
        }
        );
}

fn db_connect() -> SqliteConnection {
    let db_url = "/home/ross/.config/northship/northship.db";
    SqliteConnection::establish(&db_url).expect("Failure connecting to database.")
}

struct Northship {
    database: SqliteConnection,
    mapping: Vec<i32>,
}

/*fn send_matrix_message<C: Connect>
  (client: ruma_client::Client<C>,
  message: String,
  room: RoomId)
  -> impl Future<Item = (), Error = ruma_client::Error> + 'static {

  }*/

impl Northship {
    fn matrix_loop(&self,
                   tokio_handle: &TokioHandle,
                   homeserver_url: Url,
                   username: String,
                   password: String)
                   -> impl Future<Item = (), Error = ruma_client::Error> + 'static {
        let client = ruma_client::Client::https(tokio_handle, homeserver_url, None).unwrap();

        client.log_in(username, password).and_then(
            clone!(client => move |_| {
        client.sync(None, None, true).skip(1).for_each(|res| {
                for (room_id, room) in res.rooms.join {
                    for event in room.timeline.events {
                        if let RoomEvent::RoomMessage(MessageEvent {
                            content: MessageEventContent::Text(
                                         TextMessageEventContent {
                                             body: msg_body,
                                             ..
                                         }
                                         ),
                                         user_id,
                                         ..
                        }) = event {
                            let input_parsed = parsers::command(&msg_body);
                            match input_parsed {
                                Some(result) => {
                                    match result {
                                        parsers::Command::Todo(todo) => {
                                            match self.new_todo(todo.body,
                                                                todo.deadline,
                                                                todo.scheduled,
                                                                None,
                                                                room_id.to_string()) {
                                                Ok(()) => { 
                                                    r0::send::send_message_event::call(client.clone(),
                                                r0::send::send_message_event::Request {
                                                    room_id: room_id,
                                                    event_type: EventType::RoomMessage,
                                                    txn_id: "1".to_owned(), // TODO Probably problematic
                                                    data:
                                                        MessageEventContent::Text(TextMessageEventContent {
                                                            body: "New TODO added".to_owned(),
                                                            msgtype: MessageType::Text,
                                                        }),
                                                });
                                                },
                                                Err(_) => {
                                                    r0::send::send_message_event::call(client.clone(),
                                                r0::send::send_message_event::Request {
                                                    room_id: room_id,
                                                    event_type: EventType::RoomMessage,
                                                    txn_id: "1".to_owned(), // TODO Probably problematic
                                                    data:
                                                        MessageEventContent::Text(TextMessageEventContent {
                                                            body: "Error adding TODO to datbaase.".to_owned(),
                                                            msgtype: MessageType::Text,
                                                        }),
                                                });
                                                },



                                            };
                                        },
                                        parsers::Command::Agenda => {
                                            r0::send::send_message_event::call(client.clone(),
                                            r0::send::send_message_event::Request {
                                                room_id: room_id,
                                                event_type: EventType::RoomMessage,
                                                txn_id: "1".to_owned(), // TODO Probably problematic
                                                data:
                                                    MessageEventContent::Text(TextMessageEventContent {
                                                        body: self.format_todos().unwrap().to_owned(),
                                                        msgtype: MessageType::Text,
                                                    }),
                                            });
                                        },
                                    };
                                },
                                None => {
                                    r0::send::send_message_event::call(client.clone(),
                                r0::send::send_message_event::Request {
                                    room_id: room_id,
                                    event_type: EventType::RoomMessage,
                                    txn_id: "1".to_owned(), // TODO Probably problematic
                                    data:
                                        MessageEventContent::Text(TextMessageEventContent {
                                            body: "Sorry, I didn't catch that. Try again?".to_owned(),
                                            msgtype: MessageType::Text,
                                        }),
                                });
                                },

                            };
                        }
                    }

                }
        Ok(())
            })
            }),
        )
    }

    fn format_todos(&self) -> Result<String, diesel::result::Error> {
        use schema::todos::dsl::*;
        let results = todos.filter(room.eq("roomids"))
            .limit(20)
            .load::<Todo>(&self.database)
            .expect("Error loading Todos");

        let mut maxes = vec![0, "Deadline".len(), "Scheduled".len()];
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
                                                    None => "".to_string(),
                                                },
                                                widthb = maxes[1] + 2,
                                                sched = match todo.scheduled {
                                                    Some(ref scheddate) =>format!("{}", scheddate.format("%Y-%m-%d %H:%M:%S")),
                                                    None => "".to_string(),
                                                },
                                                widthc = maxes[2] + 2,
                                                eff = match &todo.effort {
                                                    &Some(ref minutes) => minutes.to_string(),
                                                    &None => "  ".to_string(),
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

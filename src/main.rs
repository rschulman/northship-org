
#![feature(try_from)]
#![feature(conservative_impl_trait)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate futures;
extern crate hyper;
extern crate ruma_client;
extern crate ruma_client_api;
extern crate ruma_events;
extern crate ruma_identifiers;
extern crate tokio_core;
extern crate url;
extern crate chrono;


use std::convert::TryFrom;
use std::collections::HashMap;
use std::cmp::max;

use diesel::prelude::*;
use diesel::sqlite;
use diesel::sqlite::SqliteConnection;
use futures::Future;
use hyper::client::Connect;
use ruma_client::{Client, Error};
use ruma_client::api::r0;
use ruma_events::EventType;
use ruma_events::room::message::{MessageEventContent, MessageType, TextMessageEventContent};
use ruma_identifiers::RoomAliasId;
use tokio_core::reactor::Core;
use url::Url;

mod models;
mod schema;

use self::models::{Todo, NewTodo};

fn db_connect() -> SqliteConnection {
    let db_url = "/home/ross/.config/northship/northship.db";
    SqliteConnection::establish(&db_url).expect("Failure connecting to database.")
}

struct Northship {
    database: SqliteConnection,
    mapping: Vec<u32>,
}

impl Northship {
    fn parse_todo(&self, cmd: Vec<String>) -> Result<String, Error> {
        let tokens = cmd.split_whitespace();
        let deadpos = tokens.position(|&word| word == "DEADLINE");
        let schedpos = tokens.position(|&word| word == "SCHEDULED");

//        let content = tokens.take_while(|&word| word != "DEADLINE" && word != "SCHEDULED").skip(1).collect::<Vec<&str>>().join(" ");
//        let deadline = tokens.skip(content.len() + 1).take_while(|&word| word != "SCHEDULED").collection::<Vec<&str>>().join(" ");
    }

    fn parse_cmd(&self, input: String) -> Result<String, Error> {
        let words = input.split_whitespace();
        let matches = testset.matches(words.nth(0));

        match matches.matched(0) {
            "todo" | "TODO" => self.parse_todo(matches),
            "done" => self.mark_done(matches),
            "deadline" => self.set_deadline(conn, matches),
            "schedule" => self.set_schedule(matches),
            "list" => Ok(self.format_todos(conn)),
            "agenda" => self.format_agenda(conn),
        }
    }

    fn format_todos(&self) -> Result<String, Error> {
        use schema::todos::dsl::*;
        let results = todos.filter(room.eq("roomids"))
        .limit(20)
        .load::<Todo>(self.&database)
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

    fn set_deadline(&self, cmd: Vec<String>) -> Result<(), Error> {
        let which_todo = match cmd[1].to_string() {
            Ok(number) => number,
            Err(error) => return error,
        };
        let db_todo = self.mapping[which_todo - 1];
        let be_done = match cmd[2].parse::<DateTime<FixedOffset>>() {
            Ok(parsed) => parsed,
            Err(error) => return error,
        };

        let updated = diesel::update(todos.find(db_todo))
            .set(deadline.eq(be_done.to_string()))
            .get_result::<Todo>(self.&database)
            .expect(&format!("Unable to find todo {}", db_todo));

        Ok(())
    }

    fn new_todo(&self,
                content: String,
                deadline: Option<&str>,
                scheduled: Option<&str>,
                effort: Option<i32>,
                room: String)
                -> Result<(), Error> {
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
            .execute(self.&database)
            .expect("Error saving new todo");
        Ok(())
    }
}


/*
fn run<'a, C: Connect>(conn: &'a Client<C>)
                       -> impl Future<Item = (), Error = ruma_client::Error> + 'a {
    use r0::sync::sync_events;
    use r0::membership::join_room_by_id;
    use r0::send::send_message_event;
    let mut since_time: Option<String> = None;
    loop {
        sync_events::call(conn,
                          sync_events::Request {
                              filter: None,
                              since: since_time,
                              full_state: None,
                              set_presence: None,
                              timeout: None, // Should probably set a timeout...
                          })
            .and_then(move |response| {
                since_time = Some(response.next_batch);
                if response.rooms.invite.len() > 0 {
                    for room in response.rooms.invite.keys() {
                        join_room_by_id::call(conn,
                                              join_room_by_id::Request {
                                                  room_id: room.clone(),
                                                  third_party_signed: None,
                                              })
                            .and_then(|response| {
                                let msg = MessageEventContent::Text(TextMessageEventContent {
                                    body: "Hello, I'm Northship. If you need help, just say \
                                           `help`."
                                        .to_owned(),
                                    msgtype: MessageType::Text,
                                });
                                send_message_event::call(conn,
                                                         send_message_event::Request {
                                                             room_id: room.clone(),
                                                             event_type: EventType::RoomMessage,
                                                             txn_id: "1".to_owned(),
                                                             data: msg,
                                                         });
                            });
                    }
                }
            });
    }
    futures::done(Ok(()))
}
*/

fn main() {
    use schema::todos::dsl::*;
    let dbc = db_connect();

    new_todo(&dbc,
             "Solve world hunger".to_string(),
             Some("2017-07-10"),
             Some("2017-07-10 20:00:00"),
             None,
             "roomids".to_string());
    new_todo(&dbc,
             "Fix the refrigerator and dryer".to_string(),
             Some("2017-07-31"),
             None,
             Some(90),
             "roomids".to_string());

   println!("{}", format_todos(&dbc, results).unwrap());
    // let mut core = Core::new().unwrap();
    // let handle = core.handle();
    // let server = Url::parse("https://matrix.westwork.org/").unwrap();

    // let mut client = Client::new(&handle, server);

    // core.run(client.login("northship".to_string(), "thisisapass".to_string())
    //         .and_then(run(&client)))
    //     .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_todo() {
        let test_string = "TODO go to the grocery store";

    }
}

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


use std::convert::TryFrom;
use std::collections::HashMap;

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

fn new_todo(conn: &SqliteConnection,
            content: String,
            deadline: Option<String>,
            scheduled: Option<String>,
            effort: Option<i32>,
            room: String)
            -> Result<(), Error> {
    use schema::todos;

    let obligation = NewTodo {
        content: &content,
        deadline: deadline,
        scheduled: scheduled,
        effort: effort,
        room: room,
    };

    diesel::insert(&obligation).into(todos::table).get_results(conn).expect("Error saving new todo")
}

struct Northship {
    database: SqliteConnection,
}



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

fn main() {
    let dbc = db_connect();

    new_todo(&dbc,
             "Solve world hunger",
             Some("2017-07-10"),
             Some("2017-07-10 20:00:00"),
             None,
             "roomids");
    // let mut core = Core::new().unwrap();
    // let handle = core.handle();
    // let server = Url::parse("https://matrix.westwork.org/").unwrap();

    // let mut client = Client::new(&handle, server);

    // core.run(client.login("northship".to_string(), "thisisapass".to_string())
    //         .and_then(run(&client)))
    //     .unwrap();
}
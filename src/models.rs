use schema::todos;
use chrono;

#[derive(Queryable)]
pub struct Todo {
    pub id: i32,
    pub content: String,
    pub deadline: Option<chrono::NaiveDateTime>,
    pub scheduled: Option<chrono::NaiveDateTime>,
    pub effort: Option<i32>,
    pub room: String,
}

#[derive(Insertable)]
#[table_name="todos"]
pub struct NewTodo<'a> {
    pub content: &'a str,
    pub deadline: Option<chrono::NaiveDateTime>,
    pub scheduled: Option<chrono::NaiveDateTime>,
    pub effort: Option<i32>,
    pub room: &'a str,
}

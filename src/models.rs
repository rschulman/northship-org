use schema::todos;

#[derive(Queryable)]
pub struct Todo {
    pub id: i32,
    pub content: String,
    pub deadline: Option<String>,
    pub scheduled: Option<String>,
    pub effort: Option<i32>,
    pub room: String,
}

#[derive(Insertable)]
#[table_name="todos"]
pub struct NewTodo<'a> {
    pub content: &'a str,
    pub deadline: Option<&'a str>,
    pub scheduled: Option<&'a str>,
    pub effort: Option<i32>,
    pub room: &'a str,
}
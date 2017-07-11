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
    content: &'a str,
    deadline: &'a Option<&'a str>,
    scheduled: &'a Option<&'a str>,
    effort: &'a Option<i32>,
    room: &'a str,
}
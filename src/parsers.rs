use nom::*;

#[derive(Debug, PartialEq)]
pub enum Command {
    Todo(TodoCmd),
}

#[derive(Debug, PartialEq)]
pub struct TodoCmd {
    pub body: String,
    pub deadline: Option<String>,
    pub scheduled: Option<String>
}

named!(todo<&str, &str>, tag_no_case_s!("TODO "));
named!(todo_text<&str, &str>, alt_complete!(take_until_s!(" DEADLINE") | take_until_s!(" SCHEDULED") | rest_s));
named!(deadline<&str, &str>, do_parse!(
        t: tag_no_case_s!(" DEADLINE ") >>
        d: alt_complete!(take_until_s!(" SCHEDULED") | rest_s) >>
        (d)
        ));
named!(scheduled<&str, &str>, do_parse!(
        t: tag_no_case_s!(" SCHEDULED ") >>
        d: alt_complete!(take_until_s!(" DEADLINE") | rest_s) >>
        (d)
        ));

named!(pub command<&str, Command>, complete!(
        do_parse!(
            todo >>
            c: todo_text >>
            d: opt!(deadline) >>
            s: opt!(scheduled) >>
            (Command::Todo(TodoCmd {
                body: c.to_string(),
                deadline: match d {
                    Some(deadline) => Some(deadline.to_string()),
                    None => None
                },
                scheduled: match s {
                    Some(scheduled) => Some(scheduled.to_string()),
                    None => None
                },
            }))
            )));
//pub fn command(input: &str) -> Command {



#[cfg(test)]
mod tests {
    use nom::*;
    use super::*;

    #[test]
    fn find_todo() {
        assert_eq!(todo("TODO "), IResult::Done("", "TODO "));
    }
    #[test]
    fn find_todo_text_eof() {
        assert_eq!(todo_text("go to the grocery store"),
        IResult::Done("", "go to the grocery store"));
    }
    #[test]
    fn find_todo_text_deadline() {
        assert_eq!(todo_text("go to the grocery store DEADLINE today"),
        IResult::Done(" DEADLINE today", "go to the grocery store"));
    }
    #[test]
    fn find_todo_text_scheduled() {
        assert_eq!(todo_text("go to the grocery store SCHEDULED today"),
        IResult::Done(" SCHEDULED today", "go to the grocery store"));
    }
    #[test]
    fn find_todo_text_neither() {
        // Purposefully short string to force the alt! incomplete error
        assert_eq!(todo_text("go home"),
        IResult::Done("", "go home"));
    }
    #[test]
    fn find_deadline_text_scheduled() {
        assert_eq!(deadline(" DEADLINE 2017-6-8 SCHEDULED today"),
        IResult::Done(" SCHEDULED today", "2017-6-8"));
    }
    #[test]
    fn find_deadline_text_no_scheduled() {
        assert_eq!(deadline(" DEADLINE 2017-6-8"),
        IResult::Done("", "2017-6-8"));
    }
    #[test]
    fn find_scheduled_text_no_deadline() {
        assert_eq!(scheduled(" SCHEDULED tomorrow"),
        IResult::Done("", "tomorrow"));
    }
    #[test]
    fn command_todo() {
        assert_eq!(command("TODO go to the grocery store DEADLINE 2017-08-19 SCHEDULED 2017-08-18 14:30").unwrap().1,
        Command::Todo(TodoCmd {
            body: "go to the grocery store".to_string(),
            deadline: Some("2017-08-19".to_string()),
            scheduled: Some("2017-08-18 14:30".to_string())
        })
        );
    }
    #[test]
    fn command_only_content() {
        assert_eq!(command("TODO apply to college").unwrap().1,
        Command::Todo(TodoCmd {
            body: "apply to college".to_string(),
            deadline: None,
            scheduled: None
        })
        );
    }
    #[test]
    fn command_only_deadline() {
        assert_eq!(command("TODO apply to college DEADLINE 2017-08-19").unwrap().1,
        Command::Todo(TodoCmd {
            body: "apply to college".to_string(),
            deadline: Some("2017-08-19".to_string()),
            scheduled: None
        })
        );
    }


}

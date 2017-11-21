use nom::*;
use std::str::FromStr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Debug, PartialEq)]
pub enum Command {
    Todo(TodoCmd),
    Agenda,
}

#[derive(Debug, PartialEq)]
pub struct TodoCmd {
    pub body: String,
    pub deadline: Option<NaiveDateTime>,
    pub scheduled: Option<NaiveDateTime>,
}

named!(time<&str, NaiveTime>, do_parse!(
        h: digit >>
        tag_s!(":") >>
        min: digit >>
        tag_s!(":") >>
        s: digit >>
        (NaiveTime::from_hms(u32::from_str(h).unwrap(), u32::from_str(min).unwrap(), u32::from_str(s).unwrap())
        )));
named!(date<&str, NaiveDateTime>, do_parse!(
        y: digit >>
        tag_s!("-") >>
        m: digit >>
        tag_s!("-") >>
        d: digit >>
        new_time: opt!(do_parse!(
                alt_complete!(tag_s!("T") | tag_s!(" ")) >>
                t: time >>
                (t))) >>
        (
            NaiveDateTime::new(
                NaiveDate::from_ymd(
                    i32::from_str(y).unwrap(),
                    u32::from_str(m).unwrap(),
                    u32::from_str(d).unwrap()
                    ),
                    new_time.unwrap_or(NaiveTime::from_hms(0,0,0)))
        )));
named!(todo<&str, &str>, tag_no_case_s!("TODO "));
named!(todo_text<&str, &str>, alt_complete!(take_until_s!(" DEADLINE") | take_until_s!(" SCHEDULED") | rest_s));
named!(deadline<&str, NaiveDateTime>, do_parse!(
        t: tag_no_case_s!(" DEADLINE ") >>
        d: date >>
        (d)
        ));
named!(scheduled<&str, NaiveDateTime>, do_parse!(
        t: tag_no_case_s!(" SCHEDULED ") >>
        d: date >>
        (d)
        ));

named!(todo_cmd<&str, Command>, 
       do_parse!(
           c: todo_text >>
           d: opt!(complete!(deadline)) >>
           s: opt!(complete!(scheduled)) >>
           (Command::Todo(TodoCmd {
               body: c.to_string(),
               deadline: d,
               scheduled: s,
           } ))
       ));

pub fn command(i: &str) -> Option<Command> {
    match i.split_whitespace().next() {
        Some("TODO") => {
            match todo_cmd(i.get(5..).unwrap()) {
                IResult::Done(_, cmd) => Some(cmd),
                IResult::Incomplete(_) => None,
                IResult::Error(_) => None,
            }
        }
        Some("AGENDA") => Some(Command::Agenda),
        Some(_) => None,
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use nom::*;
    use super::*;

    #[test]
    fn agenda() {
        assert_eq!(command("AGENDA"), Some(Command::Agenda));
    }
    #[test]
    fn date_no_time() {
        assert_eq!(date("2017-4-23"),
                   IResult::Done("", NaiveDate::from_ymd(2017, 4, 23).and_hms(0, 0, 0)));
    }
    #[test]
    fn date_and_time() {
        assert_eq!(date("2017-4-23 16:00:45"),
                   IResult::Done("", NaiveDate::from_ymd(2017, 4, 23).and_hms(16, 0, 45)));
    }
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
        assert_eq!(todo_text("go home"), IResult::Done("", "go home"));
    }
    /* Leaving fuzzy date parsing for later
       #[test]
       fn find_deadline_text_scheduled() {
       assert_eq!(deadline(" DEADLINE 2017-6-8 SCHEDULED today"),
       IResult::Done(" SCHEDULED today", NaiveDate::from_ymd(2017, 6, 8).and_hms(0, 0, 0)));
       }
       */
    #[test]
    fn find_deadline_text_no_scheduled() {
        assert_eq!(deadline(" DEADLINE 2017-6-8"),
                   IResult::Done("", NaiveDate::from_ymd(2017, 6, 8).and_hms(0, 0, 0)));
    }
    /*   This is a good test to someday pass but leaving fuzzy date parsing for later
         #[test]
         fn find_scheduled_text_no_deadline() {
         assert_eq!(scheduled(" SCHEDULED tomorrow"),
         IResult::Done("", "tomorrow"));
         }
         */
    #[test]
    fn command_todo() {
        assert_eq!(command("TODO go to the grocery store DEADLINE 2017-08-19 SCHEDULED \
                            2017-08-18 14:30"),
                   Some(Command::Todo(TodoCmd {
                       body: "go to the grocery store".to_string(),
                       deadline: Some(NaiveDate::from_ymd(2017, 8, 19).and_hms(0, 0, 0)),
                       scheduled: Some(NaiveDate::from_ymd(2017, 8, 18).and_hms(14, 30, 0)),
                   })));
    }
    #[test]
    fn command_only_content() {
        assert_eq!(command("TODO apply to college"),
                   Some(Command::Todo(TodoCmd {
                       body: "apply to college".to_string(),
                       deadline: None,
                       scheduled: None,
                   })));
    }
    #[test]
    fn command_only_deadline() {
        assert_eq!(command("TODO apply to college DEADLINE 2017-08-19"),
                   Some(Command::Todo(TodoCmd {
                       body: "apply to college".to_string(),
                       deadline: Some(NaiveDate::from_ymd(2017, 8, 19).and_hms(0, 0, 0)),
                       scheduled: None,
                   })));
    }


}

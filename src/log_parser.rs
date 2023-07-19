//Parser reads config files with regular expressions for each type of log source.
//When an event is passed in, it is tagged with a type, that type is matched against regex definitions read it from the config files.
//A seperate parser is created for each definiion.  parsers are in a tree format, the message is passed into the root, partially decoded
//and then passed up the tree based on values parsed in the current level.


use crate::log_event;
use std::collections::HashMap;
use serde_json::{Result, Value};
use std::str;

pub struct Parser {
    pub log_type: log_event::LogType,
    pub definition_group HashMap
}

pub fn parse_syslog(event: log_event::LogEvent) -> log_event::AnalyzedEvent {
    println!("{}", &event.msg);
    let mut analyzed_event = log_event::AnalyzedEvent{event: event, data: HashMap::new(), log_type: log_event::LogType::SYSLOG5424};
    let mut ptr = extract_pri(&mut analyzed_event);
    return analyzed_event;
}
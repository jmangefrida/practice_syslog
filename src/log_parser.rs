use crate::log_event;
use std::collections::HashMap;
use serde_json::{Result, Value};



pub fn parse_syslog(event: log_event::LogEvent) -> log_event::AnalyzedEvent {
    println!("{}", &event.msg);
    let mut analyzed_event = log_event::AnalyzedEvent{event: event, data: HashMap::new()};
    extract_pri(&mut analyzed_event);
    return analyzed_event;
}

fn extract_pri(event: &mut log_event::AnalyzedEvent) {
    event.data.get_mut("pri").get_or_insert(&mut "value".to_string());
}



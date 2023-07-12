use crate::log_event;
use std::collections::HashMap;
use serde_json::{Result, Value};
use std::str;


pub fn parse_syslog(event: log_event::LogEvent) -> log_event::AnalyzedEvent {
    println!("{}", &event.msg);
    let mut analyzed_event = log_event::AnalyzedEvent{event: event, data: HashMap::new()};
    let mut ptr = extract_pri(&mut analyzed_event);
    return analyzed_event;
}

fn extract_pri(event: &mut log_event::AnalyzedEvent) -> usize {
    
    let mut ptr: usize = 0;

    if event.event.msg.starts_with("<") {
        let end = event.event.msg.find(">");
        ptr = match end {
            Some(n) => n,
            None => 0   
        };


        if ptr > 0 {
            let pri: i32  = event.event.msg["<".len()..ptr].parse().unwrap();
            let codes = decode_pri(pri);
            event.data.insert("facility".to_string(), codes.0.to_string());
            event.data.insert("severity".to_string(), codes.1.to_string());
            println!("{},{}", codes.0, codes.1);
        }
        

    }

    //event.data.get_mut("pri").get_or_insert(&mut "value".to_string());
    ptr

}

fn decode_pri(pri: i32) -> (i32, i32) {
    //let logtype = log_event::LogType::JSON;
    let facility: i32 = pri / 8;
    let severity: i32 = pri % 8;
    (facility, severity)
}


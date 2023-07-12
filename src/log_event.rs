use uuid::Uuid;
use crate::duration::Duration;
//use scylla::FromRow;
use scylla::ValueList;
use std::collections::HashMap;
use serde_json::{Result, Value};

#[derive(Debug)]
#[non_exhaustive]
pub enum LogType {
    JSON = 0,
    SYSLOG3164 = 1,
    SYSLOG5424 = 2
}

pub enum Facitlity {
    KERNEL_MESSAGE = 0,
    USER_MESSAGE = 1,
    MAIL_SYSTEM  = 2,
    SYSTEM_DAEMON = 3,
    SECURITY = 4,
    INTERNAL_SYSLOG_Msg = 5,
    LINE_PRNT_SUB = 6,
    NET_NEWS_SUB = 7,
    UUCP_SUB = 8,
    CLOCK_DAEMON = 9,
    SECURITY_AUTH = 10,
    FTP_DAEMON = 11,
    NTP_SUB = 12,
    LOG_AUDIT = 13,
    LOG_ALERT = 14,
    CLOCK_DAEMON2 = 15,
    LOCAL0 = 16,
    LOCAL1 = 17,
    LOCAL2 = 18,
    LOCAL3 = 19,
    LOCAL4 = 20,
    LOCAL5 = 21,
    LOCAL6 = 22,
    LOCAL7 = 23,
}

pub enum Severity {
    EMERGENCY = 0,
    ALERT = 1,
    CRITICAL = 2,
    ERROR = 3,
    WARNING = 4,
    NOTICE = 5,
    INFORMATIONAL = 6,
    DEBUG = 7
}




#[derive(Debug, ValueList)]
pub struct LogEvent {
    pub id: Uuid,
    pub ingest_time: Duration,
    pub source: String,
    pub tag: String,
    pub msg: String

}

pub struct AnalyzedEvent{
    pub event: LogEvent,
    pub data: HashMap<String, String>
}

impl AnalyzedEvent {
    pub fn parse(mut self) {
        let mut ptr: usize = 0;
        ptr = self.extract_pri();
        ptr = self.extract_date(ptr);
        println!("{}", ptr);
    }

    pub fn extract_pri(&mut self) -> usize {
        
        let mut ptr: usize = 0;
        if self.event.msg.starts_with("<") {
            let end = self.event.msg.find(">");
            ptr = match end {
                Some(n) => n,
                None => 0   
            };
    
    
            if ptr > 0 {
                let pri: i32  = self.event.msg["<".len()..ptr].parse().unwrap();
                self.decode_pri(pri);
                println!("{},{}", self.data["facility"], self.data["severity"]);
            }
            
        }

        ptr
    }


    pub fn decode_pri(&mut self, pri: i32) {
    //let logtype = log_event::LogType::JSON;
        let facility: i32 = pri / 8;
        let severity: i32 = pri % 8;
        self.data.insert("facility".to_string(), facility.to_string());
        self.data.insert("severity".to_string(), severity.to_string());

    }

    pub fn extract_date(&mut self, mut ptr: usize) -> usize {
       self.event.msg.is_ascii();
        ptr
    }
}
use uuid::Uuid;
use crate::duration::Duration;
//use scylla::FromRow;
use scylla::ValueList;
use std::collections::HashMap;
//use std::os::unix::process;
use serde_json::{Result, Value};
//use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
//use chrono::format::ParseError;
//use regex::Regex;

#[derive(Debug)]
#[non_exhaustive]
pub enum LogType {
    JSON = 0,
    SYSLOG3164 = 1,
    SYSLOG5424 = 2,
    UNKNOWN = 3
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

const MONTHS: [&str; 12] = [
    "JAN",
    "FEB",
    "Mar",
    "Apr",
    "MAY",
    "JUN",
    "JUL",
    "AUG",
    "SEP",
    "OCT",
    "NOV",
    "DEC",
    ];




pub struct LogEvent {
    pub id: Uuid,
    pub ingest_time: Duration,
    pub source: String,
    pub tags: Vec<String>,
    pub msg: String,
    pub data: HashMap<String, Value>,
    pub log_type: String

}
#[derive(Debug, ValueList)]
pub struct DbEvent {
    pub id: Uuid,
    pub ingest_time: Duration,
    pub source: String,
    pub tags: Vec<String>,
    pub msg: String,
    pub original: String,
    pub log_type: String

}
pub struct AnalyzedEvent{
    pub event: LogEvent,
    pub data: HashMap<String, String>,
    pub log_type: LogType
}

impl AnalyzedEvent {
    pub fn parse(mut self) {
        let mut ptr: usize = 0;
        let is_ascii = self.event.msg.is_ascii();
        ptr = self.extract_pri();
        ptr = self.extract_version(ptr);
        ptr = self.extract_date_time(ptr);
        ptr = self.extract_hostname(ptr);
        //ptr = self.extract_process(ptr);
        
        //println!("{:?}", self.data);
        
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
                //println!("{},{}", self.data["facility"], self.data["severity"]);
            }
            
            return ptr + 1;
        }

        return 0;

        
    }


    pub fn decode_pri(&mut self, pri: i32) {
    //let logtype = log_event::LogType::JSON;
        let facility: i32 = pri / 8;
        let severity: i32 = pri % 8;
        self.data.insert("facility".to_string(), facility.to_string());
        self.data.insert("severity".to_string(), severity.to_string());

    }

    pub fn extract_version(&mut self, mut ptr: usize) -> usize {
        let version: String = self.event.msg[ptr..ptr+1].parse().unwrap();
        if version == "1" {
            self.data.insert("syslog_version".to_string(), "1".to_string());
            ptr += 2;
            return ptr;
        } else {
            self.data.insert("syslog_version".to_string(), "0".to_string());
        }
        return ptr;
    }
    
    pub fn extract_date_time(&mut self, mut ptr: usize) -> usize {

        //let sub_msg : String = self.event.msg[ptr..].parse().unwrap();
        let mut sub_msg: String;
        let mut end_ptr: usize;
        if self.data["syslog_version"] == "1"{

            end_ptr = ptr + 25;
            sub_msg = self.event.msg[ptr..end_ptr].parse().unwrap();
            //let date_time = DateTime::parse_from_rfc3339(&sub_msg).unwrap();
            
        } else {
            end_ptr = ptr + 15;
            sub_msg = self.event.msg[ptr..end_ptr].parse().unwrap();
            //let date_time = DateTime::parse_from_str(&sub_msg, "%b %d %H:%M:%S");
        }
        self.data.insert("event_time".to_string(), sub_msg.clone());
        //println!("{}", sub_msg);
        if MONTHS.contains(&sub_msg.as_str()){

        }
        end_ptr + 1
    }

    pub fn extract_hostname(&mut self, mut ptr: usize) -> usize {
        let sub_msg: String = self.event.msg[ptr..].parse().unwrap();
        let end_ptr: usize = sub_msg.find(" ").unwrap() + ptr;
        let hostname: String = self.event.msg[ptr..end_ptr].parse().unwrap();

        self.data.insert("hostname".to_string(), hostname.clone());

        end_ptr + 1
    }

    pub fn extract_process(&mut self, mut ptr: usize) -> usize {
        let mut sub_msg: String = self.event.msg[ptr..].parse().unwrap();
        let find_result = sub_msg.find(char::is_alphanumeric);
        let ptr_padding = match find_result {
            Some(n) => n,
            None => 0
        };
        if ptr_padding == 0 {
            return ptr
        }
        sub_msg = sub_msg[ptr_padding..].parse().unwrap();
        //sub_msg = sub_msg.trim().to_string();
        let end_ptr: usize = sub_msg.find(" ").unwrap() + ptr + ptr_padding;
        let process: String = self.event.msg[ptr..end_ptr].parse().unwrap();
        let ptr_id = sub_msg.find("[");
        let process_id =  match ptr_id {
            Some(n) => n,
            None => 0
        };
        if process_id > 0 && process.len() > 1 {
            self.data.insert("process_id".to_string(), process[process_id+1..process.len()-2].parse().unwrap());
            self.data.insert("process".to_string(), process[0..process_id].parse().unwrap());
        } else {
            self.data.insert("process".to_string(), process);
        }

        end_ptr

    }
}
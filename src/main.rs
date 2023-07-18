use crate::result::Result;
//use chrono::NaiveDateTime;
//use scylla::Session;
//use syslog_loose::Message;
//use syslog_rfc5424::message;
//use syslog_rfc5424::SyslogMessage;
//use std::net::UdpSocket;
//use std::str;
//use uuid::Uuid;
mod db;
mod result;
//use crate::duration::Duration;
//use chrono::Duration;
mod duration;
//use std::time::SystemTime;
mod log_event;
mod ingest;
use std::thread;
use tokio::runtime::Runtime;
mod log_parser;

#[tokio::main]
async fn main() -> Result<()> {
    let db_uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "192.168.122.206:9042".to_string());
    let sock_uri: String = "192.168.1.50:10514".to_string();
    let listener = ingest::SyslogListener{db_uri, sock_uri};
    
    let handle = thread::spawn(move || {
        let _v = Runtime::new().unwrap().block_on(listener.listen());
    });
    
    handle.join().unwrap();

    Ok(())
    
}


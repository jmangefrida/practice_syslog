use crate::{result::Result, ingest::SyslogListener};
//use serde_json::Value;
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
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
mod config;
mod parser;
use crate::config::Config;

//#[tokio::main]
 fn main() -> Result<()> {
    //let flut_config: config::Config = config::Config::build().expect("Error: Could not read config");
    //let tester = flut_config.bind_ip;
    //let db_uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "192.168.122.206:9042".to_string());
    //let sock_uri: String = "192.168.1.50:10514".to_string();
    let config: Config = Config::build();
    let parsers = config.clone().build_parsers();
    /*let config: Value = match config::read_config() {
        Ok(value) => value,
        Err(err) => err
    };*/

    let mut listeners: Vec<SyslogListener> = vec![];
    let mut handles: Vec<JoinHandle<()>> = vec![];
    for ingester in config.ingesters {
        listeners.push(ingest::SyslogListener{
            db_uri:config.db_uri.clone(),
            sock_uri:ingester.bind_addr,
            parser:parsers[&ingester.parser].clone(),
            tags: ingester.tags
         })
    }
    //let listener = ingest::SyslogListener{db_uri:config.db_uri, sock_uri:config.bind_ip, parser: fiters[0].clone()};
    
    //let handle = thread::spawn(move || {
    //    let _v = Runtime::new().unwrap().block_on(listener.listen());
    //});

    for listener in listeners {
        handles.push(thread::spawn(move || {
            let _v = Runtime::new().unwrap().block_on(listener.listen());
        }))
    }

    print!("stuff");
    
    for handle in handles {
        handle.join().unwrap();
    }
    //handle.join().unwrap();
    print!("more stuff");
    Ok(())
    
}


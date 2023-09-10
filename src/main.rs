use crate::{result::Result, ingest::{SyslogListener, JSONListner}, config::Ingester, query::RealtimeQuerier};
use serde_json::{json, Value};
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
mod tcp_pool;
use std::{thread::{self, JoinHandle}, sync::mpsc};
use tokio::runtime::Runtime;
use tokio::join;
mod config;
mod parser;
mod pool;
mod query;
use crate::config::Config;
use std::time;

//#[tokio::main]
 fn main() -> Result<()> {

    let mut handles: Vec<JoinHandle<()>> = vec![];

    //let tempv = json!(("hellow", query::ACTION::ALERT("test".to_string())));

    //println!("{}", tempv);

    let config: Config = Config::build();
    let parsers = config.clone().build_parsers();
    /*let config: Value = match config::read_config() {
        Ok(value) => value,
        Err(err) => err
    };*/

    let (query_sender, query_receiver) = mpsc::channel();
    let mut querier: RealtimeQuerier = query::RealtimeQuerier::new(config.queries, query_receiver);

    handles.push(thread::spawn(move || {
        querier.start();
    }));


    let mut listeners: Vec<SyslogListener> = vec![];
    //let mut tcp_listeners: Vec<JSONListner> = vec![];
    
    for ingester in config.ingesters {
        listeners.push(ingest::SyslogListener{
            db_uri:config.db_uri.clone(),
            sock_uri:ingester.bind_addr,
            parser:parsers[&ingester.parser].clone(),
            tags: ingester.tags,
            sender: query_sender.clone(),
            threads: ingester.threads
         })
    }


    //let listener = ingest::SyslogListener{db_uri:config.db_uri, sock_uri:config.bind_ip, parser: fiters[0].clone()};
    
    //let handle = thread::spawn(move || {
    //    let _v = Runtime::new().unwrap().block_on(listener.listen());
    //});

    for listener in listeners {
        handles.push(thread::spawn(move || {
            let _v = Runtime::new().unwrap().block_on(listener.listen());
            //let _v = listener.listen();
            
            
        }))
    }


    let json_listener: JSONListner = JSONListner { 
        db_uri: config.db_uri.clone(), sock_uri: "127.0.0.1:7878".to_string(), tags: (vec!["json".to_string(), "fluent".to_string()]), query_sender: query_sender.clone()};

    handles.push(thread::spawn(move || {
        let _v = Runtime::new().unwrap().block_on(json_listener.listen());
    }));


    print!("stuff");
    
    for handle in handles {
        handle.join().unwrap();
    }
    //handle.join().unwrap();
    print!("more stuff");
    loop {
        thread::sleep(time::Duration::from_millis(1000));
    }
    Ok(())
    
}


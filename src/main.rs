use crate::{result::Result, ingest::{SyslogListener, JSONListner}, config::Ingester, query::RealtimeQuerier};
//use serde_json::{json, Value};
//use chrono::NaiveDateTime;
//use scylla::Session;
//use syslog_loose::Message;
//use syslog_rfc5424::message;
//use syslog_rfc5424::SyslogMessage;
//use std::net::UdpSocket;
//use std::str;
//use uuid::Uuid;
//use std::fs::{File, OpenOptions, self};
mod db;
mod result;
//use crate::duration::Duration;
//use chrono::Duration;
mod duration;
//use std::time::SystemTime;
mod log_event;
mod ingest;
mod tcp_pool;
use std::{thread::{self, JoinHandle}, sync::mpsc, collections::HashMap, hash::Hash};
use tokio::runtime::Runtime;
//use tokio::join;
mod config;
mod parser;
mod pool;
mod query;
mod datastore;
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

    //let mut ds_channels: HashMap<String, mpsc::Sender<Value>> = HashMap::new();
    let mut ds_channels: HashMap<String, mpsc::Sender<log_event::DsEvent>> = HashMap::new();
    for ds in config.datastores {
        let (sender, receiver) = mpsc::channel();
        let mut ds_inst = datastore::DSWriter::new(ds.name.clone(), config.datapath.clone());
        //datastores.insert(ds.name.clone(), ds_inst);
        handles.push(thread::spawn(move || {
            ds_inst.start(receiver);
        }));
        ds_channels.insert(ds.name.clone(), sender);
    }

    let (query_sender, query_receiver) = mpsc::channel();
    let mut querier: RealtimeQuerier = query::RealtimeQuerier::new(config.queries, query_receiver);

    handles.push(thread::spawn(move || {
        querier.start();
    }));


    let mut listeners: Vec<SyslogListener> = vec![];
    //let mut tcp_listeners: Vec<JSONListner> = vec![];
    
    for ingester in config.ingesters {
        listeners.push(ingest::SyslogListener{
            //db_uri:config.db_uri.clone(),
            sock_uri:ingester.bind_addr,
            parser:parsers[&ingester.parser].clone(),
            tags: ingester.tags,
            sender: query_sender.clone(),
            threads: ingester.threads,
            ds_sender: ds_channels["default"].clone()
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
        sock_uri: "127.0.0.1:7878".to_string(), tags: (vec!["json".to_string(), "fluent".to_string()]), query_sender: query_sender.clone(), ds_sender:  ds_channels["default"].clone()};

    handles.push(thread::spawn(move || {
        let _v = Runtime::new().unwrap().block_on(json_listener.listen());
    }));


    print!("stuff");

    //let mut event_count: usize = 0;
    loop {
        thread::sleep(time::Duration::from_millis(3000));
        let search: datastore::DSReader = datastore::DSReader::new("default".to_owned(), config.datapath.clone(), 1696077600);
        let mut search_params: HashMap<String, String> = HashMap::new();
        search_params.insert("ACTION".to_owned(), "\"query\"".to_owned());
        //let search_result = search.search(search_params).unwrap();
        //println!("search length: {}", search_result["ACTION"].len());
        //let file_name: String = config.datapath.clone() + "/default/" + &datastore::DSWriter::gen_file_name();
        //let contents = match fs::read_to_string(file_name) {
        //    Ok(content) => content,
        //    Err(_) => "".to_string()
        //};
        //let count = contents.chars().map(|f| f ).filter(|&f | f == '\u{0003}').count();
        //drop(contents);
        //println!("Events per second:{}", (count - event_count) / 10);
        //event_count = count;
    }
    
    //for handle in handles {
    //    handle.join().unwrap();
    //}
    //handle.join().unwrap();
    //print!("more stuff");
    
    Ok(())
    
}


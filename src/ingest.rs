//use std::collections::HashMap;
//use std::collections::HashMap;
//use std::net::UdpSocket;
//use crate::result::Result;
use std::str;
use crate::parser;
use crate::pool;
//use crate::filter;
//use std::time;
use crate::log_event;
use syn::buffer;
use uuid::Uuid;
use crate::duration::Duration;
use tokio::runtime::Runtime;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream, UdpSocket}, collections::HashMap,
    time,
    thread
};
use std::time::SystemTime;
use std::{sync::{mpsc, Arc, Mutex}};
use crate::db;
use scylla::Session;
use crate::query::Query;
use serde_json::{json, Value};

//use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct SyslogListener {
    //pub db_uri: String,
    pub sock_uri: String,
    pub parser: parser::ParserCollection,
    pub tags: Vec<String>,
    pub sender: mpsc::Sender<HashMap<String, Value>>,
    pub threads: usize,
    //pub ds_sender: mpsc::Sender<Value>
    pub ds_sender: mpsc::Sender<log_event::DsEvent>
    
}

impl SyslogListener {
    pub async fn listen(self) -> Result<(), String> {
        println!("connecting to IP:{}", self.sock_uri);
        let s = UdpSocket::bind(&self.sock_uri).unwrap();
        let mut buf = [0u8; 2048];
        let pool = pool::ThreadPool::new(self.threads, self.parser, self.sender.clone(), self.ds_sender).await;

        
        loop {
            //print!("looping");
            let (data_read, src_address) = s.recv_from(&mut buf).unwrap();
            //let start = time::Instant::now();
            let original = str::from_utf8(&buf[0..data_read]).unwrap();   
            //println!("{}", original);
            pool.execute((original.to_string(), src_address.to_string(), self.tags.clone()));
            //let tduration = start.elapsed();
            //println!("timing:{:?}", tduration);
            
        }
    }

    
}

pub struct  JSONListner {
    //pub db_uri: String,
    pub sock_uri: String,
    pub tags: Vec<String>,
    pub query_sender: mpsc::Sender<HashMap<String, Value >>,
    //pub ds_sender: mpsc::Sender<Value>
    pub ds_sender: mpsc::Sender<log_event::DsEvent>
    //pub threads: usize
}

impl JSONListner {
    pub async fn listen(self) -> Result<(), String> {
        println!("Listening Tcp");
        
        //let mut session:Session;
        
        //let session = Runtime::new().unwrap().block_on(
        //    db::create_session(&self.db_uri)
        //).expect("Unable to connect to database.");
        //let session = Arc::new(session);
        
        //Runtime::new().unwrap().block_on(
        //    session = db::create_session(self.db_uri);
        //);
        //let session = Mutex::new(session);

        //let session = db::create_session(&self.db_uri).await.expect("Error connecting...");
        //let asession = Arc::new(session);
        let listener = match TcpListener:: bind(self.sock_uri) {
            Ok(tcplistener) => tcplistener,
            Err(v) => return Err(v.to_string()) 
        };

        for stream in listener.incoming() {
            println!("Incoming Stream");
            let stream = stream.unwrap();
            let tags = self.tags.clone();
            
            let qs = self.query_sender.clone();
            let ds_sender = self.ds_sender.clone();

            thread::spawn(move ||{
                let src_address = stream.peer_addr().unwrap();
                let mut handle = stream.take(409600000);
                let mut buf_reader = BufReader::new(&mut handle);
                let mut request: String = "".to_string();

                loop {
                    request.clear();
                    if buf_reader.read_line(&mut request).expect("cannot read stream") == 0{
                        println!("breaking");
                        break;
                    }
                    //let start = time::Instant::now();

                    let Ok(msg) = serde_json::from_str::<HashMap<String,Value>>(&request) else {
                        println!("Malformed packet!");
                        break;
                    };


                    
                    let event: log_event::DsEvent = log_event::DsEvent { 
                        id: Uuid::new_v4().to_string(), 
                        ingest_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                        source: src_address.to_string(), 
                        tags: tags.clone(), 
                        msg: serde_json::from_str(&request).unwrap(), 
                        original: request.to_string(), 
                        log_type: "json".to_string() };

                    //let buff = json!(event);
                    //ds_sender.send(buff).unwrap();
                    ds_sender.send(event).unwrap();

                    
                    qs.send(msg).unwrap();
                    //drop(event);
                    
                    //let tduration = start.elapsed();
                    //println!("TCP timing:{:?}", tduration);
                }

            });


            //let mut buf_reader = BufReader::new(&mut stream);
            //let mut data: String = "".to_string();
            
        }

        Ok(())
    }
   
}
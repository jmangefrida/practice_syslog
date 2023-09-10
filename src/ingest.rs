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
    pub db_uri: String,
    pub sock_uri: String,
    pub parser: parser::ParserCollection,
    pub tags: Vec<String>,
    pub sender: mpsc::Sender<Box<HashMap<String, Value>>>,
    pub threads: usize
    
}

impl SyslogListener {
    pub async fn listen(self) -> Result<(), String> {
        println!("connecting to IP:{}", self.sock_uri);
        let s = UdpSocket::bind(&self.sock_uri).unwrap();
        let mut buf = [0u8; 2048];
        let pool = pool::ThreadPool::new(self.threads, &self.db_uri, self.parser, self.sender.clone()).await;

        
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
    pub db_uri: String,
    pub sock_uri: String,
    pub tags: Vec<String>,
    pub query_sender: mpsc::Sender<Box<HashMap<String, Value >>>
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

        let session = db::create_session(&self.db_uri).await.expect("Error connecting...");
        let asession = Arc::new(session);
        let listener = match TcpListener:: bind(self.sock_uri) {
            Ok(tcplistener) => tcplistener,
            Err(v) => return Err(v.to_string()) 
        };

        for stream in listener.incoming() {
            println!("Incoming Stream");
            let mut stream = stream.unwrap();
            let tags = self.tags.clone();
            //let sn = Runtime::new().unwrap().block_on(
            //    Arc::clone(&session);
            //).expect("Error cloning session");
            //let sn = Arc::clone(&session);
            let db_uri = self.db_uri.clone();
            //let session = Runtime::new().unwrap().block_on(
            //    db::create_session(&db_uri)
            //).expect("Unable to connect to database.");
            let qs = self.query_sender.clone();
            let sn = Arc::clone(&asession);
            thread::spawn(move ||{
                //Runtime::new().unwrap().block_on(
                //session.refresh_topology()
                //).expect("bad refresh");
                let src_address = stream.peer_addr().unwrap();
                let mut buf_reader = BufReader::new(&mut stream);
                let mut request: String = "".to_string();
                //let query = Query::new("PRI = 30".to_owned());

                //let sn = Arc::new(session);
                loop {
                    request.clear();
                    if buf_reader.read_line(&mut request).expect("cannot read stream") == 0{
                        println!("breaking");
                        break;
                    }
                    //println!("{request}");

                    let event: log_event::DbEvent = log_event::DbEvent { 
                        id: Uuid::new_v4(),
                        ingest_time: Duration::seconds(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64),
                        source: src_address.to_string(),
                        tags: tags.clone(),
                        msg: request.clone(),
                        original: request.to_string(),
                        log_type: "json".to_string() };
    
                    //let sn = session.lock().unwrap();
                    
                    //Runtime::new().unwrap().block_on(
                    //    db::add_event(&session, &event)
                    //).expect("Error saving event to DB.");
                    //println!("{:?}", event);
                    Runtime::new().unwrap().block_on(
                        db::add_event(&sn, &event)  
                    ).unwrap();
                    let start = time::Instant::now();
                    qs.send(Box::new(serde_json::from_str(&request).unwrap())).unwrap();
                    //println!("{}", query.check(serde_json::from_str(&request).unwrap()));
                    let tduration = start.elapsed();
                    //println!("TCP timing:{:?}", tduration);
                }

            });


            //let mut buf_reader = BufReader::new(&mut stream);
            //let mut data: String = "".to_string();
            
        }

        Ok(())
    }
   
}
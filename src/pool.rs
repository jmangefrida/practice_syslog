/// Based on the threading pool example from the rust-lang book
/// A channel is created whtn the threadpool is instantiated.
/// The receiver end of the channel is then wrapped in a mutex to make it thread safe
/// When each worker is instantiated, a clone (Arc::clone) of reference to it is passed into the workers new()
/// This way multiple threads can access the same reciever.
/// 
/// The workers have loop (thread::spawn(move || loop {)
/// inside the loop the cloned mutex is locked.  the threads all pause there, so each thread is waiting at the lock for their turn.

use std::{sync::{mpsc, Arc, Mutex}, thread, collections::HashMap};
use scylla::Session;
use crate::log_event;
use uuid::Uuid;
use crate::duration::Duration;
use crate::result::Result;
use std::str;
use std::time::SystemTime;
use crate::db;
use crate::parser;
use crate::query::Query;
//use crate::filter;
use std::time;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Box<(String, String, Vec<String>)>>>,
    //ds_sender: mpsc::Sender<Value>
    ds_sender: mpsc::Sender<log_event::DsEvent>

}


impl ThreadPool {
    pub async fn new(size: usize, parser: parser::ParserCollection, query_sender: mpsc::Sender<HashMap<String, Value>>, ds_sender: mpsc::Sender<log_event::DsEvent>) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        //let session = db::create_session(db_uri).await.expect("Could not connect to database.");
        //let session = Arc::new(session);
        

        for id in 0..size {
            //let query: Query = Query::new("PRI = 134 AND ACTION = pass AND IP_VERSION=4 AND (HOSTNAME=cerberus.localdomain OR HOSTNAME=cerberus.localdomain)".to_owned());
            workers.push(Worker::new(id, Arc::clone(&receiver), parser.clone(), query_sender.clone(), ds_sender.clone()).await, );
        }

        ThreadPool {
            workers,
            sender: Some(sender),
            ds_sender: ds_sender
        }
    }

    //pub async fn initialize_db(db_uri: &str) -> Result<Session> {
    //    
    //    let session = db::create_session(&db_uri).await?;
    //    db::create_keyspace(&session).await?;
    //    db::create_table_log(&session).await?;
    //    //db::update_table_log(&session).await?;
    //    Ok(session)
    //}

    pub fn execute(& self, msg: (String, String, Vec<String>)){
        
        self.sender.as_ref().unwrap().send(Box::new(msg)).unwrap();
    }
    
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    async fn  new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Box<(String, String, Vec<String>)>>>>, parser: parser::ParserCollection, query_sender: mpsc::Sender<HashMap<String, Value>>, ds_sender: mpsc::Sender<log_event::DsEvent>,) -> Worker {
        //let session: Session = self.initialize_db().await.expect("Error connecting to database.");
        //let session = Runtime::new().unwrap().block_on(db::create_session(db_uri)).unwrap();
        println!("Session created for thread{}", id);
       
        let thread = thread::spawn( move ||
        
            loop {
                //println!("thread {} is waiting for messages", id);
                //let query: Query = Query::new("PRI = 134 OR PRI = 30.0".to_owned(), parser.value_type.clone());
                let message = receiver.lock().unwrap().recv();
                //println!("thread {} got a message", id);
                
                match message {
                    Ok(msg) => {

                        let (original, src_address, tags) = *msg;
                        let data = parser.parse(original.to_string());
                        //parser.parse(original.to_string());
                        
                        let event: log_event::DsEvent = log_event::DsEvent { 
                                                id: Uuid::new_v4().to_string(),
                                                ingest_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                                                source: src_address.to_string(),
                                                tags: tags,
                                                msg: json!(data),
                                                //data: parser.parse(original.to_string()),
                                                original: original.to_string(),
                                                log_type: parser.name.clone() };
                        //ds_sender.send(json!(event)).unwrap();
                        ds_sender.send(event).unwrap();
                        
                        //let event: log_event::DbEvent = log_event::DbEvent { 
                        //                        id: Uuid::new_v4(),
                        //                        ingest_time: Duration::seconds(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64),
                        //                        source: src_address.to_string(),
                        //                        tags: tags,
                        //                        msg: json!(data).to_string(),
                        //                        //data: parser.parse(original.to_string()),
                        //                        original: original.to_string(),
                        //                        log_type: parser.name.clone() };
            //
                        //
                        //
                        //Runtime::new().unwrap().block_on(
                        //    db::add_event(&session, &event)
//
                        //).expect("Could not save event to database.");
                        
                        
                        
                        //println!("message saved");
                        //println!("Message: {}, {}, {}", event.msg, event.source, event.original);
                        //println!("Message: {:?}", data);
                        let start = time::Instant::now();
                        //let query_result = query.check(data);
                        query_sender.send(data).unwrap();
                        let tduration = start.elapsed();                        
                        //println!("Result: {}", query_result);
                        //println!("timing:{:?}", tduration);

                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }


}
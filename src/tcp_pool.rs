use std::{sync::{mpsc, Arc, Mutex}, thread};
use scylla::Session;
use crate::log_event;
use uuid::Uuid;
use crate::duration::Duration;
use crate::result::Result;
use std::str;
use std::time::SystemTime;
use crate::db;
use crate::parser;
//use crate::filter;
use std::time;
use tokio::runtime::Runtime;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, collections::HashMap,
};
use serde_json;
use serde_json::Value;

pub struct TcpThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Box<(TcpStream, Vec<String>)>>> //(Stream, tags)

}

impl TcpThreadPool {
    pub async fn new(size: usize, db_uri: &str) -> TcpThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        let session = db::create_session(db_uri).await.unwrap();
        let session = Arc::new(session);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&session)).await);
        }

        TcpThreadPool {
            workers,
            sender: Some(sender)
        }
    }

    pub fn execute(&self, msg: (TcpStream, Vec<String>)){
        self.sender.as_ref().unwrap().send(Box::new(msg)).unwrap();
    }
}

impl Drop for TcpThreadPool {
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
    async fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Box<(TcpStream, Vec<String>)>>>>, session: Arc<Session>) -> Worker {
        let thread = thread::spawn( move ||
            loop {
                let message = receiver.lock().unwrap().recv();

            match message {
                Ok(msg) => {
                    let (mut stream, tags) = *msg;
                    let src_address = stream.peer_addr().unwrap();
                    let mut buf_reader = BufReader::new(&mut stream);
                    let mut request: String = "".to_string();
                    loop {
                        if buf_reader.read_line(&mut request).expect("cannot read stream") == 0{
                            println!("breaking");
                            break;
                        }
                        let start = time::Instant::now();
                        //let vals: HashMap<String, Value> = serde_json::from_str(&request).expect("vals error");
                        let event: log_event::LogEvent = log_event::LogEvent { 
                                                id: Uuid::new_v4(),
                                                ingest_time: Duration::seconds(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64),
                                                source: src_address.to_string(),
                                                tags: tags.clone(),
                                                msg: request.to_string(),
                                                data: HashMap::new(),
                                                log_type: "json".to_string() };
                        
                        //Runtime::new().unwrap().block_on(
                        //    db::add_event(&session, &event)
                        //).unwrap();request.clear();
                        let tduration = start.elapsed();

                    }


                },
                Err(_) => {
                    println!("Worker {id} disconnected: shutting down");
                    break;
                }
            }

            }
            

        );

        Worker{
            id,
            thread: Some(thread),
        }
    }
}
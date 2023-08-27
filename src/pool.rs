/// Based on the threading pool example from the rust-lang book
/// A channel is created whtn the threadpool is instantiated.
/// The receiver end of the channel is then wrapped in a mutex to make it thread safe
/// When each worker is instantiated, a clone (Arc::clone) of reference to it is passed into the workers new()
/// This way multiple threads can access the same reciever.
/// 
/// The workers have loop (thread::spawn(move || loop {)
/// inside the loop the cloned mutex is locked.  the threads all pause there, so each thread is waiting at the lock for their turn.

use std::{sync::{mpsc, Arc, Mutex}, thread};
use scylla::Session;

pub struct ThreadPool {
    workers: Vec<worker>,
    sender: Option<mpsc::Sender<Job>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(worker::new(id, Arc::clone(&reciever)));
        }

        ThreadPool {
            workers,
            sender: Some(sender)
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        //let session: Session = self.initialize_db().await.expect("Error connecting to database.");
        let session = db::create_session(&self.db_uri).await?;
        let thread = thread::spawn(move || 
            loop {
                db::add_event(&session, &event).await.expect("DB Error");
                match message {
                    Ok(job) => {
                        job();
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
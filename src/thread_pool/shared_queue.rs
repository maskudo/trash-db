use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self},
};

use super::ThreadPool;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct SharedQueueThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(n: usize) -> crate::Result<Self>
    where
        Self: Sized,
    {
        assert!(n > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(n);
        for i in 0..n {
            workers.push(Worker::new(i, receiver.clone()));
        }
        Ok(SharedQueueThreadPool {
            workers,
            sender: Some(sender),
        })
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                if let Err(e) = thread.join() {
                    println!("{:?}", e);
                };
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    // job might panic
                    // catch_unwind(move || job());
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        });
        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

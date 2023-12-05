use super::ThreadPool;
use log::error;
use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct SharedQueueThreadPool {
    sender: mpsc::Sender<Job>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(n: usize) -> crate::Result<Self>
    where
        Self: Sized,
    {
        assert!(n > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..n {
            let worker = Worker(receiver.clone());
            thread::spawn(move || run_task(worker));
        }
        Ok(Self { sender })
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
        self.sender.send(job).unwrap();
    }
}

fn run_task(worker: Worker) {
    loop {
        let job = worker.0.lock().unwrap().recv();
        match job {
            Ok(job) => job(),
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if thread::panicking() {
            let receiver = self.0.clone();
            thread::spawn(move || run_task(Worker(receiver)));
        }
    }
}

struct Worker(Arc<Mutex<Receiver<Job>>>);

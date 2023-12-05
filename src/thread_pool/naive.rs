use std::thread;

use super::ThreadPool;

pub struct NaiveThreadPool;
impl ThreadPool for NaiveThreadPool {
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }

    fn new(_: usize) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self)
    }
}

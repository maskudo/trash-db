use crate::Result;

pub mod naive;
pub mod rayon;
pub mod shared_queue;

pub trait ThreadPool {
    fn new(n: usize) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

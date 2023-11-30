use super::ThreadPool;

pub struct SharedQueueThreadPool;
impl ThreadPool for SharedQueueThreadPool {
    fn new(n: usize) -> crate::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        unimplemented!()
    }
}

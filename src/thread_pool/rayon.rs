use super::ThreadPool;

pub struct RayonThreadPool;
impl ThreadPool for RayonThreadPool {
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

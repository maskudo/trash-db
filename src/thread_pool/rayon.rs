use super::ThreadPool;

pub struct RayonThreadPool(rayon::ThreadPool);
impl ThreadPool for RayonThreadPool {
    fn new(n: usize) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pool = Self(rayon::ThreadPoolBuilder::new().num_threads(n).build()?);
        Ok(pool)
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.0.spawn(job)
    }
}

use crate::Result;

pub mod kvstore;
pub mod sled;

pub trait KvsEngine
where
    Self: Send + Clone + 'static,
{
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<()>;
}

use crate::cache::{CacheFacade, LockOps};
use parking_lot::RwLock;

// WHY: An in-memory only CacheFacade for tests.
#[derive(Default)]
pub struct InMemoryCacheService {
    memory: RwLock<Vec<(String, String)>>,
    persistent: RwLock<Vec<(String, String)>>,
}

impl CacheFacade for InMemoryCacheService {
    fn get_memory(&self, key: &str) -> Option<String> {
        let map = LockOps::read_guard(&self.memory);
        map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_memory(&self, key: &str, value: String) {
        let mut map = LockOps::write_guard(&self.memory);
        if let Some(pos) = map.iter().position(|(k, _)| k == key) {
            if let Some(entry) = map.get_mut(pos) {
                entry.1 = value;
            }
        } else {
            map.push((key.to_string(), value));
        }
    }

    fn get_persistent(&self, key: &str) -> Option<String> {
        let data = LockOps::read_guard(&self.persistent);
        data.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    fn set_persistent(&self, key: &str, value: String) -> anyhow::Result<()> {
        let mut data = LockOps::write_guard(&self.persistent);
        if let Some(pos) = data.iter().position(|(k, _)| k == key) {
            if let Some(entry) = data.get_mut(pos) {
                entry.1 = value;
            }
        } else {
            data.push((key.to_string(), value));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

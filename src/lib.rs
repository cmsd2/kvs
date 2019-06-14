use std::collections::BTreeMap;

pub struct KvStore {
    store: BTreeMap<String,String>,
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key,value);
    }
    
    pub fn get(&mut self, key: String) -> Option<String> {
        self.store.get(&key).map(|v| v.to_owned())
    }

    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }

    pub fn new() -> KvStore {
        KvStore {
            store: BTreeMap::new(),
        }
    }
}
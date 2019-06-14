use std::collections::BTreeMap;
use std::path::Path;
use std::fs;

pub mod result;
pub mod command;
pub mod kvdb;
pub mod logdb;

pub use result::*;
use kvdb::{KvDb,Visitor};
use command::Command;

pub const DEFAULT_FILE_NAME: &str = "kvs.json";

pub struct KvStore {
    kvdb: KvDb,
    store: BTreeMap<String,String>,
}

struct Loader {
    pub index: BTreeMap<String,String>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            index: BTreeMap::new(),
        }
    }
}

impl Visitor for Loader {
    fn command(&mut self, c: Command) -> Result<bool> {
        match c {
            Command::Set{key,value} => {
                self.index.insert(key,value);
            },
            Command::Remove{key} => {
                self.index.remove(&key);
            },
        }
        Ok(true)
    }
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.store.insert(key.clone(),value.clone());
        self.kvdb.append(Command::Set{key,value})?;
        Ok(())
    }
    
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self.store.get(&key).map(|v| v.to_owned()))
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.store.remove(&key).is_none() {
            Err(KvsErrorKind::NotFound(key))?;
        } else {
            self.kvdb.append(Command::Remove{key})?;
        }
        Ok(())
    }

    pub fn new(kvdb: KvDb, index: BTreeMap<String,String>) -> KvStore {
        KvStore {
            kvdb: kvdb,
            store: index,
        }
    }

    pub fn open(path: &Path) -> Result<KvStore> {
        let path = if fs::metadata(path).map_err(|e| KvsErrorKind::Io(e))?.is_dir() {
            path.to_owned().join(crate::DEFAULT_FILE_NAME)
        } else {
            path.to_owned()
        };

        let mut kvdb = KvDb::open(&path)?;
        let visitor = kvdb.visit(Loader::new())?;
        let store = KvStore::new(kvdb, visitor.index);
        Ok(store)
    }
}